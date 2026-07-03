//! # Concrete syntax tree
//!
//! The core representation: a whole calendar as generic, byte-faithful syntax.
//!
//! [`IcalCst`] is the hub of the crate. It models a component (the `VCALENDAR`
//! and every nested `VEVENT`, `VALARM`, `VTIMEZONE`, ...) as a `BEGIN` / `END`
//! envelope wrapping an ordered body of [`items`](IcalCst::items): property
//! lines and nested components, in source order, so anything round-trips byte
//! for byte even when a producer interleaves them. It knows nothing about what
//! a property or component *means*. It is filled from bytes ([`parse`](IcalCst::parse))
//! or from typed items, exports its bytes byte-faithfully
//! ([`to_bytes`](IcalCst::to_bytes), or the lossy-for-non-UTF-8
//! [`Display`](core::fmt::Display) / `to_string`), and offers typed access by
//! lens ([`prop`](IcalCst::prop), [`prop_mut`](IcalCst::prop_mut),
//! [`component`](IcalCst::component)). The semantic projection
//! ([`decode`](IcalCst::decode)) and the codec live in the
//! [`decode`](crate::tree::codec::decode) / [`encode`](crate::tree::codec::encode)
//! siblings.
//!
//! # Examples
//!
//! ```rust
//! use ical::tree::cst::IcalCst;
//! use ical::tree::component::vevent::VEVENT;
//! use ical::tree::prop::summary::SUMMARY;
//!
//! let raw = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//EN\r\nBEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\nSUMMARY:Lunch\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
//! let mut cst = IcalCst::parse(raw).unwrap();
//! assert_eq!(cst.to_string(), raw);
//!
//! cst.component_mut::<VEVENT>()
//!     .unwrap()
//!     .prop_mut::<SUMMARY>()
//!     .unwrap()
//!     .set_text("Dinner");
//! assert!(cst.to_string().contains("SUMMARY:Dinner\r\n"));
//! ```

use core::fmt;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::{
    prop::IcalProp,
    tree::{
        codec::mode::Escaper, component::IcalComponentLens, error::IcalParseError, line::IcalLine,
        prop::IcalPropLens,
    },
    version::IcalVersion,
};

/// One item in a component body: a property line, or a nested component.
#[derive(Clone, Debug)]
pub enum IcalItem<'a> {
    /// A property line.
    Prop(IcalLine<'a>),
    /// A nested component (its own `BEGIN` / `END` subtree).
    Component(IcalCst<'a>),
}

/// A component as raw syntax: an optional `BEGIN` / `END` envelope and the
/// ordered body of property lines and nested components between them. Used for
/// the `VCALENDAR` root and every subcomponent alike. The envelope is absent
/// only for a bare, envelope-less record parsed by
/// [`parse`](IcalCst::parse).
#[derive(Clone, Debug)]
pub struct IcalCst<'a> {
    /// The `BEGIN` line, or `None` for a bare record parsed without an
    /// envelope.
    pub begin: Option<IcalLine<'a>>,
    /// The body items (property lines and nested components), in source order.
    pub items: Vec<IcalItem<'a>>,
    /// The `END` line, absent exactly when [`begin`](Self::begin) is.
    pub end: Option<IcalLine<'a>>,
}

impl<'a> IcalCst<'a> {
    /// Start an empty iCalendar 2.0 calendar, BEGIN/VERSION/END seeded, ready
    /// for properties and components.
    pub fn v2() -> Self {
        Self {
            begin: Some(IcalLine::text("BEGIN", "VCALENDAR")),
            items: vec![IcalItem::Prop(IcalLine::text(
                "VERSION",
                &*IcalVersion::V2_0,
            ))],
            end: Some(IcalLine::text("END", "VCALENDAR")),
        }
    }

    /// Parse the first calendar from raw text, borrowing it for the Cst
    /// lifetime. A bare, envelope-less record (every line a property) is also
    /// accepted, so a lone component fragment round-trips.
    pub fn parse<T: AsRef<[u8]> + ?Sized>(input: &'a T) -> Result<Self, IcalParseError> {
        let input = trim_leading_eol(input.as_ref());
        let (first, _rest) = IcalLine::take(input)?;

        if first.name.get().eq_ignore_ascii_case("BEGIN") {
            let (mut cst, _rest) = Self::take_component(input)?;
            let escaper = Escaper::for_version_str(&cst.version_str());
            cst.stamp_escaper(escaper);
            Ok(cst)
        } else {
            Self::parse_bare(input)
        }
    }

    /// Parse a bare, envelope-less record: every line becomes a property.
    fn parse_bare(input: &'a [u8]) -> Result<Self, IcalParseError> {
        let mut items: Vec<IcalItem<'a>> = Vec::new();
        let mut rest = trim_leading_eol(input);

        while !rest.is_empty() {
            let (line, tail) = IcalLine::take(rest)?;
            items.push(IcalItem::Prop(line));
            rest = trim_leading_eol(tail);
        }

        let mut cst = Self {
            begin: None,
            items,
            end: None,
        };
        let escaper = Escaper::for_version_str(&cst.version_str());
        cst.stamp_escaper(escaper);
        Ok(cst)
    }

