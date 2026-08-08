//! # Decode (syntax to model)
//!
//! The read side of the structural bridge: project a raw syntax tree onto the
//! decoded model. A [`IcalValueNode`] decodes its components, a
//! [`IcalParamNode`] decodes into a [`IcalParam`], a [`IcalLine`] decodes into
//! a [`IcalProp`], and a [`IcalCst`] decodes into a whole [`Ical`]
//! (recursively, walking every nested component).
//!
//! A property's value kind is resolved through its spec, not a name match:
//! [`IcalLine::decode`] maps the name to a [`IcalPropKind`], asks the spec for
//! the in-force value kind (version plus any declared `VALUE`), then routes to
//! that kind's decoder. The parameter name dispatch is the match in
//! [`IcalParamNode::decode`].

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    component::{IcalComponent, IcalComponentName},
    ical::Ical,
    param::{IcalParam, IcalParamKind},
    prop::{IcalProp, IcalPropKind, IcalPropName},
    tree::{
        codec::{Codec, unescape::unescape},
        cst::{IcalCst, IcalItem},
        line::IcalLine,
        param::IcalParamNode,
        prop::prop_spec,
        value::IcalValueNode,
    },
    value::{
        IcalUnknownValue, IcalValue, IcalValueKind,
        binary::IcalBinary,
        boolean::IcalBoolean,
        cal_address::IcalCalAddress,
        datetime::{IcalDate, IcalDateTime, IcalDateTimeList, IcalTime},
        duration::IcalDuration,
        float::IcalFloat,
        geo::IcalGeo,
        integer::IcalInteger,
        period::IcalPeriod,
        recur::IcalRecur,
        request_status::IcalRequestStatus,
        text::{IcalText, IcalTextList},
        uri::IcalUri,
        utc_offset::IcalUtcOffset,
    },
    version::IcalVersion,
};

impl IcalCst<'_> {
    /// Decode the whole calendar into the semantic [`Ical`] model. `VERSION` is
    /// held as the calendar's indicator, not as a free property.
    pub fn decode(&self) -> Ical<'_> {
        let version = self.version();
        let mut props = Vec::new();
        let mut components = Vec::new();

        for item in &self.items {
            match item {
                IcalItem::Prop(line) if line.name.get().eq_ignore_ascii_case("VERSION") => {}
                IcalItem::Prop(line) => props.push(line.decode(version)),
                IcalItem::Component(child) => components.push(child.decode_component(version)),
                // NOTE: An opaque line carried no structure to decode.
                IcalItem::Opaque(_) => {}
            }
        }

        Ical {
            version,
            props,
            components,
        }
    }

    /// Decode a nested component into the recursive [`IcalComponent`] model.
    fn decode_component(&self, version: IcalVersion) -> IcalComponent<'_> {
        let mut props = Vec::new();
        let mut components = Vec::new();

        for item in &self.items {
            match item {
                IcalItem::Prop(line) => props.push(line.decode(version)),
                IcalItem::Component(child) => components.push(child.decode_component(version)),
                // NOTE: An opaque line carried no structure to decode.
                IcalItem::Opaque(_) => {}
            }
        }

        IcalComponent {
            name: IcalComponentName::from(self.component_name()),
            props,
            components,
        }
    }
}

impl IcalLine<'_> {
    /// Decode the line into a typed property. A known property dispatches its
    /// value through the spec (see `decode_value`); an unknown one keeps its
    /// raw components so it round-trips.
    pub fn decode(&self, version: IcalVersion) -> IcalProp<'_> {
        let name = self.name.get();
        let params = self.params.iter().map(IcalParamNode::decode).collect();

        let value = match name.parse::<IcalPropKind>() {
            Ok(prop) => self.decode_value(prop, version),
            // NOTE: A name outside the vocabulary has no spec to consult, but a
            // line that declares its own VALUE has said what to read it as
            // (RFC 5545 3.2.20), and that holds for an X- name as much as for a
            // registered one.
            Err(_) => match self.declared_value_kind() {
                Some(kind) => decode_value_kind(kind, &self.value),
                None => IcalValue::Unknown(IcalUnknownValue::decode(&self.value)),
            },
        };

        IcalProp {
            name: IcalPropName::from(name),
            params,
            value,
        }
    }

    /// Decode a known property's value through its spec: resolve the in-force
    /// value kind from the calendar version and any declared `VALUE`, then run
    /// that kind's decoder over the value node.
    pub(crate) fn decode_value(&self, prop: IcalPropKind, version: IcalVersion) -> IcalValue<'_> {
        let declared = self.declared_value_kind();
        let kind = (prop_spec(prop).value)(version, declared);
        decode_value_kind(kind, &self.value)
    }

    /// The value kind named by this line's `VALUE` parameter, if any.
    fn declared_value_kind(&self) -> Option<IcalValueKind> {
        self.params
            .iter()
            .find(|param| matches!(param.name.get().parse(), Ok(IcalParamKind::Value)))
            .and_then(|param| param.values.first())
            .and_then(|value| value.get().parse::<IcalValueKind>().ok())
    }

    /// Whether the line declares the `QUOTED-PRINTABLE` encoding, as an
    /// `ENCODING=` parameter or a bare token (the 1.0 short form).
    #[cfg(feature = "quoted-printable")]
    pub(crate) fn is_quoted_printable(&self) -> bool {
        self.params.iter().any(param_is_quoted_printable)
    }

    /// The value of this line's `CHARSET` parameter, if any.
    #[cfg(feature = "encoding")]
    pub(crate) fn charset_label(&self) -> Option<&str> {
        self.params
            .iter()
            .find(|param| param.name.get().eq_ignore_ascii_case("CHARSET"))
            .and_then(|param| param.values.first())
            .map(|value| value.get())
    }
}

