//! # Decode (syntax to model)
//!
//! The read side of the structural bridge: project a raw syntax tree onto the
//! decoded model.
//!
//! A [`IcalValueNode`] decodes its components, a [`IcalParamNode`] decodes
//! into an [`IcalParam`], an [`IcalLine`] decodes into an [`IcalProp`], and an
//! [`IcalCst`] decodes into a whole [`Ical`], recursively, walking every
//! nested component.
//!
//! A property's value kind is resolved through its spec, not a name match:
//! [`IcalLine::decode`] maps the name to an [`IcalPropKind`], asks the spec
//! for the in-force value kind (version plus any declared `VALUE`), then
//! routes to that kind's decoder.
//!
//! The parameter name dispatch is the match in [`IcalParamNode::decode`].

use core::mem;

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    component::{IcalComponent, IcalComponentName},
    ical::Ical,
    param::{IcalParam, IcalParamKind},
    prop::{IcalProp, IcalPropKind, IcalPropName, spec::prop_spec},
    tree::{
        codec::{
            Codec,
            unescape::{has_carets, param_text, unescape_param},
        },
        cst::{IcalCst, IcalItem},
        line::IcalLine,
        param::node::IcalParamNode,
        value::node::IcalValueNode,
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
        let props = self.prop_count();
        let mut cal = Ical {
            version,
            props: Vec::with_capacity(props),
            components: Vec::with_capacity(self.items.len() - props),
        };

        // NOTE: Every property and component is decoded into a slot its list
        // grows by, a constant empty one: one decoded aside and moved in
        // waits on the stores of every part it was built from.
        for item in &self.items {
            match item {
                IcalItem::Prop(line) if line.name.get().eq_ignore_ascii_case("VERSION") => {}
                IcalItem::Prop(line) => {
                    cal.props.resize_with(cal.props.len() + 1, IcalProp::empty);
                    let last = cal.props.len() - 1;
                    line.decode_into(version, &mut cal.props[last]);
                }
                IcalItem::Component(child) => {
                    let components = &mut cal.components;
                    components.resize_with(components.len() + 1, IcalComponent::empty);
                    let last = components.len() - 1;
                    child.decode_into(version, &mut components[last]);
                }
                // NOTE: An opaque line carried no structure to decode.
                IcalItem::Opaque(_) => {}
            }
        }

        cal
    }

    /// Decode a nested component into `slot`, an
    /// [empty](IcalComponent::empty) component written over without being
    /// dropped, there being nothing in it to free.
    fn decode_into<'d>(&'d self, version: IcalVersion, slot: &mut IcalComponent<'d>) {
        let props = self.prop_count();

        mem::forget(mem::replace(
            &mut slot.name,
            IcalComponentName::from(self.component_name()),
        ));
        slot.props.reserve_exact(props);
        slot.components.reserve_exact(self.items.len() - props);

        for item in &self.items {
            match item {
                IcalItem::Prop(line) => {
                    slot.props
                        .resize_with(slot.props.len() + 1, IcalProp::empty);
                    let last = slot.props.len() - 1;
                    line.decode_into(version, &mut slot.props[last]);
                }
                IcalItem::Component(child) => {
                    let components = &mut slot.components;
                    components.resize_with(components.len() + 1, IcalComponent::empty);
                    let last = components.len() - 1;
                    child.decode_into(version, &mut components[last]);
                }
                // NOTE: An opaque line carried no structure to decode.
                IcalItem::Opaque(_) => {}
            }
        }
    }

    /// How many of the items are property lines, so each decoded list is
    /// sized once for what it will hold.
    fn prop_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item, IcalItem::Prop(_)))
            .count()
    }
}

