//! # Encode (model to syntax)
//!
//! The write side of the structural bridge: project the decoded model onto a
//! raw syntax tree.
//!
//! A value's [`Codec`](crate::tree::codec::Codec) impl encodes it into a
//! [`IcalValueNode`], an [`IcalParam`] into an [`IcalParamNode`], a
//! [`IcalProp`] into an [`IcalLine`], an [`IcalComponent`] into a nested
//! [`IcalCst`], and an [`Ical`] into the whole `VCALENDAR` [`IcalCst`],
//! recursively.
//!
//! The whole calendar is encoded for its version's [`Escaper`], which the
//! value codecs use to escape every leaf.
//!
//! [`Display`](core::fmt::Display) for [`Ical`] writes the bytes that tree
//! would serialize to straight from the model: each value type spells its
//! wire shape once, in an `emit` that writes either a node's leaves or text.

use core::{fmt, mem, slice};

use alloc::{
    borrow::Cow,
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    component::{IcalComponent, IcalComponentName},
    ical::Ical,
    param::IcalParam,
    prop::{IcalProp, IcalPropName},
    tree::{
        codec::{
            escape::{escape_into, escape_param, escape_with, position},
            mode::Escaper,
            unescape::text,
        },
        cst::{IcalCst, IcalItem, push_line},
        leaf::{IcalLeaf, IcalValueLeaf},
        line::{IcalLine, fold_long},
        param::node::IcalParamNode,
        value::node::IcalValueNode,
    },
    validator::IcalValid,
};

impl Ical<'_> {
    /// Encode the whole calendar into a `VCALENDAR` CST for its version's
    /// escaping mode. `VERSION` is emitted as the first property.
    pub fn encode(&self) -> IcalCst<'static> {
        let escaper = Escaper::for_version(self.version);

        let mut items = Vec::with_capacity(1 + self.props.len() + self.components.len());
        items.push(IcalItem::Prop(IcalLine::text(
            "VERSION",
            self.version.wire_name(),
        )));
        encode_items(&self.props, &self.components, escaper, &mut items);

        IcalCst {
            begin: Some(IcalLine::text("BEGIN", "VCALENDAR")),
            items,
            end: Some(IcalLine::text("END", "VCALENDAR")),
            trailing: Cow::Borrowed(""),
        }
    }
}

impl IcalComponent<'_> {
    /// Encode this component (and its nested components) into a CST for the
    /// given escaping mode.
    pub fn encode(&self, escaper: Escaper) -> IcalCst<'static> {
        let mut cst = IcalCst::bare(Vec::new());
        self.encode_into(escaper, &mut cst);
        cst
    }

    /// [`encode`](Self::encode) into `cst`, a bare component holding nothing.
    fn encode_into(&self, escaper: Escaper, cst: &mut IcalCst<'static>) {
        // NOTE: A known name is its canonical spelling, borrowed for good.
        let name: Cow<'static, str> = match &self.name {
            IcalComponentName::Kind(kind) => Cow::Borrowed(kind.wire_name()),
            IcalComponentName::Unknown(name) => Cow::Owned(name.to_string()),
        };

        cst.items
            .reserve_exact(self.props.len() + self.components.len());
        encode_items(&self.props, &self.components, escaper, &mut cst.items);
        cst.begin = Some(IcalLine::text("BEGIN", name.clone()));
        cst.end = Some(IcalLine::text("END", name));
    }

    /// Append the lines [`encode`](Self::encode) builds, as they serialize.
    fn write_text(&self, escaper: Escaper, out: &mut Vec<u8>) {
        write_text_line(out, "BEGIN", &self.name);

        for prop in &self.props {
            prop.write_text(escaper, out);
        }

        for component in &self.components {
            component.write_text(escaper, out);
        }

        write_text_line(out, "END", &self.name);
    }

    /// About how many bytes [`write_text`](Self::write_text) appends: a short
    /// line a property, which most are.
    fn text_size_hint(&self) -> usize {
        let components = self.components.iter().map(Self::text_size_hint);
        64 + 32 * self.props.len() + components.sum::<usize>()
    }
}

impl<'a> From<Ical<'a>> for IcalCst<'static> {
    fn from(cal: Ical<'a>) -> Self {
        cal.encode()
    }
}

impl<'a> From<IcalValid<Ical<'a>>> for IcalCst<'static> {
    fn from(valid: IcalValid<Ical<'a>>) -> Self {
        valid.into_inner().encode()
    }
}

