//! # Import
//!
//! The jCal-to-model half: an RFC 7265 component, property, parameter and
//! value read back into the decoded model, borrowing the JSON tree's strings
//! where it can.
//!
//! In, anything is accepted: an unknown name, an unknown parameter and an
//! unrecognised type slot all survive, a non-string scalar is coerced to text,
//! a missing part is an empty one, and a component that is not well shaped
//! comes back empty rather than as an error.
//!
//! The type slot resolves through the same spec vtable as the wire decoder, so
//! a jCal and the calendar it was written from decode to the same model.

use alloc::{borrow::Cow, string::String, vec, vec::Vec};

use serde_json::Value;

use crate::{
    component::{IcalComponent, IcalComponentName},
    jcal::{
        datetime::{
            date_from_json, datetime_from_json, offset_from_json, period_from_json, time_from_json,
        },
        json::scalar_text,
        recur::recur_from_json,
    },
    param::{IcalParam, IcalParamKind},
    prop::{IcalProp, IcalPropName, spec::prop_spec},
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

impl<'a> IcalComponent<'a> {
    /// One component read back, liberally: anything that is not a well-shaped
    /// component comes back empty rather than as an error.
    pub(crate) fn from_jcal(jcal: &'a Value, version: IcalVersion) -> Self {
        let Some((name, props, components)) = split_component(jcal) else {
            return IcalComponent {
                name: IcalComponentName::from(Cow::Borrowed("")),
                props: Vec::new(),
                components: Vec::new(),
            };
        };

        IcalComponent {
            name: IcalComponentName::from(Cow::Borrowed(name)),
            props: props
                .iter()
                .map(|prop| IcalProp::from_jcal(prop, version))
                .collect(),
            components: components
                .iter()
                .map(|component| IcalComponent::from_jcal(component, version))
                .collect(),
        }
    }
}

impl<'a> IcalProp<'a> {
    /// One property read back.
    ///
    /// A list-kinded property returns as a list and an unknown slot leaves the
    /// value undecoded. The slot goes back into a `VALUE` parameter only where
    /// it says more than the default, inverting the move the export made.
    pub(crate) fn from_jcal(entry: &'a Value, version: IcalVersion) -> Self {
        let array = entry.as_array().map(Vec::as_slice).unwrap_or(&[]);
        let name = array.first().and_then(Value::as_str).unwrap_or("");
        let params = array.get(1).and_then(Value::as_object);
        let slot = array.get(2).and_then(Value::as_str).unwrap_or("");
        let values = array.get(3..).unwrap_or(&[]);

        let mut decoded: Vec<IcalParam<'a>> = params
            .into_iter()
            .flat_map(|params| {
                params
                    .iter()
                    .map(|(name, value)| IcalParam::from_jcal(name, value))
            })
            .collect();

        let declared = slot.parse::<IcalValueKind>().ok();
        let name = IcalPropName::from(Cow::Borrowed(name));
        let spec = match name {
            IcalPropName::Kind(kind) => Some(prop_spec(kind)),
            IcalPropName::Unknown(_) => None,
        };

        let kind = match &spec {
            Some(spec) => Some((spec.value)(version, declared)),
            None => declared,
        };

        // NOTE: The slot survives as a VALUE parameter whenever it says more
        // than the property's default would, so `VALUE=PERIOD` and
        // `VALUE=BINARY` come back rather than dissolving into the type they
        // named.
        let default = spec
            .as_ref()
            .and_then(|spec| (spec.allowed_values)(version).first().copied())
            .map(default_slot)
            .unwrap_or(IcalValueKind::Text);
        let names_something = !slot.is_empty() && !slot.eq_ignore_ascii_case("unknown");

        if names_something && declared != Some(default) {
            // NOTE: Value kinds are case-insensitive tokens, and uppercase is
            // how the wire spells them.
            decoded.push(IcalParam::Value(Cow::Owned(slot.to_ascii_uppercase())));
        }

        IcalProp {
            name: uppercased(name),
            params: decoded,
            value: IcalValue::from_jcal(kind, values),
        }
    }
}

impl<'a> IcalParam<'a> {
    /// One parameter read back, by name.
    pub(crate) fn from_jcal(name: &'a str, value: &'a Value) -> Self {
        let values: Vec<Cow<'a, str>> = match value {
            Value::Array(items) => items.iter().map(scalar_text).collect(),
            other => vec![scalar_text(other)],
        };

        let Ok(kind) = name.parse::<IcalParamKind>() else {
            return IcalParam::Unknown {
                // NOTE: Lowercased by jCal, uppercase by iCalendar convention.
                name: Cow::Owned(name.to_ascii_uppercase()),
                values,
            };
        };

        let first = values.first().cloned().unwrap_or(Cow::Borrowed(""));

        match kind {
            IcalParamKind::AltRep => IcalParam::AltRep(first),
            IcalParamKind::Cn => IcalParam::Cn(first),
            IcalParamKind::CuType => IcalParam::CuType(first),
            IcalParamKind::DelegatedFrom => IcalParam::DelegatedFrom(values),
            IcalParamKind::DelegatedTo => IcalParam::DelegatedTo(values),
            IcalParamKind::Dir => IcalParam::Dir(first),
            IcalParamKind::Encoding => IcalParam::Encoding(first),
            IcalParamKind::FmtType => IcalParam::FmtType(first),
            IcalParamKind::FbType => IcalParam::FbType(first),
            IcalParamKind::Language => IcalParam::Language(first),
            IcalParamKind::Member => IcalParam::Member(values),
            IcalParamKind::PartStat => IcalParam::PartStat(first),
            IcalParamKind::Range => IcalParam::Range(first),
            IcalParamKind::Related => IcalParam::Related(first),
            IcalParamKind::RelType => IcalParam::RelType(first),
            IcalParamKind::Role => IcalParam::Role(first),
            IcalParamKind::Rsvp => IcalParam::Rsvp(first),
            IcalParamKind::SentBy => IcalParam::SentBy(first),
            IcalParamKind::TzId => IcalParam::TzId(first),
            IcalParamKind::Value => IcalParam::Value(first),
            IcalParamKind::Display => IcalParam::Display(first),
            IcalParamKind::Email => IcalParam::Email(first),
            IcalParamKind::Feature => IcalParam::Feature(values),
            IcalParamKind::Label => IcalParam::Label(first),
            IcalParamKind::Order => IcalParam::Order(first),
            IcalParamKind::Schema => IcalParam::Schema(first),
            IcalParamKind::Derived => IcalParam::Derived(first),
            IcalParamKind::ScheduleAgent => IcalParam::ScheduleAgent(first),
            IcalParamKind::ScheduleForceSend => IcalParam::ScheduleForceSend(first),
            IcalParamKind::ScheduleStatus => IcalParam::ScheduleStatus(first),
            IcalParamKind::LinkRel => IcalParam::LinkRel(first),
            IcalParamKind::Gap => IcalParam::Gap(first),
            IcalParamKind::Charset => IcalParam::Charset(first),
        }
    }
}

