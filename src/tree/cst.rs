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
    boxed::Box,
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
    /// A nested component (its own `BEGIN` / `END` subtree). Boxed, since a
    /// component is recursive and a property line is not.
    Component(Box<IcalCst<'a>>),
    /// One physical line that could not be structured, kept verbatim (its
    /// ending included) so it still round-trips. Only
    /// [`parse_recovering`](IcalCst::parse_recovering) ever produces one: the
    /// strict entry points refuse the calendar instead.
    Opaque(Cow<'a, [u8]>),
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
    /// The blank lines after `END`, kept so a file ending in one round-trips.
    /// Only ever set on a root calendar, and only when nothing but whitespace
    /// follows it.
    pub trailing: Cow<'a, str>,
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
            trailing: Cow::Borrowed(""),
        }
    }

    /// Parse the first calendar from raw text, borrowing it for the Cst
    /// lifetime. A bare, envelope-less record (every line a property) is also
    /// accepted, so a lone component fragment round-trips.
    ///
    /// Anything after the first calendar is not part of it and is dropped,
    /// except trailing blank lines, which are kept: use
    /// [`parse_many`](Self::parse_many) to read a multi-calendar file whole.
    pub fn parse<T: AsRef<[u8]> + ?Sized>(input: &'a T) -> Result<Self, IcalParseError> {
        let input = input.as_ref();
        let (first, _rest) = IcalLine::take(input)?;

        if first.name.get().eq_ignore_ascii_case("BEGIN") {
            let (mut cst, rest) = Self::take_component(input)?;
            cst.take_trailing(rest);
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
        let mut rest = input;

        while !is_blank(rest) {
            let (line, tail) = IcalLine::take(rest)?;
            items.push(IcalItem::Prop(line));
            rest = tail;
        }

        let mut cst = Self {
            begin: None,
            items,
            end: None,
            trailing: Cow::Borrowed(""),
        };
        cst.take_trailing(rest);
        let escaper = Escaper::for_version_str(&cst.version_str());
        cst.stamp_escaper(escaper);
        Ok(cst)
    }

    /// Parse every top-level calendar in the input, lazily, one item per
    /// calendar (or the parse error that stopped iteration).
    ///
    /// Blank lines between calendars belong to the calendar that follows them,
    /// and blank lines after the last one to the last calendar, so
    /// concatenating what this yields reproduces the file byte for byte.
    pub fn parse_many<T: AsRef<[u8]> + ?Sized>(
        input: &'a T,
    ) -> impl Iterator<Item = Result<Self, IcalParseError>> {
        let mut rest = input.as_ref();

        core::iter::from_fn(move || {
            if is_blank(rest) {
                return None;
            }

            match Self::take_component(rest) {
                Ok((mut cst, tail)) => {
                    rest = cst.take_trailing(tail);
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

    /// Parse the whole input, recovering from anything that cannot be
    /// structured instead of refusing the calendar.
    ///
    /// A physical line with no colon, or whose name is not UTF-8, is kept as an
    /// [`Opaque`](IcalItem::Opaque) item and parsing carries on. A component
    /// left open at end of input is closed with no `END`. Either way the bytes
    /// survive, so the recovered calendars still serialize back to the input,
    /// and every problem is reported in [`IcalRecovery::problems`].
    ///
    /// The strict entry points ([`parse`](Self::parse),
    /// [`parse_many`](Self::parse_many)) are unchanged and stay the default:
    /// use this one when a calendar from the wild matters more than the
    /// guarantee that it was well formed.
    pub fn parse_recovering<T: AsRef<[u8]> + ?Sized>(input: &'a T) -> IcalRecovery<'a> {
        let mut rest = input.as_ref();
        let mut recovery = IcalRecovery::default();

        // NOTE: Items outside any BEGIN, which is where a bare record's properties
        // and any stray line land.
        let mut loose: Vec<IcalItem<'a>> = Vec::new();

        while !is_blank(rest) {
            match IcalLine::take(rest) {
                Ok((line, _tail)) if line.name.get().eq_ignore_ascii_case("BEGIN") => {
                    recovery.close_loose(&mut loose);

                    let (mut cst, tail) = Self::take_component_recovering(rest, &mut recovery);
                    rest = tail;
                    let escaper = Escaper::for_version_str(&cst.version_str());
                    cst.stamp_escaper(escaper);
                    recovery.calendars.push(cst);
                }
                Ok((line, tail)) => {
                    loose.push(IcalItem::Prop(line));
                    rest = tail;
                }
                Err(error) => {
                    let (opaque, tail) = IcalLine::take_physical(rest);
                    loose.push(IcalItem::Opaque(Cow::Borrowed(opaque)));
                    recovery.problems.push(error);
                    rest = tail;
                }
            }
        }

        recovery.close_loose(&mut loose);

        if let Some(last) = recovery.calendars.last_mut() {
            last.take_trailing(rest);
        } else {
            let mut bare = Self::bare(Vec::new());
            bare.take_trailing(rest);
            recovery.calendars.push(bare);
        }

        recovery
    }

    /// Take one component recovering from what it cannot structure: an
    /// unstructurable line becomes an opaque item, and an unclosed component is
    /// closed at end of input.
    fn take_component_recovering(
        input: &'a [u8],
        recovery: &mut IcalRecovery<'a>,
    ) -> (Self, &'a [u8]) {
        // NOTE: The caller only ever enters here on a line that tokenised as BEGIN.
        let (begin, mut rest) = IcalLine::take(input).expect("a BEGIN line");
        let name = begin.raw_value_str().into_owned();

        let mut items: Vec<IcalItem<'a>> = Vec::new();

        loop {
            if is_blank(rest) {
                recovery.problems.push(IcalParseError::MissingEnd(name));
                return (
                    Self {
                        begin: Some(begin),
                        items,
                        end: None,
                        trailing: Cow::Borrowed(""),
                    },
                    rest,
                );
            }

            match IcalLine::take(rest) {
                Ok((line, tail)) => {
                    let line_name = line.name.get();

                    if line_name.eq_ignore_ascii_case("END") {
                        return (
                            Self {
                                begin: Some(begin),
                                items,
                                end: Some(line),
                                trailing: Cow::Borrowed(""),
                            },
                            tail,
                        );
                    }

                    if line_name.eq_ignore_ascii_case("BEGIN") {
                        let (child, next) = Self::take_component_recovering(rest, recovery);
                        items.push(IcalItem::Component(Box::new(child)));
                        rest = next;
                        continue;
                    }

                    items.push(IcalItem::Prop(line));
                    rest = tail;
                }
                Err(error) => {
                    let (opaque, tail) = IcalLine::take_physical(rest);
                    items.push(IcalItem::Opaque(Cow::Borrowed(opaque)));
                    recovery.problems.push(error);
                    rest = tail;
                }
            }
        }
    }

    /// A bare, envelope-less calendar around `items`.
    fn bare(items: Vec<IcalItem<'a>>) -> Self {
        Self {
            begin: None,
            items,
            end: None,
            trailing: Cow::Borrowed(""),
        }
    }

    /// Keep `rest` as this calendar's trailing blank lines when nothing but
    /// whitespace follows, and report what is left to parse.
    fn take_trailing(&mut self, rest: &'a [u8]) -> &'a [u8] {
        if !is_blank(rest) {
            return rest;
        }

        self.trailing = Cow::Borrowed(str::from_utf8(rest).unwrap_or(""));
        b""
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
                // NOTE: The component's name, not the whole input: an error that
                // carries a megabyte of calendar is not a diagnostic.
                return Err(IcalParseError::MissingEnd(
                    begin.raw_value_str().into_owned(),
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
                        trailing: Cow::Borrowed(""),
                    },
                    tail,
                ));
            }

            if name.eq_ignore_ascii_case("BEGIN") {
                // NOTE: A nested component starts at `rest` (the BEGIN line just
                // peeked); recurse from there and skip past its END.
                let (child, next) = Self::take_component(rest)?;
                items.push(IcalItem::Component(Box::new(child)));
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
                IcalItem::Opaque(_) => {}
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
        self.items.push(IcalItem::Component(Box::new(component)));
        self
    }

    /// Remove every property of type `L` from this component's direct
    /// properties.
    pub fn remove<L: IcalPropLens>(&mut self) -> &mut Self {
        self.items.retain(|item| match item {
            IcalItem::Prop(line) => !line.name.get().eq_ignore_ascii_case(&L::KIND),
            IcalItem::Component(_) => true,
            IcalItem::Opaque(_) => true,
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
            IcalItem::Component(child) if child.is_kind::<C>() => Some(&**child),
            _ => None,
        })
    }

    /// The first direct child component of type `C`, mutably.
    pub fn component_mut<C: IcalComponentLens>(&mut self) -> Option<&mut IcalCst<'a>> {
        self.items.iter_mut().find_map(|item| match item {
            IcalItem::Component(child) if child.is_kind::<C>() => Some(&mut **child),
            _ => None,
        })
    }

    /// Every direct child component of type `C`, in source order.
    pub fn components<C: IcalComponentLens>(&self) -> impl Iterator<Item = &IcalCst<'a>> {
        self.items.iter().filter_map(|item| match item {
            IcalItem::Component(child) if child.is_kind::<C>() => Some(&**child),
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
                    IcalItem::Component(child) => {
                        IcalItem::Component(Box::new(child.into_static()))
                    }
                    IcalItem::Opaque(bytes) => IcalItem::Opaque(Cow::Owned(bytes.into_owned())),
                })
                .collect(),
            end: self.end.map(IcalLine::into_static),
            trailing: Cow::Owned(self.trailing.into_owned()),
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
                IcalItem::Opaque(bytes) => out.extend_from_slice(bytes),
                IcalItem::Component(child) => child.write_bytes(out),
            }
        }
        if let Some(end) = &self.end {
            end.write_bytes(out);
        }
        out.extend_from_slice(self.trailing.as_bytes());
    }
}

/// What a recovering parse read: the calendars it could structure, and every
/// problem it worked around.
///
/// The bytes are never lost, whatever the problems: serializing the calendars
/// in order reproduces the input.
#[derive(Clone, Debug, Default)]
pub struct IcalRecovery<'a> {
    /// Every top-level calendar, in source order. A run of lines outside any
    /// `BEGIN` becomes a bare, envelope-less calendar of its own.
    pub calendars: Vec<IcalCst<'a>>,
    /// What could not be structured, in the order it was met.
    pub problems: Vec<IcalParseError>,
}

impl<'a> IcalRecovery<'a> {
    /// Whether the input parsed with nothing to work around, in which case a
    /// strict parse would have accepted it too.
    pub fn is_clean(&self) -> bool {
        self.problems.is_empty()
    }

    /// Serialize every calendar, in order: the input, byte for byte.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        for cst in &self.calendars {
            cst.write_bytes(&mut out);
        }

        out
    }

    /// Close a run of loose items into a bare calendar, if there is one.
    fn close_loose(&mut self, loose: &mut Vec<IcalItem<'a>>) {
        if loose.is_empty() {
            return;
        }

        self.calendars.push(IcalCst::bare(core::mem::take(loose)));
    }
}