/// Decode a value node as the given value kind, routing to that value type's
/// [`Codec`].
fn decode_value_kind<'v>(kind: IcalValueKind, node: &'v IcalValueNode<'_>) -> IcalValue<'v> {
    match kind {
        IcalValueKind::Binary => IcalValue::Binary(IcalBinary::decode(node)),
        IcalValueKind::Boolean => IcalValue::Boolean(IcalBoolean::decode(node)),
        IcalValueKind::CalAddress => IcalValue::CalAddress(IcalCalAddress::decode(node)),
        IcalValueKind::Date => IcalValue::Date(IcalDate::decode(node)),
        IcalValueKind::DateTime => IcalValue::DateTime(IcalDateTime::decode(node)),
        IcalValueKind::DateTimeList => IcalValue::DateTimeList(IcalDateTimeList::decode(node)),
        IcalValueKind::Duration => IcalValue::Duration(IcalDuration::decode(node)),
        IcalValueKind::Float => IcalValue::Float(IcalFloat::decode(node)),
        IcalValueKind::Geo => IcalValue::Geo(IcalGeo::decode(node)),
        IcalValueKind::Integer => IcalValue::Integer(IcalInteger::decode(node)),
        IcalValueKind::Period => IcalValue::Period(IcalPeriod::decode(node)),
        IcalValueKind::Recur => IcalValue::Recur(IcalRecur::decode(node)),
        IcalValueKind::RequestStatus => IcalValue::RequestStatus(IcalRequestStatus::decode(node)),
        IcalValueKind::Text => IcalValue::Text(IcalText::decode(node)),
        IcalValueKind::TextList => IcalValue::TextList(IcalTextList::decode(node)),
        IcalValueKind::Time => IcalValue::Time(IcalTime::decode(node)),
        IcalValueKind::Uri => IcalValue::Uri(IcalUri::decode(node)),
        IcalValueKind::UtcOffset => IcalValue::UtcOffset(IcalUtcOffset::decode(node)),
    }
}