impl IcalLine<'_> {
    /// Decode the line into a typed property. A known property dispatches its
    /// value through the spec (see `value_kind`); an unknown one keeps its
    /// raw components so it round-trips.
    pub fn decode(&self, version: IcalVersion) -> IcalProp<'_> {
        let mut prop = IcalProp::empty();
        self.decode_into(version, &mut prop);
        prop
    }

    /// [`decode`](Self::decode) the line into `slot`, an
    /// [empty](IcalProp::empty) property whose fields are written over without
    /// being dropped, there being nothing in them to free.
    fn decode_into<'s>(&'s self, version: IcalVersion, slot: &mut IcalProp<'s>) {
        let name = self.name.get();
        let prop = IcalPropKind::from_name(name);

        mem::forget(mem::replace(
            &mut slot.name,
            match prop {
                Some(kind) => IcalPropName::Kind(kind),
                None => IcalPropName::Unknown(Cow::Borrowed(name)),
            },
        ));

        // NOTE: Sized first: an empty list grown by one reserves room for
        // four.
        slot.params.reserve_exact(self.params.len());

        for param in &self.params {
            slot.params
                .resize_with(slot.params.len() + 1, IcalParam::empty);
            let last = slot.params.len() - 1;
            param.decode_into(&mut slot.params[last]);
        }

        match self.value_kind_of(prop, version) {
            Some(kind) => decode_value_kind_into(kind, &self.value, &mut slot.value),
            None => put(
                &mut slot.value,
                IcalValue::Unknown(IcalUnknownValue::decode(&self.value)),
            ),
        }
    }

    /// The value kind the line's value is read and written as: the one its
    /// spec puts in force for a known name, from the calendar version and any
    /// declared `VALUE`, and the declared one alone for any other name.
    ///
    /// None for an undeclared extension, whose value stays raw.
    pub(crate) fn value_kind(&self, version: IcalVersion) -> Option<IcalValueKind> {
        self.value_kind_of(IcalPropKind::from_name(self.name.get()), version)
    }

    /// The [`value_kind`](Self::value_kind) of the line, its name already
    /// looked up as `prop`.
    fn value_kind_of(
        &self,
        prop: Option<IcalPropKind>,
        version: IcalVersion,
    ) -> Option<IcalValueKind> {
        match prop {
            Some(prop) => Some((prop_spec(prop).value)(version, self.declared_value_kind())),
            // NOTE: A name outside the vocabulary has no spec to consult, but a
            // line that declares its own VALUE has said what to read it as
            // (RFC 5545 3.2.20), and that holds for an X- name as much as for a
            // registered one.
            None => self.declared_value_kind(),
        }
    }

    /// The value kind named by this line's `VALUE` parameter, if any.
    fn declared_value_kind(&self) -> Option<IcalValueKind> {
        self.params
            .iter()
            .find(|param| param.name.get().eq_ignore_ascii_case("VALUE"))
            .and_then(|param| param.values.first())
            .and_then(|value| IcalValueKind::from_name(value.get()))
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

/// Decode a value node as the given value kind into `slot`, a value holding
/// nothing, written over without being dropped, routing to that value type's
/// [`Codec`].
///
/// Each arm stores its own value: one value matched out of the arms and then
/// moved is copied through memory, which waits on the stores of its parts.
/// The kinds holding one text read it inline, once for all of them, so it is
/// stored from registers.
fn decode_value_kind_into<'v>(
    kind: IcalValueKind,
    node: &'v IcalValueNode<'_>,
    slot: &mut IcalValue<'v>,
) {
    match kind {
        IcalValueKind::DateTimeList => {
            put(
                slot,
                IcalValue::DateTimeList(IcalDateTimeList::decode(node)),
            );
        }
        IcalValueKind::Geo => put(slot, IcalValue::Geo(IcalGeo::decode(node))),
        IcalValueKind::RequestStatus => {
            put(
                slot,
                IcalValue::RequestStatus(IcalRequestStatus::decode(node)),
            );
        }
        IcalValueKind::TextList => put(slot, IcalValue::TextList(IcalTextList::decode(node))),
        IcalValueKind::Binary
        | IcalValueKind::Boolean
        | IcalValueKind::CalAddress
        | IcalValueKind::Date
        | IcalValueKind::DateTime
        | IcalValueKind::Duration
        | IcalValueKind::Float
        | IcalValueKind::Integer
        | IcalValueKind::Period
        | IcalValueKind::Recur
        | IcalValueKind::Text
        | IcalValueKind::Time
        | IcalValueKind::Uri
        | IcalValueKind::UtcOffset => match node.plain() {
            Some(text) => put(slot, text_value(kind, Cow::Borrowed(text))),
            None => decode_text_into(kind, node, slot),
        },
    }
}

