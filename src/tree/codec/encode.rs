//! # Encode (model to syntax)
//!
//! The write side of the structural bridge: project the decoded model onto a
//! raw syntax tree.
//!
//! A value's [`Codec`] impl encodes it into a [`IcalValueNode`], an
//! [`IcalParam`] into an [`IcalParamNode`], a [`IcalProp`] into an
//! [`IcalLine`], an [`IcalComponent`] into a nested [`IcalCst`], and an
//! [`Ical`] into the whole `VCALENDAR` [`IcalCst`], recursively.
//!
//! The whole calendar is encoded for its version's [`Escaper`], which the
//! value codecs use to escape every leaf.
//!
//! [`Display`](core::fmt::Display) for [`Ical`] renders a decoded calendar
//! straight to its serialized bytes through here.

use core::fmt;

use alloc::{borrow::Cow, boxed::Box, string::ToString, vec, vec::Vec};

use crate::{
    component::IcalComponent,
    ical::Ical,
    param::IcalParam,
    prop::IcalProp,
    tree::{
        codec::{
            Codec,
            escape::{escape_param, escape_with},
            mode::Escaper,
        },
        cst::{IcalCst, IcalItem},
        leaf::{IcalLeaf, IcalValueLeaf},
        line::IcalLine,
        param::node::IcalParamNode,
        value::node::IcalValueNode,
        wire::IcalWire,
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
            self.version.to_string(),
        )));
        items.extend(
            self.props
                .iter()
                .map(|prop| IcalItem::Prop(prop.encode(escaper))),
        );
        items.extend(
            self.components
                .iter()
                .map(|component| IcalItem::Component(Box::new(component.encode(escaper)))),
        );

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
        let name = self.name.to_string();

        let mut items = Vec::with_capacity(self.props.len() + self.components.len());
        items.extend(
            self.props
                .iter()
                .map(|prop| IcalItem::Prop(prop.encode(escaper))),
        );
        items.extend(
            self.components
                .iter()
                .map(|component| IcalItem::Component(Box::new(component.encode(escaper)))),
        );

        IcalCst {
            begin: Some(IcalLine::text("BEGIN", name.clone())),
            items,
            end: Some(IcalLine::text("END", name)),
            trailing: Cow::Borrowed(""),
        }
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
        IcalLine {
            name: IcalLeaf::from(self.name.to_string()),
            params: self
                .params
                .iter()
                .map(|param| param.encode(escaper))
                .collect(),
            value: self.value.encode(escaper),
            eol: IcalLeaf::from("\r\n".to_string()),
            // NOTE: An encoded property has no wire history: it is written out
            // unfolded, in canonical form.
            wire: IcalWire::default(),
        }
    }
}