impl IcalProp<'_> {
    /// Encode the property into a raw content line for the given escaping mode,
    /// dispatching on its value.
    pub fn encode(&self, escaper: Escaper) -> IcalLine<'static> {
        let mut line = IcalLine::empty();
        self.encode_into(escaper, &mut line);
        line
    }

    /// [`encode`](Self::encode) into `slot`, an [empty](IcalLine::empty) line
    /// whose fields are written over without being dropped, there being
    /// nothing in them to free.
    ///
    /// Its wire shape stays empty: an encoded property has no wire history,
    /// it is written out in canonical form, folded at 75 octets.
    fn encode_into(&self, escaper: Escaper, slot: &mut IcalLine<'static>) {
        // NOTE: A known name is its canonical spelling, borrowed for good.
        let name = match &self.name {
            IcalPropName::Kind(kind) => Cow::Borrowed(kind.wire_name()),
            IcalPropName::Unknown(name) => Cow::Owned(name.to_string()),
        };

        mem::forget(mem::replace(&mut slot.name, IcalLeaf(name)));
        slot.params.reserve_exact(self.params.len());
        slot.params
            .extend(self.params.iter().map(|param| param.encode(escaper)));
        slot.value.escaper = escaper;
        self.value.emit(&mut IcalValueOut {
            escaper,
            sink: Sink::Node(&mut slot.value),
        });
        mem::forget(mem::replace(&mut slot.eol, IcalLeaf::from("\r\n")));
    }

    /// Append the line [`encode`](Self::encode) builds, as it serializes.
    fn write_text(&self, escaper: Escaper, out: &mut Vec<u8>) {
        let start = out.len();

        out.extend_from_slice(self.name.as_bytes());

        for param in &self.params {
            out.push(b';');
            param.write_text(escaper, out);
        }

        out.push(b':');

        self.value.emit(&mut IcalValueOut {
            escaper,
            sink: Sink::Bytes {
                out,
                started: false,
            },
        });

        end_line(out, start);
    }
}

impl IcalParam<'_> {
    /// Encode the parameter into a raw parameter node for the given escaping
    /// mode, dispatching on its kind.
    pub fn encode(&self, escaper: Escaper) -> IcalParamNode<'static> {
        let (name, values) = self.wire();

        IcalParamNode {
            name: IcalLeaf(match self.kind() {
                Some(kind) => Cow::Borrowed(kind.wire_name()),
                None => Cow::Owned(name.to_string()),
            }),
            values: values
                .iter()
                .map(|value| IcalLeaf::from(escape_param(value, escaper).into_owned()))
                .collect(),
            escaper,
        }
    }

    /// Append the node [`encode`](Self::encode) builds, as it serializes.
    fn write_text(&self, escaper: Escaper, out: &mut Vec<u8>) {
        let (name, values) = self.wire();

        out.extend_from_slice(name.as_bytes());

        for (i, value) in values.iter().enumerate() {
            out.push(if i == 0 { b'=' } else { b',' });
            out.extend_from_slice(escape_param(value, escaper).as_bytes());
        }
    }

    /// The parameter's wire name and its values, not yet encoded: a scalar
    /// kind holds one.
    fn wire(&self) -> (&str, &[Cow<'_, str>]) {
        use crate::param::IcalParamKind::*;

        match self {
            IcalParam::AltRep(v) => (&AltRep, slice::from_ref(v)),
            IcalParam::Cn(v) => (&Cn, slice::from_ref(v)),
            IcalParam::CuType(v) => (&CuType, slice::from_ref(v)),
            IcalParam::DelegatedFrom(vs) => (&DelegatedFrom, vs),
            IcalParam::DelegatedTo(vs) => (&DelegatedTo, vs),
            IcalParam::Dir(v) => (&Dir, slice::from_ref(v)),
            IcalParam::Encoding(v) => (&Encoding, slice::from_ref(v)),
            IcalParam::FmtType(v) => (&FmtType, slice::from_ref(v)),
            IcalParam::FbType(v) => (&FbType, slice::from_ref(v)),
            IcalParam::Language(v) => (&Language, slice::from_ref(v)),
            IcalParam::Member(vs) => (&Member, vs),
            IcalParam::PartStat(v) => (&PartStat, slice::from_ref(v)),
            IcalParam::Range(v) => (&Range, slice::from_ref(v)),
            IcalParam::Related(v) => (&Related, slice::from_ref(v)),
            IcalParam::RelType(v) => (&RelType, slice::from_ref(v)),
            IcalParam::Role(v) => (&Role, slice::from_ref(v)),
            IcalParam::Rsvp(v) => (&Rsvp, slice::from_ref(v)),
            IcalParam::SentBy(v) => (&SentBy, slice::from_ref(v)),
            IcalParam::TzId(v) => (&TzId, slice::from_ref(v)),
            IcalParam::Value(v) => (&Value, slice::from_ref(v)),
            IcalParam::Display(v) => (&Display, slice::from_ref(v)),
            IcalParam::Email(v) => (&Email, slice::from_ref(v)),
            IcalParam::Feature(vs) => (&Feature, vs),
            IcalParam::Label(v) => (&Label, slice::from_ref(v)),
            IcalParam::Order(v) => (&Order, slice::from_ref(v)),
            IcalParam::Schema(v) => (&Schema, slice::from_ref(v)),
            IcalParam::Derived(v) => (&Derived, slice::from_ref(v)),
            IcalParam::ScheduleAgent(v) => (&ScheduleAgent, slice::from_ref(v)),
            IcalParam::ScheduleForceSend(v) => (&ScheduleForceSend, slice::from_ref(v)),
            IcalParam::ScheduleStatus(v) => (&ScheduleStatus, slice::from_ref(v)),
            IcalParam::LinkRel(v) => (&LinkRel, slice::from_ref(v)),
            IcalParam::Gap(v) => (&Gap, slice::from_ref(v)),
            IcalParam::Charset(v) => (&Charset, slice::from_ref(v)),

            IcalParam::Unknown { name, values } => (name, values),
        }
    }
}