    /// Parse every top-level calendar in the input, lazily, one item per
    /// calendar (or the parse error that stopped iteration). Blank lines
    /// between calendars are skipped.
    pub fn parse_many<T: AsRef<[u8]> + ?Sized>(
        input: &'a T,
    ) -> impl Iterator<Item = Result<Self, IcalParseError>> {
        let mut rest = input.as_ref();

        core::iter::from_fn(move || {
            rest = trim_leading_eol(rest);
            if rest.is_empty() {
                return None;
            }

            match Self::take_component(rest) {
                Ok((mut cst, tail)) => {
                    rest = tail;
                    let escaper = Escaper::for_version_str(&cst.version_str());
                    cst.stamp_escaper(escaper);
                    Some(Ok(cst))
                }
                Err(error) => {
                    rest = b"";
                    Some(Err(error))
                }
            }
        })
    }

    /// Take one component (recursively) off the front of `input`, returning it
    /// and the unconsumed rest. A nested `BEGIN` recurses; the matching `END`
    /// closes this component.
    fn take_component(input: &'a [u8]) -> Result<(Self, &'a [u8]), IcalParseError> {
        let (begin, mut rest) = IcalLine::take(input)?;

        if !begin.name.get().eq_ignore_ascii_case("BEGIN") {
            return Err(IcalParseError::ExpectedBegin(begin.name.get().to_string()));
        }

        let mut items: Vec<IcalItem<'a>> = Vec::new();

        loop {
            if rest.is_empty() {
                return Err(IcalParseError::MissingEnd(
                    String::from_utf8_lossy(input).into_owned(),
                ));
            }

            let (line, tail) = IcalLine::take(rest)?;
            let name = line.name.get();

            if name.eq_ignore_ascii_case("END") {
                return Ok((
                    Self {
                        begin: Some(begin),
                        items,
                        end: Some(line),
                    },
                    tail,
                ));
            }

            if name.eq_ignore_ascii_case("BEGIN") {
                // A nested component starts at `rest` (the BEGIN line just
                // peeked); recurse from there and skip past its END.
                let (child, next) = Self::take_component(rest)?;
                items.push(IcalItem::Component(child));
                rest = next;
                continue;
            }

            items.push(IcalItem::Prop(line));
            rest = tail;
        }
    }

    /// Stamp the escaping mode onto every value node in the subtree, once the
    /// root `VERSION` is known (it can only be determined for the whole tree).
    fn stamp_escaper(&mut self, escaper: Escaper) {
        for item in &mut self.items {
            match item {
                IcalItem::Prop(line) => line.value.escaper = escaper,
                IcalItem::Component(child) => child.stamp_escaper(escaper),
            }
        }
    }