impl<'a> IcalValue<'a> {
    /// The value of a property, read back as the kind its type slot named.
    fn from_jcal(kind: Option<IcalValueKind>, values: &'a [Value]) -> Self {
        let first = values.first();
        let text = || first.map(scalar_text).unwrap_or(Cow::Borrowed(""));

        // NOTE: Keep the borrow when a JSON spelling needs no rewriting.
        fn rewritten(text: Cow<'_, str>, by: impl Fn(&str) -> Option<String>) -> Cow<'_, str> {
            match by(&text) {
                Some(rewritten) => Cow::Owned(rewritten),
                None => text,
            }
        }

        let Some(kind) = kind else {
            return IcalValue::Unknown(IcalUnknownValue::from_jcal(values));
        };

        match kind {
            // NOTE: RFC 7265 3.5.1 spells a binary value as base64 text; the
            // URI form of ATTACH arrives through the `uri` type slot instead.
            IcalValueKind::Binary => IcalValue::Binary(IcalBinary::Base64(text())),
            IcalValueKind::Boolean => IcalValue::Boolean(IcalBoolean(match first {
                Some(Value::Bool(true)) => Cow::Borrowed("TRUE"),
                Some(Value::Bool(false)) => Cow::Borrowed("FALSE"),
                _ => text(),
            })),
            IcalValueKind::CalAddress => IcalValue::CalAddress(IcalCalAddress(text())),
            IcalValueKind::Date => IcalValue::Date(IcalDate(rewritten(text(), date_from_json))),
            IcalValueKind::DateTime => {
                IcalValue::DateTime(IcalDateTime(rewritten(text(), datetime_from_json)))
            }
            IcalValueKind::DateTimeList => IcalValue::DateTimeList(IcalDateTimeList(
                values
                    .iter()
                    .map(|value| rewritten(scalar_text(value), period_from_json))
                    .collect(),
            )),
            IcalValueKind::Duration => IcalValue::Duration(IcalDuration(text())),
            IcalValueKind::Float => IcalValue::Float(IcalFloat(text())),
            IcalValueKind::Geo => {
                let pair = first
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                IcalValue::Geo(IcalGeo {
                    latitude: pair.first().map(scalar_text).unwrap_or(Cow::Borrowed("")),
                    longitude: pair.get(1).map(scalar_text).unwrap_or(Cow::Borrowed("")),
                })
            }
            IcalValueKind::Integer => IcalValue::Integer(IcalInteger(text())),
            IcalValueKind::Period => {
                IcalValue::Period(IcalPeriod(rewritten(text(), period_from_json)))
            }
            IcalValueKind::Recur => IcalValue::Recur(IcalRecur(recur_from_json(first))),
            IcalValueKind::RequestStatus => {
                let parts = first
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                IcalValue::RequestStatus(IcalRequestStatus {
                    code: parts.first().map(scalar_text).unwrap_or(Cow::Borrowed("")),
                    description: parts.get(1).map(scalar_text).unwrap_or(Cow::Borrowed("")),
                    extra: parts.get(2).map(scalar_text).unwrap_or(Cow::Borrowed("")),
                })
            }
            IcalValueKind::Text => IcalValue::Text(IcalText(text())),
            IcalValueKind::TextList => {
                IcalValue::TextList(IcalTextList(values.iter().map(scalar_text).collect()))
            }
            IcalValueKind::Time => IcalValue::Time(IcalTime(rewritten(text(), time_from_json))),
            IcalValueKind::Uri => IcalValue::Uri(IcalUri(text())),
            IcalValueKind::UtcOffset => {
                IcalValue::UtcOffset(IcalUtcOffset(rewritten(text(), offset_from_json)))
            }
        }
    }
}

impl<'a> IcalUnknownValue<'a> {
    /// An undecoded value read back, mirroring what the export wrote.
    fn from_jcal(values: &'a [Value]) -> Self {
        let nested = values.iter().any(|value| value.is_array());

        let components = match nested {
            true => values
                .iter()
                .map(|value| match value.as_array() {
                    Some(items) => items.iter().map(scalar_text).collect(),
                    None => vec![scalar_text(value)],
                })
                .collect(),
            false => vec![values.iter().map(scalar_text).collect()],
        };

        IcalUnknownValue { components }
    }
}

/// Split a jCal component array into its three parts.
pub(crate) fn split_component(jcal: &Value) -> Option<(&str, &[Value], &[Value])> {
    let array = jcal.as_array()?;
    let name = array.first()?.as_str()?;
    let props = array.get(1).and_then(Value::as_array).map(Vec::as_slice);
    let components = array.get(2).and_then(Value::as_array).map(Vec::as_slice);

    Some((name, props.unwrap_or(&[]), components.unwrap_or(&[])))
}

/// Put an unknown name back in the case iCalendar writes it in.
///
/// jCal lowercases every name (RFC 7265 3.3), and a known name normalises to
/// its canonical spelling on its own. An unknown one has only the convention
/// to go on, and the convention is uppercase.
fn uppercased(name: IcalPropName<'_>) -> IcalPropName<'_> {
    match name {
        IcalPropName::Unknown(name) => IcalPropName::Unknown(Cow::Owned(name.to_ascii_uppercase())),
        known => known,
    }
}

/// The type slot a property with this default kind writes when nothing
/// declares otherwise, the inverse of the collapse the export does.
fn default_slot(kind: IcalValueKind) -> IcalValueKind {
    match kind {
        IcalValueKind::DateTimeList => IcalValueKind::DateTime,
        IcalValueKind::TextList => IcalValueKind::Text,
        other => other,
    }
}