/// Serialize the decoded calendar as its [`encode`](Ical::encode)d tree
/// serializes, written straight from the model.
impl fmt::Display for Ical<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let escaper = Escaper::for_version(self.version);

        let components = self.components.iter().map(IcalComponent::text_size_hint);
        let size = 64 + 32 * self.props.len() + components.sum::<usize>();
        let mut out = Vec::with_capacity(size);

        write_text_line(&mut out, "BEGIN", "VCALENDAR");
        write_text_line(&mut out, "VERSION", &self.version);

        for prop in &self.props {
            prop.write_text(escaper, &mut out);
        }

        for component in &self.components {
            component.write_text(escaper, &mut out);
        }

        write_text_line(&mut out, "END", "VCALENDAR");

        // NOTE: Every piece is text and every escape and fold keeps it so,
        // the lossy walk being there only to keep this total.
        match text(&out) {
            Some(text) => f.write_str(text),
            None => f.write_str(&String::from_utf8_lossy(&out)),
        }
    }
}

/// Append a line of a raw text value, as [`IcalLine::text`] builds it and it
/// serializes.
fn write_text_line(out: &mut Vec<u8>, name: &str, value: &str) {
    let start = out.len();

    out.extend_from_slice(name.as_bytes());
    out.push(b':');
    out.extend_from_slice(value.as_bytes());

    end_line(out, start);
}

/// End the encoded line written in `out` from `start` on, folded as a line
/// with no recorded shape and broken with `\r\n`.
fn end_line(out: &mut Vec<u8>, start: usize) {
    fold_long(out, start, true);
    out.extend_from_slice(b"\r\n");
}

/// Encode `props`, then `components`, onto `items`, each into a slot it
/// lands in: a line or a component encoded aside and moved in waits on its
/// stores.
fn encode_items(
    props: &[IcalProp<'_>],
    components: &[IcalComponent<'_>],
    escaper: Escaper,
    items: &mut Vec<IcalItem<'static>>,
) {
    for prop in props {
        prop.encode_into(escaper, push_line(items));
    }

    for component in components {
        let mut child = Box::new(const { IcalCst::bare(Vec::new()) });
        component.encode_into(escaper, &mut child);
        items.push(IcalItem::Component(child));
    }
}

/// Where a value's wire shape goes: the leaves of a syntax node, as
/// [`Codec::encode`](crate::tree::codec::Codec::encode) builds it, or the
/// bytes that node serializes to.
///
/// A value type spells its shape once, in its `emit`, as one
/// [`uri`](Self::uri) or [`raw`](Self::raw) value or a run of
/// [`component`](Self::component)s, so the two ways out cannot drift apart.
pub(crate) struct IcalValueOut<'o> {
    /// The escaping mode.
    pub(crate) escaper: Escaper,
    /// What the shape is written into.
    sink: Sink<'o>,
}

impl IcalValueOut<'_> {
    /// Build a value's node from what `emit` writes.
    pub(crate) fn node(
        escaper: Escaper,
        emit: impl FnOnce(&mut IcalValueOut<'_>),
    ) -> IcalValueNode<'static> {
        let mut node = IcalValueNode::from_components(Vec::new(), escaper);

        emit(&mut IcalValueOut {
            escaper,
            sink: Sink::Node(&mut node),
        });

        node
    }

    /// Write one URI exactly as given, with no text escaping at all, as
    /// [`verbatim_node`] owns one.
    pub(crate) fn uri(&mut self, value: &str) {
        match &mut self.sink {
            Sink::Node(node) => **node = verbatim_node(value.as_bytes(), self.escaper),
            Sink::Bytes { out, .. } => percent_into(value.as_bytes(), out),
        }
    }

    /// Write one value byte for byte, with no escaping of any kind.
    pub(crate) fn raw(&mut self, value: &str) {
        match &mut self.sink {
            Sink::Node(node) => {
                **node = IcalValueNode::from_raw(value.as_bytes().to_vec(), self.escaper);
            }
            Sink::Bytes { out, .. } => out.extend_from_slice(value.as_bytes()),
        }
    }

    /// Write one `;`-separated component of `,`-separated values, each
    /// escaped by the mode.
    pub(crate) fn component<S: AsRef<str>>(&mut self, values: &[S]) {
        match &mut self.sink {
            Sink::Node(node) => node.set_component(node.component_count(), values),
            Sink::Bytes { out, started } => {
                if mem::replace(started, true) {
                    out.push(b';');
                }

                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        out.push(b',');
                    }

                    escape_into(value.as_ref().as_bytes(), self.escaper, out);
                }
            }
        }
    }
}