impl IcalParam<'_> {
    /// Encode the parameter into a raw parameter node for the given escaping
    /// mode, dispatching on its kind.
    pub fn encode(&self, escaper: Escaper) -> IcalParamNode<'static> {
        use crate::param::IcalParamKind::*;

        match self {
            IcalParam::AltRep(v) => param_scalar(&AltRep, v, escaper),
            IcalParam::Cn(v) => param_scalar(&Cn, v, escaper),
            IcalParam::CuType(v) => param_scalar(&CuType, v, escaper),
            IcalParam::DelegatedFrom(vs) => param_list(&DelegatedFrom, vs, escaper),
            IcalParam::DelegatedTo(vs) => param_list(&DelegatedTo, vs, escaper),
            IcalParam::Dir(v) => param_scalar(&Dir, v, escaper),
            IcalParam::Encoding(v) => param_scalar(&Encoding, v, escaper),
            IcalParam::FmtType(v) => param_scalar(&FmtType, v, escaper),
            IcalParam::FbType(v) => param_scalar(&FbType, v, escaper),
            IcalParam::Language(v) => param_scalar(&Language, v, escaper),
            IcalParam::Member(vs) => param_list(&Member, vs, escaper),
            IcalParam::PartStat(v) => param_scalar(&PartStat, v, escaper),
            IcalParam::Range(v) => param_scalar(&Range, v, escaper),
            IcalParam::Related(v) => param_scalar(&Related, v, escaper),
            IcalParam::RelType(v) => param_scalar(&RelType, v, escaper),
            IcalParam::Role(v) => param_scalar(&Role, v, escaper),
            IcalParam::Rsvp(v) => param_scalar(&Rsvp, v, escaper),
            IcalParam::SentBy(v) => param_scalar(&SentBy, v, escaper),
            IcalParam::TzId(v) => param_scalar(&TzId, v, escaper),
            IcalParam::Value(v) => param_scalar(&Value, v, escaper),
            IcalParam::Display(v) => param_scalar(&Display, v, escaper),
            IcalParam::Email(v) => param_scalar(&Email, v, escaper),
            IcalParam::Feature(vs) => param_list(&Feature, vs, escaper),
            IcalParam::Label(v) => param_scalar(&Label, v, escaper),
            IcalParam::Order(v) => param_scalar(&Order, v, escaper),
            IcalParam::Schema(v) => param_scalar(&Schema, v, escaper),
            IcalParam::Derived(v) => param_scalar(&Derived, v, escaper),
            IcalParam::ScheduleAgent(v) => param_scalar(&ScheduleAgent, v, escaper),
            IcalParam::ScheduleForceSend(v) => param_scalar(&ScheduleForceSend, v, escaper),
            IcalParam::ScheduleStatus(v) => param_scalar(&ScheduleStatus, v, escaper),
            IcalParam::LinkRel(v) => param_scalar(&LinkRel, v, escaper),
            IcalParam::Gap(v) => param_scalar(&Gap, v, escaper),
            IcalParam::Charset(v) => param_scalar(&Charset, v, escaper),

            IcalParam::Unknown { name, values } => param_list(name, values, escaper),
        }
    }
}

/// Serialize the decoded calendar by encoding it into a CST (canonical).
impl fmt::Display for Ical<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

/// A one-component, one-value syntax node, escaping the value by the given
/// mode.
pub(crate) fn scalar_node(value: &str, escaper: Escaper) -> IcalValueNode<'static> {
    IcalValueNode::from_components(vec![encode_component(&[value], escaper)], escaper)
}

/// Own one value exactly as given, with no escaping at all.
///
/// A URI is not text: RFC 5545 section 3.3.13 gives it no escapes, so escaping
/// its `;` or `,` on the way out would rewrite the reference the value is, and
/// a value that decoded whole would not survive its own round trip.
pub(crate) fn verbatim_node(value: &str, escaper: Escaper) -> IcalValueNode<'static> {
    IcalValueNode::from_raw(value.as_bytes().to_vec(), escaper)
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

/// A parameter node from a single value, encoded by the given mode's parameter
/// rules.
fn param_scalar(name: &str, value: &str, escaper: Escaper) -> IcalParamNode<'static> {
    IcalParamNode {
        name: IcalLeaf::from(name.to_string()),
        values: vec![IcalLeaf::from(escape_param(value, escaper).into_owned())],
        escaper,
    }
}

/// A parameter node from a value list, encoded by the given mode's parameter
/// rules.
fn param_list(name: &str, values: &[Cow<'_, str>], escaper: Escaper) -> IcalParamNode<'static> {
    IcalParamNode {
        name: IcalLeaf::from(name.to_string()),
        values: values
            .iter()
            .map(|v| IcalLeaf::from(escape_param(v, escaper).into_owned()))
            .collect(),
        escaper,
    }
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
        value::text::IcalText,
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
            "SUMMARY;LANGUAGE=en;ALTREP=\"cid:part1.0001@example.org\"",
            ";X-PATH=\"C:\\temp\";X-NOTE=a^nb^^c^'d:Lunch\r\n",
            "END:VEVENT\r\n",
            "END:VCALENDAR\r\n",
        );
        let cst = IcalCst::parse(input).unwrap();

        assert_eq!(cst.decode().to_string(), input);
    }
}