/// Whether nothing but blank-line bytes (`\r` / `\n`) is left.
fn is_blank(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| matches!(byte, b'\r' | b'\n'))
}

impl fmt::Display for IcalCst<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(begin) = &self.begin {
            write!(f, "{begin}")?;
        }
        for item in &self.items {
            match item {
                IcalItem::Prop(line) => write!(f, "{line}")?,
                IcalItem::Opaque(bytes) => f.write_str(&String::from_utf8_lossy(bytes))?,
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
    use alloc::{
        string::{String, ToString},
        vec::Vec,
    };

    use crate::tree::{
        component::vevent::VEVENT,
        cst::IcalCst,
        error::IcalParseError,
        prop::{prodid::PRODID, summary::SUMMARY},
    };

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

    #[test]
    fn round_trips_a_folded_calendar_byte_for_byte() {
        // NOTE: What a real exporter emits: folded at a column, blank lines between
        // components, and a blank line at the end of the file.
        let raw = concat!(
            "BEGIN:VCALENDAR\r\n",
            "VERSION:2.0\r\n",
            "PRODID:-//Example//EN\r\n",
            "\r\n",
            "BEGIN:VEVENT\r\n",
            "UID:1\r\n",
            "DTSTAMP:20260101T000000Z\r\n",
            "DESCRIPTION:a very long description that an exporter would fold at s\r\n",
            " ome column\r\n",
            "END:VEVENT\r\n",
            "END:VCALENDAR\r\n",
            "\r\n",
        );

        let cst = IcalCst::parse(raw).unwrap();
        assert_eq!(String::from_utf8(cst.to_bytes()).unwrap(), raw);
    }

    #[test]
    fn round_trips_a_leading_blank_line() {
        let raw = "\r\nBEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";
        let cst = IcalCst::parse(raw).unwrap();
        assert_eq!(String::from_utf8(cst.to_bytes()).unwrap(), raw);
    }

    #[test]
    fn round_trips_a_whole_multi_calendar_file() {
        // NOTE: `parse` reads the first calendar and stops, so a file holding several
        // round-trips through `parse_many`, whose output concatenates to the
        // input, blank lines between calendars included.
        let raw = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n",
            "\r\n",
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n",
        );

        let mut out = Vec::new();
        for cst in IcalCst::parse_many(raw) {
            out.extend_from_slice(&cst.unwrap().to_bytes());
        }

        assert_eq!(String::from_utf8(out).unwrap(), raw);
    }

    #[test]
    fn recovers_a_line_with_no_colon() {
        let raw = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n",
            "this line has no colon\r\n",
            "PRODID:-//Example//EN\r\nEND:VCALENDAR\r\n",
        );

        assert!(IcalCst::parse(raw).is_err());

        let recovery = IcalCst::parse_recovering(raw);
        assert_eq!(String::from_utf8(recovery.to_bytes()).unwrap(), raw);
        assert_eq!(recovery.calendars.len(), 1);
        assert!(matches!(
            recovery.problems.as_slice(),
            [IcalParseError::MissingPropertyColon(_)]
        ));

        // NOTE: The rest of the calendar survived: the property after the bad line is
        // there to be read.
        let cal = &recovery.calendars[0];
        assert_eq!(&*cal.prop::<PRODID>().unwrap().0, "-//Example//EN");
    }

    #[test]
    fn recovers_a_component_with_no_end() {
        let raw = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:1\r\n";

        assert!(IcalCst::parse(raw).is_err());

        let recovery = IcalCst::parse_recovering(raw);
        assert_eq!(String::from_utf8(recovery.to_bytes()).unwrap(), raw);
        assert_eq!(
            recovery.problems,
            [
                IcalParseError::MissingEnd("VEVENT".into()),
                IcalParseError::MissingEnd("VCALENDAR".into()),
            ]
        );
        assert!(recovery.calendars[0].component::<VEVENT>().is_some());
    }

    #[test]
    fn reports_nothing_for_a_calendar_the_strict_parser_accepts() {
        let recovery = IcalCst::parse_recovering(CAL);
        assert!(recovery.is_clean());
        assert_eq!(String::from_utf8(recovery.to_bytes()).unwrap(), CAL);
    }

    #[test]
    fn refolds_nothing_once_a_value_is_edited() {
        let raw = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\n",
            "SUMMARY:a summary long enough to have been fol\r\n ded by its exporter\r\n",
            "END:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        let mut cst = IcalCst::parse(raw).unwrap();
        cst.component_mut::<VEVENT>()
            .unwrap()
            .prop_mut::<SUMMARY>()
            .unwrap()
            .set_text("Dinner");

        assert_eq!(
            String::from_utf8(cst.to_bytes()).unwrap(),
            concat!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\n",
                "SUMMARY:Dinner\r\n",
                "END:VEVENT\r\nEND:VCALENDAR\r\n",
            )
        );
    }
}