/// What an [`IcalValueOut`] writes into.
enum Sink<'o> {
    /// A node, its components pushed one by one, or replaced whole by a URI
    /// or a raw value.
    Node(&'o mut IcalValueNode<'static>),
    /// Bytes, `started` once a component is written, so the next one goes
    /// after a `;`.
    Bytes { out: &'o mut Vec<u8>, started: bool },
}

/// Own one URI exactly as given, with no text escaping at all.
///
/// A URI is not text: RFC 5545 section 3.3.13 gives it no escapes, so escaping
/// its `;` or `,` on the way out would rewrite the reference the value is, and
/// a value that decoded whole would not survive its own round trip.
///
/// Two exceptions go out percent-encoded, the only way RFC 3986 section 2.1
/// lets a URI carry either: a line break, which would end the line the value
/// sits on, and a backslash, which a reader takes for the text escape a
/// producer in the wild put there, so the value would not read back.
pub(crate) fn verbatim_node(value: &[u8], escaper: Escaper) -> IcalValueNode<'static> {
    let mut bytes = Vec::with_capacity(value.len());
    percent_into(value, &mut bytes);
    IcalValueNode::from_raw(bytes, escaper)
}

/// Append a URI to `out`, its line breaks and backslashes percent-encoded, as
/// [`verbatim_node`] writes it.
fn percent_into(value: &[u8], out: &mut Vec<u8>) {
    let mut rest = value;

    while let Some(at) = position(rest, |byte| {
        (byte == b'\r') | (byte == b'\n') | (byte == b'\\')
    }) {
        out.extend_from_slice(&rest[..at]);
        out.extend_from_slice(match rest[at] {
            b'\r' => b"%0D",
            b'\n' => b"%0A",
            _ => b"%5C",
        });
        rest = &rest[at + 1..];
    }

    out.extend_from_slice(rest);
}

/// Escape and own a clean value list into one component, by escaping mode.
pub(crate) fn encode_component<S: AsRef<str>>(
    values: &[S],
    escaper: Escaper,
) -> Vec<IcalValueLeaf<'static>> {
    values
        .iter()
        .map(|v| IcalValueLeaf::from(escape_with(v.as_ref().as_bytes(), escaper).into_owned()))
        .collect()
}