impl IcalParamNode<'_> {
    /// Decode the parameter into a typed parameter, dispatching on the name.
    pub fn decode(&self) -> IcalParam<'_> {
        let Ok(kind) = self.name.get().parse::<IcalParamKind>() else {
            return IcalParam::Unknown {
                name: unescape(self.name.get()),
                values: self.list(),
            };
        };

        match kind {
            IcalParamKind::AltRep => IcalParam::AltRep(self.scalar()),
            IcalParamKind::Cn => IcalParam::Cn(self.scalar()),
            IcalParamKind::CuType => IcalParam::CuType(self.scalar()),
            IcalParamKind::DelegatedFrom => IcalParam::DelegatedFrom(self.list()),
            IcalParamKind::DelegatedTo => IcalParam::DelegatedTo(self.list()),
            IcalParamKind::Dir => IcalParam::Dir(self.scalar()),
            IcalParamKind::Encoding => IcalParam::Encoding(self.scalar()),
            IcalParamKind::FmtType => IcalParam::FmtType(self.scalar()),
            IcalParamKind::FbType => IcalParam::FbType(self.scalar()),
            IcalParamKind::Language => IcalParam::Language(self.scalar()),
            IcalParamKind::Member => IcalParam::Member(self.list()),
            IcalParamKind::PartStat => IcalParam::PartStat(self.scalar()),
            IcalParamKind::Range => IcalParam::Range(self.scalar()),
            IcalParamKind::Related => IcalParam::Related(self.scalar()),
            IcalParamKind::RelType => IcalParam::RelType(self.scalar()),
            IcalParamKind::Role => IcalParam::Role(self.scalar()),
            IcalParamKind::Rsvp => IcalParam::Rsvp(self.scalar()),
            IcalParamKind::SentBy => IcalParam::SentBy(self.scalar()),
            IcalParamKind::TzId => IcalParam::TzId(self.scalar()),
            IcalParamKind::Value => IcalParam::Value(self.scalar()),
            IcalParamKind::Display => IcalParam::Display(self.scalar()),
            IcalParamKind::Email => IcalParam::Email(self.scalar()),
            IcalParamKind::Feature => IcalParam::Feature(self.list()),
            IcalParamKind::Label => IcalParam::Label(self.scalar()),
            IcalParamKind::Order => IcalParam::Order(self.scalar()),
            IcalParamKind::Schema => IcalParam::Schema(self.scalar()),
            IcalParamKind::Derived => IcalParam::Derived(self.scalar()),
            IcalParamKind::ScheduleAgent => IcalParam::ScheduleAgent(self.scalar()),
            IcalParamKind::ScheduleForceSend => IcalParam::ScheduleForceSend(self.scalar()),
            IcalParamKind::ScheduleStatus => IcalParam::ScheduleStatus(self.scalar()),
            IcalParamKind::LinkRel => IcalParam::LinkRel(self.scalar()),
            IcalParamKind::Gap => IcalParam::Gap(self.scalar()),
            IcalParamKind::Charset => IcalParam::Charset(self.scalar()),
        }
    }

    /// The parameter's first value, decoded (empty when there is none).
    fn scalar(&self) -> Cow<'_, str> {
        self.values
            .first()
            .map(|v| unescape(v.get()))
            .unwrap_or(Cow::Borrowed(""))
    }

    /// The parameter's values, decoded.
    fn list(&self) -> Vec<Cow<'_, str>> {
        self.values.iter().map(|v| unescape(v.get())).collect()
    }
}

/// Whether a parameter is `ENCODING=QUOTED-PRINTABLE` or the bare 1.0 token.
#[cfg(feature = "quoted-printable")]
fn param_is_quoted_printable(param: &IcalParamNode<'_>) -> bool {
    let name = param.name.get();

    (name.eq_ignore_ascii_case("ENCODING")
        && param
            .values
            .iter()
            .any(|v| v.get().eq_ignore_ascii_case("QUOTED-PRINTABLE")))
        || (param.values.is_empty() && name.eq_ignore_ascii_case("QUOTED-PRINTABLE"))
}

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use crate::{
        tree::cst::IcalCst,
        value::{IcalValue, datetime::IcalDateTime, text::IcalText},
    };

    #[test]
    fn decodes_a_calendar_with_a_nested_event() {
        let input = concat!(
            "BEGIN:VCALENDAR\r\n",
            "VERSION:2.0\r\n",
            "PRODID:-//x//EN\r\n",
            "BEGIN:VEVENT\r\n",
            "UID:1\r\n",
            "DTSTART:20260101T120000Z\r\n",
            "SUMMARY:Lunch\r\n",
            "END:VEVENT\r\n",
            "END:VCALENDAR\r\n",
        );
        let cst = IcalCst::parse(input).unwrap();
        let cal = cst.decode();

        // NOTE: VERSION is the indicator; PRODID is the only calendar-level
        // prop.
        assert_eq!(cal.version, crate::version::IcalVersion::V2_0);
        assert_eq!(cal.props.len(), 1);
        assert_eq!(&*cal.props[0].name, "PRODID");
        assert_eq!(cal.components.len(), 1);

        let event = &cal.components[0];
        assert_eq!(&*event.name, "VEVENT");
        assert_eq!(event.props.len(), 3);
        assert_eq!(
            event.props[1].value,
            IcalValue::DateTime(IcalDateTime(Cow::Borrowed("20260101T120000Z"))),
        );
        assert_eq!(
            event.props[2].value,
            IcalValue::Text(IcalText(Cow::Borrowed("Lunch"))),
        );
    }

    #[test]
    fn an_unknown_property_round_trips_as_unknown() {
        let input = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nX-WR-CALNAME:Work\r\nEND:VCALENDAR\r\n";
        let cst = IcalCst::parse(input).unwrap();
        let cal = cst.decode();
        assert!(matches!(cal.props[0].value, IcalValue::Unknown(_)));
        assert_eq!(&*cal.props[0].name, "X-WR-CALNAME");
    }
}