    /// The `VERSION` line among this component's direct properties, if any.
    fn version_str(&self) -> Cow<'_, str> {
        self.items
            .iter()
            .find_map(|item| match item {
                IcalItem::Prop(line) if line.name.get().eq_ignore_ascii_case("VERSION") => {
                    Some(line.raw_value_str())
                }
                _ => None,
            })
            .unwrap_or(Cow::Borrowed(""))
    }

    /// The calendar's version indicator, read from its `VERSION` line. An
    /// unrecognised or missing version normalises to
    /// [`V2_0`](IcalVersion::V2_0).
    pub fn version(&self) -> IcalVersion {
        self.version_str().parse().unwrap_or(IcalVersion::V2_0)
    }

    /// Append a typed property to this component, encoding it into a line.
    pub fn push(&mut self, prop: IcalProp<'a>) -> &mut Self {
        let escaper = Escaper::for_version_str(&self.version_str());
        self.items.push(IcalItem::Prop(prop.encode(escaper)));
        self
    }

    /// Append a nested component to this one.
    pub fn push_component(&mut self, component: IcalCst<'a>) -> &mut Self {
        self.items.push(IcalItem::Component(component));
        self
    }

    /// Remove every property of type `L` from this component's direct
    /// properties.
    pub fn remove<L: IcalPropLens>(&mut self) -> &mut Self {
        self.items.retain(|item| match item {
            IcalItem::Prop(line) => !line.name.get().eq_ignore_ascii_case(&L::KIND),
            IcalItem::Component(_) => true,
        });
        self
    }

    /// The first direct property of type `L`, decoded into a borrowed snapshot.
    pub fn prop<L: IcalPropLens>(&self) -> Option<L::Target<'_>> {
        let version = self.version();
        self.items.iter().find_map(|item| match item {
            IcalItem::Prop(line) if line.name.get().eq_ignore_ascii_case(&L::KIND) => {
                Some(L::decode(line, version))
            }
            _ => None,
        })
    }

    /// The first direct property of type `L`, as a typed cursor for in-place
    /// editing.
    pub fn prop_mut<L: IcalPropLens>(&mut self) -> Option<L::Cursor<'_, 'a>> {
        self.items.iter_mut().find_map(|item| match item {
            IcalItem::Prop(line) if line.name.get().eq_ignore_ascii_case(&L::KIND) => {
                Some(L::cursor(line))
            }
            _ => None,
        })
    }

    /// The first direct child component of type `C`, as a borrowed subtree.
    pub fn component<C: IcalComponentLens>(&self) -> Option<&IcalCst<'a>> {
        self.items.iter().find_map(|item| match item {
            IcalItem::Component(child) if child.is_kind::<C>() => Some(child),
            _ => None,
        })
    }

    /// The first direct child component of type `C`, mutably.
    pub fn component_mut<C: IcalComponentLens>(&mut self) -> Option<&mut IcalCst<'a>> {
        self.items.iter_mut().find_map(|item| match item {
            IcalItem::Component(child) if child.is_kind::<C>() => Some(child),
            _ => None,
        })
    }

    /// Every direct child component of type `C`, in source order.
    pub fn components<C: IcalComponentLens>(&self) -> impl Iterator<Item = &IcalCst<'a>> {
        self.items.iter().filter_map(|item| match item {
            IcalItem::Component(child) if child.is_kind::<C>() => Some(child),
            _ => None,
        })
    }

    /// Whether this component's `BEGIN` name matches the lens marker `C`.
    fn is_kind<C: IcalComponentLens>(&self) -> bool {
        self.begin
            .as_ref()
            .map(|begin| begin.raw_value_str().eq_ignore_ascii_case(&C::KIND))
            .unwrap_or(false)
    }

    /// The wire name of this component (its `BEGIN` value), or `""` for a bare
    /// record.
    pub(crate) fn component_name(&self) -> Cow<'_, str> {
        self.begin
            .as_ref()
            .map(|begin| begin.raw_value_str())
            .unwrap_or(Cow::Borrowed(""))
    }

    /// Own every borrowed leaf, detaching the calendar from the source bytes so
    /// it can outlive them.
    pub fn into_static(self) -> IcalCst<'static> {
        IcalCst {
            begin: self.begin.map(IcalLine::into_static),
            items: self
                .items
                .into_iter()
                .map(|item| match item {
                    IcalItem::Prop(line) => IcalItem::Prop(line.into_static()),
                    IcalItem::Component(child) => IcalItem::Component(child.into_static()),
                })
                .collect(),
            end: self.end.map(IcalLine::into_static),
        }
    }

    /// Serialize the calendar to raw bytes, exactly as parsed.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write_bytes(&mut out);
        out
    }

    fn write_bytes(&self, out: &mut Vec<u8>) {
        if let Some(begin) = &self.begin {
            begin.write_bytes(out);
        }
        for item in &self.items {
            match item {
                IcalItem::Prop(line) => line.write_bytes(out),
                IcalItem::Component(child) => child.write_bytes(out),
            }
        }
        if let Some(end) = &self.end {
            end.write_bytes(out);
        }
    }
}

/// Skip leading blank-line bytes (`\r` / `\n`) between calendars.
fn trim_leading_eol(mut bytes: &[u8]) -> &[u8] {
    while let Some((first, rest)) = bytes.split_first() {
        if matches!(first, b'\r' | b'\n') {
            bytes = rest;
        } else {
            break;
        }
    }

    bytes
}

impl fmt::Display for IcalCst<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(begin) = &self.begin {
            write!(f, "{begin}")?;
        }
        for item in &self.items {
            match item {
                IcalItem::Prop(line) => write!(f, "{line}")?,
                IcalItem::Component(child) => write!(f, "{child}")?,
            }
        }
        if let Some(end) = &self.end {
            write!(f, "{end}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::{component::vevent::VEVENT, cst::IcalCst, prop::summary::SUMMARY};

    const CAL: &str = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:1\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "SUMMARY:Lunch\r\n",
        "BEGIN:VALARM\r\n",
        "ACTION:DISPLAY\r\n",
        "TRIGGER:-PT15M\r\n",
        "END:VALARM\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    #[test]
    fn round_trips_a_nested_calendar_byte_for_byte() {
        let cst = IcalCst::parse(CAL).unwrap();
        assert_eq!(cst.to_string(), CAL);
    }

    #[test]
    fn reads_a_nested_property_through_component_and_prop_lenses() {
        let cst = IcalCst::parse(CAL).unwrap();
        let event = cst.component::<VEVENT>().expect("a VEVENT");
        assert_eq!(&*event.prop::<SUMMARY>().unwrap().0, "Lunch");
    }

    #[test]
    fn edits_a_nested_property_leaving_every_other_byte_intact() {
        let mut cst = IcalCst::parse(CAL).unwrap();
        cst.component_mut::<VEVENT>()
            .unwrap()
            .prop_mut::<SUMMARY>()
            .unwrap()
            .set_text("Dinner");
        assert_eq!(
            cst.to_string(),
            CAL.replace("SUMMARY:Lunch", "SUMMARY:Dinner")
        );
    }

    #[test]
    fn reports_the_version() {
        let cst = IcalCst::parse(CAL).unwrap();
        assert_eq!(cst.version(), crate::version::IcalVersion::V2_0);
    }
}