/// Escape and own raw value bytes into one component, by escaping mode.
///
/// The foreign-charset escape hatch: only the structural separators are
/// escaped, every other byte going out exactly as given.
pub(crate) fn encode_bytes_component<B: AsRef<[u8]>>(
    values: &[B],
    escaper: Escaper,
) -> Vec<IcalValueLeaf<'static>> {
    values
        .iter()
        .map(|v| IcalValueLeaf::from(escape_with(v.as_ref(), escaper).into_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString};

    use crate::{
        param::IcalParam,
        tree::{
            codec::{Codec, mode::Escaper},
            cst::IcalCst,
        },
        value::{cal_address::IcalCalAddress, text::IcalText, uri::IcalUri},
    };

    #[test]
    fn encodes_a_text_value_escaping_it() {
        let node = IcalText(Cow::Borrowed("hi, there")).encode(Escaper::Modern);
        assert_eq!(node.to_string(), r"hi\, there");
    }

    #[test]
    fn round_trips_a_decoded_calendar_back_to_bytes() {
        let input = concat!(
            "BEGIN:VCALENDAR\r\n",
            "VERSION:2.0\r\n",
            "PRODID:-//x//EN\r\n",
            "BEGIN:VEVENT\r\n",
            "UID:1\r\n",
            "DTSTAMP:20260101T000000Z\r\n",
            "SUMMARY:Lunch\r\n",
            "END:VEVENT\r\n",
            "END:VCALENDAR\r\n",
        );
        let cst = IcalCst::parse(input).unwrap();
        let cal = cst.decode();
        assert_eq!(cal.to_string(), input);
    }

    #[test]
    fn encodes_the_rfc_6868_parameter_sequences() {
        // NOTE: RFC 6868 section 3.1 read backwards, over the three characters
        // a parameter value cannot carry raw.
        let param = IcalParam::Cn(Cow::Borrowed("a\nb^c\"d"));

        assert_eq!(param.encode(Escaper::Modern).to_string(), "CN=a^nb^^c^'d",);
    }

    /// The decoded model holds a parameter's content, its RFC 5545 section 3.1
    /// delimiters excluded, so the pair is put back around a value carrying a
    /// character a bare `paramtext` may not hold.
    #[test]
    fn quotes_a_parameter_value_carrying_a_delimiter() {
        let param = IcalParam::AltRep(Cow::Borrowed("cid:part1.0001@example.org"));

        assert_eq!(
            param.encode(Escaper::Modern).to_string(),
            "ALTREP=\"cid:part1.0001@example.org\"",
        );
    }

    #[test]
    fn round_trips_a_parameter_byte_for_byte() {
        let input = concat!(
            "BEGIN:VCALENDAR\r\n",
            "VERSION:2.0\r\n",
            "BEGIN:VEVENT\r\n",
            "UID:1\r\n",
            "DTSTAMP:20260101T000000Z\r\n",
            "SUMMARY;LANGUAGE=en;ALTREP=\"cid:part1.0001@example.org\":Lunch\r\n",
            "COMMENT;X-PATH=\"C:\\temp\";X-NOTE=a^nb^^c^'d:Bring it\r\n",
            "END:VEVENT\r\n",
            "END:VCALENDAR\r\n",
        );
        let cst = IcalCst::parse(input).unwrap();

        assert_eq!(cst.decode().to_string(), input);
    }

    /// A calendar user address is a URI (RFC 5545 3.3.3), and RFC 5545 3.3.13
    /// gives a URI no escapes, so text escaping one rewrote the address.
    #[test]
    fn encodes_a_calendar_address_as_a_uri() {
        let address = r"mailto:a,b;c\d@example.com";
        let node = IcalCalAddress(Cow::Borrowed(address)).encode(Escaper::Modern);

        // NOTE: The backslash alone is percent-encoded, so the address reads
        // back as the same reference rather than losing it to the unescape.
        assert_eq!(node.to_string(), "mailto:a,b;c%5Cd@example.com");
        assert_eq!(
            IcalCalAddress::decode(&node).0,
            "mailto:a,b;c%5Cd@example.com"
        );
    }

    #[test]
    fn round_trips_a_calendar_address_through_the_model() {
        let input = concat!(
            "BEGIN:VCALENDAR\r\n",
            "VERSION:2.0\r\n",
            "BEGIN:VEVENT\r\n",
            "ORGANIZER;CN=Ada:mailto:ada,lovelace;x@example.com\r\n",
            "ATTENDEE:http://example.com/a;b,c\r\n",
            "END:VEVENT\r\n",
            "END:VCALENDAR\r\n",
        );
        let cst = IcalCst::parse(input).unwrap();

        assert_eq!(cst.decode().to_string(), input);
    }

    #[test]
    fn percent_encodes_a_line_break_written_into_a_uri() {
        // NOTE: Raw, the break would end the line the value sits on, and a URI
        // has no escape but RFC 3986 2.1 percent-encoding to carry one.
        let address = IcalCalAddress(Cow::Borrowed("mailto:a\r\nb@example.com"));
        let uri = IcalUri(Cow::Borrowed("https://example.com/a\nb"));

        assert_eq!(
            address.encode(Escaper::Modern).to_string(),
            "mailto:a%0D%0Ab@example.com",
        );
        assert_eq!(
            uri.encode(Escaper::Modern).to_string(),
            "https://example.com/a%0Ab",
        );
    }
}