/// [`decode_value_kind_into`] for a kind holding one text that is not
/// [plain](IcalValueNode::plain), out of line: a text from either arm is
/// merged into one, copied through memory.
#[inline(never)]
fn decode_text_into<'v>(
    kind: IcalValueKind,
    node: &'v IcalValueNode<'_>,
    slot: &mut IcalValue<'v>,
) {
    put(slot, text_value(kind, node.decode()));
}

/// The value of a kind holding one text, as its [`Codec`] decodes it.
#[inline(always)]
fn text_value(kind: IcalValueKind, text: Cow<'_, str>) -> IcalValue<'_> {
    match kind {
        IcalValueKind::Boolean => IcalValue::Boolean(IcalBoolean(text)),
        IcalValueKind::CalAddress => IcalValue::CalAddress(IcalCalAddress(text)),
        IcalValueKind::Date => IcalValue::Date(IcalDate(text)),
        IcalValueKind::DateTime => IcalValue::DateTime(IcalDateTime(text)),
        IcalValueKind::Duration => IcalValue::Duration(IcalDuration(text)),
        IcalValueKind::Float => IcalValue::Float(IcalFloat(text)),
        IcalValueKind::Integer => IcalValue::Integer(IcalInteger(text)),
        IcalValueKind::Period => IcalValue::Period(IcalPeriod(text)),
        IcalValueKind::Recur => IcalValue::Recur(IcalRecur(text)),
        IcalValueKind::Text => IcalValue::Text(IcalText(text)),
        IcalValueKind::Time => IcalValue::Time(IcalTime(text)),
        IcalValueKind::Uri => IcalValue::Uri(IcalUri(text)),
        IcalValueKind::UtcOffset => IcalValue::UtcOffset(IcalUtcOffset(text)),
        _ => IcalValue::Binary(IcalBinary::Base64(text)),
    }
}

/// Write `value` over `slot`, which holds nothing, without dropping it.
#[inline(always)]
fn put<T>(slot: &mut T, value: T) {
    mem::forget(mem::replace(slot, value));
}

impl IcalParamNode<'_> {
    /// Decode the parameter into a typed parameter, dispatching on the name.
    pub fn decode(&self) -> IcalParam<'_> {
        let mut param = IcalParam::empty();
        self.decode_into(&mut param);
        param
    }

    /// [`decode`](Self::decode) the parameter into `slot`, an
    /// [empty](IcalParam::empty) parameter written over without being
    /// dropped, inlined so a parameter list is filled where it lies.
    #[inline(always)]
    fn decode_into<'s>(&'s self, slot: &mut IcalParam<'s>) {
        let Some(kind) = IcalParamKind::from_name(self.name.get()) else {
            // NOTE: a parameter name is a token (RFC 5545 3.2), with no
            // encoding of any kind to resolve.
            let name = Cow::Borrowed(self.name.get());
            return put(
                slot,
                IcalParam::Unknown {
                    name,
                    values: self.list(),
                },
            );
        };

        // NOTE: A first value with no caret to resolve is borrowed, and one
        // that holds a caret is decoded out of line: a value from either arm
        // is merged into one, copied through memory.
        let text = self
            .values
            .first()
            .map_or("", |value| param_text(value.get(), self.escaper));

        if has_carets(text, self.escaper) {
            return self.decode_carets_into(kind, slot);
        }

        self.typed_into(kind, Cow::Borrowed(text), slot);
    }

    /// [`decode_into`](Self::decode_into) a parameter whose first value
    /// holds a caret.
    #[inline(never)]
    fn decode_carets_into<'s>(&'s self, kind: IcalParamKind, slot: &mut IcalParam<'s>) {
        let text = self.values.first().map_or("", |value| value.get());
        self.typed_into(kind, unescape_param(text, self.escaper), slot);
    }

    /// Write the parameter of `kind` into `slot`: a list kind holds every
    /// value, a scalar one `value`, its first value decoded by the RFC 6868
    /// rules (empty when there is none). Each arm stores its own parameter,
    /// for the reason [`decode_value_kind_into`] gives.
    #[inline(always)]
    fn typed_into<'s>(
        &'s self,
        kind: IcalParamKind,
        value: Cow<'s, str>,
        slot: &mut IcalParam<'s>,
    ) {
        match kind {
            IcalParamKind::AltRep => put(slot, IcalParam::AltRep(value)),
            IcalParamKind::Cn => put(slot, IcalParam::Cn(value)),
            IcalParamKind::CuType => put(slot, IcalParam::CuType(value)),
            IcalParamKind::DelegatedFrom => put(slot, IcalParam::DelegatedFrom(self.list())),
            IcalParamKind::DelegatedTo => put(slot, IcalParam::DelegatedTo(self.list())),
            IcalParamKind::Dir => put(slot, IcalParam::Dir(value)),
            IcalParamKind::Encoding => put(slot, IcalParam::Encoding(value)),
            IcalParamKind::FmtType => put(slot, IcalParam::FmtType(value)),
            IcalParamKind::FbType => put(slot, IcalParam::FbType(value)),
            IcalParamKind::Language => put(slot, IcalParam::Language(value)),
            IcalParamKind::Member => put(slot, IcalParam::Member(self.list())),
            IcalParamKind::PartStat => put(slot, IcalParam::PartStat(value)),
            IcalParamKind::Range => put(slot, IcalParam::Range(value)),
            IcalParamKind::Related => put(slot, IcalParam::Related(value)),
            IcalParamKind::RelType => put(slot, IcalParam::RelType(value)),
            IcalParamKind::Role => put(slot, IcalParam::Role(value)),
            IcalParamKind::Rsvp => put(slot, IcalParam::Rsvp(value)),
            IcalParamKind::SentBy => put(slot, IcalParam::SentBy(value)),
            IcalParamKind::TzId => put(slot, IcalParam::TzId(value)),
            IcalParamKind::Value => put(slot, IcalParam::Value(value)),
            IcalParamKind::Display => put(slot, IcalParam::Display(value)),
            IcalParamKind::Email => put(slot, IcalParam::Email(value)),
            IcalParamKind::Feature => put(slot, IcalParam::Feature(self.list())),
            IcalParamKind::Label => put(slot, IcalParam::Label(value)),
            IcalParamKind::Order => put(slot, IcalParam::Order(value)),
            IcalParamKind::Schema => put(slot, IcalParam::Schema(value)),
            IcalParamKind::Derived => put(slot, IcalParam::Derived(value)),
            IcalParamKind::ScheduleAgent => put(slot, IcalParam::ScheduleAgent(value)),
            IcalParamKind::ScheduleForceSend => put(slot, IcalParam::ScheduleForceSend(value)),
            IcalParamKind::ScheduleStatus => put(slot, IcalParam::ScheduleStatus(value)),
            IcalParamKind::LinkRel => put(slot, IcalParam::LinkRel(value)),
            IcalParamKind::Gap => put(slot, IcalParam::Gap(value)),
            IcalParamKind::Charset => put(slot, IcalParam::Charset(value)),
        }
    }

    /// The parameter's values, decoded by the RFC 6868 rules, each pushed by
    /// its own arm for the reason [`decode`](Self::decode) gives.
    fn list(&self) -> Vec<Cow<'_, str>> {
        let mut values = Vec::with_capacity(self.values.len());

        for value in &self.values {
            let text = param_text(value.get(), self.escaper);

            if has_carets(text, self.escaper) {
                values.push(unescape_param(value.get(), self.escaper));
            } else {
                values.push(Cow::Borrowed(text));
            }
        }

        values
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
    use alloc::{borrow::Cow, vec};

    use crate::{
        param::IcalParam,
        tree::{
            codec::Codec, cst::IcalCst, param::node::IcalParamNode, value::node::IcalValueNode,
        },
        value::{
            IcalValue,
            binary::IcalBinary,
            boolean::IcalBoolean,
            cal_address::IcalCalAddress,
            datetime::{IcalDate, IcalDateTime, IcalDateTimeList, IcalTime},
            duration::IcalDuration,
            float::IcalFloat,
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

        assert_eq!(cal.version, IcalVersion::V2_0);
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

    /// A kind with no `;`-structure of its own is decoded whole.
    ///
    /// RFC 5545 3.3.11 has a text value escape a semicolon it means literally
    /// and 3.3.13 gives a URI no structure, so an unescaped `;` is content in
    /// either, and reading component by component dropped everything past it.
    #[test]
    fn decodes_every_unstructured_kind_whole() {
        let node = IcalValueNode::parse(b"a;b,c");

        assert_eq!(IcalText::decode(&node).0, "a;b,c");
        assert_eq!(IcalUri::decode(&node).0, "a;b,c");
        assert_eq!(IcalCalAddress::decode(&node).0, "a;b,c");
        assert_eq!(IcalPeriod::decode(&node).0, "a;b,c");
        assert_eq!(IcalRecur::decode(&node).0, "a;b,c");
        assert_eq!(
            IcalBinary::decode(&node),
            IcalBinary::Base64(Cow::Borrowed("a;b,c")),
        );
        assert_eq!(IcalBoolean::decode(&node).0, "a;b,c");
        assert_eq!(IcalDate::decode(&node).0, "a;b,c");
        assert_eq!(IcalDateTime::decode(&node).0, "a;b,c");
        assert_eq!(IcalTime::decode(&node).0, "a;b,c");
        assert_eq!(IcalDuration::decode(&node).0, "a;b,c");
        assert_eq!(IcalFloat::decode(&node).0, "a;b,c");
        assert_eq!(IcalInteger::decode(&node).0, "a;b,c");
        assert_eq!(IcalUtcOffset::decode(&node).0, "a;b,c");

        // NOTE: A list value owns its commas and nothing else, so only they
        // separate.
        assert_eq!(
            IcalTextList::decode(&node).0,
            vec![Cow::Borrowed("a;b"), Cow::Borrowed("c")],
        );
        assert_eq!(
            IcalDateTimeList::decode(&node).0,
            vec![Cow::Borrowed("a;b"), Cow::Borrowed("c")],
        );
    }

    /// A structured value's component keeps the commas inside it.
    ///
    /// A `REQUEST-STATUS` description is a text, where a comma separates
    /// nothing, so reading only a component's first comma-piece truncated the
    /// status a caller reads.
    #[test]
    fn decodes_a_structured_component_past_its_first_comma() {
        let node = IcalValueNode::parse(br"2.0;Success\, welcome;rcpt,two");
        let status = IcalRequestStatus::decode(&node);

        assert_eq!(status.code, "2.0");
        assert_eq!(status.description, "Success, welcome");
        assert_eq!(status.extra, "rcpt,two");
    }

    #[test]
    fn decodes_the_rfc_6868_parameter_sequences() {
        // NOTE: RFC 6868 section 3.1 spells the three characters a parameter
        // value cannot carry raw.
        let node = IcalParamNode::parse("CN=a^nb^^c^'d");

        assert_eq!(node.decode(), IcalParam::Cn(Cow::Borrowed("a\nb^c\"d")));
    }

    #[test]
    fn keeps_an_unknown_caret_sequence_in_a_parameter() {
        // NOTE: RFC 6868 section 3.1 forbids reading any other caret sequence
        // as an error, so the caret and what follows stay literal, and so does
        // a trailing one.
        let node = IcalParamNode::parse("CN=a^xb^");

        assert_eq!(node.decode(), IcalParam::Cn(Cow::Borrowed("a^xb^")));
    }

    #[test]
    fn keeps_a_backslash_in_a_parameter() {
        // NOTE: RFC 6868 section 3.2 forbids backslash escaping in a parameter
        // value, so a Windows path keeps its separators.
        let node = IcalParamNode::parse(r"X-PATH=C:\temp\note.txt");

        assert_eq!(
            node.decode(),
            IcalParam::Unknown {
                name: Cow::Borrowed("X-PATH"),
                values: vec![Cow::Borrowed(r"C:\temp\note.txt")],
            },
        );
    }
}
