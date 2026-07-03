//! # Encode (model to syntax)
//!
//! The write side of the structural bridge: project the decoded model onto a
//! raw syntax tree. A value's [`Codec`] impl encodes it into a
//! [`IcalValueNode`], a [`IcalParam`] encodes into a [`IcalParamNode`], a
//! [`IcalProp`] encodes into a [`IcalLine`], a [`IcalComponent`] encodes into a
//! nested [`IcalCst`], and a [`Ical`] encodes into the whole `VCALENDAR`
//! [`IcalCst`] (recursively). The whole calendar is encoded for its version's
//! [`Escaper`], which the value codecs use to escape every leaf.
//! [`Display`](core::fmt::Display) for [`Ical`] renders a decoded calendar
//! straight to its serialized bytes through here.

use core::fmt;

use alloc::{borrow::Cow, string::ToString, vec, vec::Vec};

use crate::{
    component::IcalComponent,
    ical::Ical,
    param::IcalParam,
    prop::IcalProp,
    tree::{
        codec::{Codec, escape::escape_with, mode::Escaper},
        cst::{IcalCst, IcalItem},
        leaf::{IcalLeaf, IcalValueLeaf},
        line::IcalLine,
        param::IcalParamNode,
        value::IcalValueNode,
    },
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
                .map(|component| IcalItem::Component(component.encode(escaper))),
        );

        IcalCst {
            begin: Some(IcalLine::text("BEGIN", "VCALENDAR")),
            items,
            end: Some(IcalLine::text("END", "VCALENDAR")),
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
                .map(|component| IcalItem::Component(component.encode(escaper))),
        );

        IcalCst {
            begin: Some(IcalLine::text("BEGIN", name.clone())),
            items,
            end: Some(IcalLine::text("END", name)),
        }
    }
}

impl<'a> From<Ical<'a>> for IcalCst<'static> {
    fn from(cal: Ical<'a>) -> Self {
        cal.encode()
    }
}

impl IcalProp<'_> {
    /// Encode the property into a raw content line for the given escaping mode,
    /// dispatching on its value.
    pub fn encode(&self, escaper: Escaper) -> IcalLine<'static> {
        IcalLine {
            name: IcalLeaf::from(self.name.to_string()),
            params: self.params.iter().map(IcalParam::encode).collect(),
            value: self.value.encode(escaper),
            eol: IcalLeaf::from("\r\n".to_string()),
        }
    }
}

impl IcalParam<'_> {
    /// Encode the parameter into a raw parameter node, dispatching on its kind.
    pub fn encode(&self) -> IcalParamNode<'static> {
        use crate::param::IcalParamKind::*;

        match self {
            IcalParam::AltRep(v) => param_scalar(&AltRep, v),
            IcalParam::Cn(v) => param_scalar(&Cn, v),
            IcalParam::CuType(v) => param_scalar(&CuType, v),
            IcalParam::DelegatedFrom(vs) => param_list(&DelegatedFrom, vs),
            IcalParam::DelegatedTo(vs) => param_list(&DelegatedTo, vs),
            IcalParam::Dir(v) => param_scalar(&Dir, v),
            IcalParam::Encoding(v) => param_scalar(&Encoding, v),
            IcalParam::FmtType(v) => param_scalar(&FmtType, v),
            IcalParam::FbType(v) => param_scalar(&FbType, v),
            IcalParam::Language(v) => param_scalar(&Language, v),
            IcalParam::Member(vs) => param_list(&Member, vs),
            IcalParam::PartStat(v) => param_scalar(&PartStat, v),
            IcalParam::Range(v) => param_scalar(&Range, v),
            IcalParam::Related(v) => param_scalar(&Related, v),
            IcalParam::RelType(v) => param_scalar(&RelType, v),
            IcalParam::Role(v) => param_scalar(&Role, v),
            IcalParam::Rsvp(v) => param_scalar(&Rsvp, v),
            IcalParam::SentBy(v) => param_scalar(&SentBy, v),
            IcalParam::TzId(v) => param_scalar(&TzId, v),
            IcalParam::Value(v) => param_scalar(&Value, v),
            IcalParam::Display(v) => param_scalar(&Display, v),
            IcalParam::Email(v) => param_scalar(&Email, v),
            IcalParam::Feature(vs) => param_list(&Feature, vs),
            IcalParam::Label(v) => param_scalar(&Label, v),
            IcalParam::Order(v) => param_scalar(&Order, v),
            IcalParam::Schema(v) => param_scalar(&Schema, v),
            IcalParam::Derived(v) => param_scalar(&Derived, v),
            IcalParam::Charset(v) => param_scalar(&Charset, v),

            IcalParam::Unknown { name, values } => IcalParamNode {
                name: IcalLeaf::from(name.to_string()),
                values: values
                    .iter()
                    .map(|v| IcalLeaf::from(v.to_string()))
                    .collect(),
            },
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

/// A parameter node from a single value (parameter values are not escaped: the
/// wire form is quoted, not backslash-escaped).
fn param_scalar(name: &str, value: &str) -> IcalParamNode<'static> {
    IcalParamNode {
        name: IcalLeaf::from(name.to_string()),
        values: vec![IcalLeaf::from(value.to_string())],
    }
}

/// A parameter node from a value list (parameter values are not escaped).
fn param_list(name: &str, values: &[Cow<'_, str>]) -> IcalParamNode<'static> {
    IcalParamNode {
        name: IcalLeaf::from(name.to_string()),
        values: values
            .iter()
            .map(|v| IcalLeaf::from(v.to_string()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString};

    use crate::{
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
}
