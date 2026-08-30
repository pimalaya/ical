//! # jCal
//!
//! The RFC 7265 jCal codec: the decoded calendar as JSON, and back.
//!
//! jCal is the JSON spelling of the iCalendar model. A component is
//! `[name, [properties], [components]]` and a property is
//! `[name, {params}, type, value...]` (RFC 7265 3.3, 3.4).
//!
//! [`Ical::to_jcal`] writes the decoded model as a [`serde_json::Value`];
//! [`Ical::from_jcal`] reads one back, borrowing the JSON tree's strings.
//!
//! Import resolves each property's value kind through the same spec vtable as
//! the wire decoder, so a jCal and the calendar it was written from decode to
//! the same model.
//!
//! The boundary is a raw `Value`, not a serde implementation on any calendar
//! type. One model has two JSON spellings here, jCal and JSCalendar (behind
//! the `jscalendar` feature), and serde keys one representation per type, so
//! it is the wrong tool.
//!
//! A raw-value boundary also keeps the public API free of a serialization
//! commitment.
//!
//! ## Postel, again
//!
//! On the way out the RFC is followed: names are lowercased, the `VALUE`
//! parameter moves into the type slot (RFC 7265 3.5.4), and dates, times,
//! periods, offsets and recurrence rules are re-spelled in the JSON forms
//! (3.5.1 to 3.5.7).
//!
//! On the way in anything is accepted: an unknown name, an unknown parameter
//! and an unrecognised type slot all survive, a non-string scalar is coerced
//! to text, and a missing part is an empty one.
//!
//! ## What round-trips, and what normalises
//!
//! A calendar written to jCal and read back decodes to the same model, with
//! three normalisations that are the JSON format's, not the codec's.
//!
//! Parameter order is lost to the JSON object, a recurrence rule comes back
//! with its parts in the RFC's canonical order (a JSON object has no order to
//! preserve), and names come back in their canonical spelling.
//!
//! Byte fidelity is the syntax tree's job; jCal is a projection of the
//! decoded model.

use core::{error, fmt};

use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value, json};

use crate::{
    component::{IcalComponent, IcalComponentName},
    ical::Ical,
    param::{IcalParam, IcalParamKind},
    prop::{IcalProp, IcalPropName},
    tree::prop::prop_spec,
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

/// What a jCal value cannot be read as.
///
/// Only the shape of the document is refused; everything inside it is read
/// liberally, so this is a short list on purpose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalJcalError {
    /// The document is not a `[name, [properties], [components]]` array.
    NotAComponent,
    /// The outermost component is not a `vcalendar`.
    NotACalendar(String),
}

impl fmt::Display for IcalJcalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAComponent => {
                f.write_str("jCal value is not a [name, [properties], [components]] array")
            }
            Self::NotACalendar(name) => {
                write!(f, "jCal document is a `{name}`, not a `vcalendar`")
            }
        }
    }
}

impl error::Error for IcalJcalError {}

impl Ical<'_> {
    /// The calendar as an RFC 7265 jCal value.
    ///
    /// `VERSION` leads the property list, as it does on the wire, carrying the
    /// calendar's own version rather than a fixed one.
    pub fn to_jcal(&self) -> Value {
        let mut props = vec![json!([
            "version",
            Map::new(),
            "text",
            Value::String((*self.version).to_string())
        ])];
        props.extend(self.props.iter().map(prop_to_jcal));

        let components: Vec<Value> = self.components.iter().map(component_to_jcal).collect();

        json!(["vcalendar", props, components])
    }
}

impl<'a> Ical<'a> {
    /// Read a calendar back from an RFC 7265 jCal value, borrowing its strings.
    pub fn from_jcal(jcal: &'a Value) -> Result<Self, IcalJcalError> {
        let (name, props, components) =
            split_component(jcal).ok_or(IcalJcalError::NotAComponent)?;

        if !name.eq_ignore_ascii_case("vcalendar") {
            return Err(IcalJcalError::NotACalendar(name.to_string()));
        }

        let mut version = IcalVersion::V2_0;
        let mut decoded = Vec::new();

        for entry in props {
            let prop = prop_from_jcal(entry, version);

            // NOTE: VERSION is the hoisted-out indicator, never a property of
            // the model (see `Ical::props`).
            if prop.name.eq_ignore_ascii_case("VERSION") {
                if let IcalValue::Text(text) = &prop.value {
                    version = text.0.parse().unwrap_or(IcalVersion::V2_0);
                }
                continue;
            }

            decoded.push(prop);
        }

        Ok(Ical {
            version,
            props: decoded,
            components: components
                .iter()
                .map(|component| component_from_jcal(component, version))
                .collect(),
        })
    }
}

/// One component as `[name, [properties], [components]]`.
pub(crate) fn component_to_jcal(component: &IcalComponent<'_>) -> Value {
    let props: Vec<Value> = component.props.iter().map(prop_to_jcal).collect();
    let children: Vec<Value> = component.components.iter().map(component_to_jcal).collect();

    json!([component.name.to_lowercase(), props, children])
}

/// One component read back, liberally: anything that is not a well-shaped
/// component comes back empty rather than as an error.
pub(crate) fn component_from_jcal<'a>(jcal: &'a Value, version: IcalVersion) -> IcalComponent<'a> {
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
            .map(|prop| prop_from_jcal(prop, version))
            .collect(),
        components: components
            .iter()
            .map(|component| component_from_jcal(component, version))
            .collect(),
    }
}

/// Split a jCal component array into its three parts.
fn split_component(jcal: &Value) -> Option<(&str, &[Value], &[Value])> {
    let array = jcal.as_array()?;
    let name = array.first()?.as_str()?;
    let props = array.get(1).and_then(Value::as_array).map(Vec::as_slice);
    let components = array.get(2).and_then(Value::as_array).map(Vec::as_slice);

    Some((name, props.unwrap_or(&[]), components.unwrap_or(&[])))
}

/// One property as `[name, {params}, type, value...]`.
pub(crate) fn prop_to_jcal(prop: &IcalProp<'_>) -> Value {
    let mut params = Map::new();

    for param in &prop.params {
        // NOTE: The declared VALUE moves into the type slot (RFC 7265 3.5.4).
        if matches!(param.kind(), Some(IcalParamKind::Value)) {
            continue;
        }

        let (name, value) = param_to_jcal(param);
        params.insert(name, value);
    }

    let mut entry = vec![
        Value::String(prop.name.to_lowercase()),
        Value::Object(params),
        Value::String(type_slot(prop)),
    ];
    entry.extend(value_to_jcal(&prop.value));

    Value::Array(entry)
}

/// The type slot: what `VALUE` declared, else the value's kind, else `unknown`.
///
/// The declared kind wins as the finer answer: `date-time-list` is a
/// modelling detail rather than an RFC 7265 type, so `RDATE;VALUE=PERIOD`
/// goes out as `period`, and import resolves it back through the spec.
pub(crate) fn type_slot(prop: &IcalProp<'_>) -> String {
    let declared = prop.params.iter().find_map(|param| match param {
        IcalParam::Value(kind) => Some(kind.to_ascii_lowercase()),
        _ => None,
    });

    if let Some(declared) = declared {
        return declared;
    }

    match prop.value.kind() {
        // NOTE: A list is several values of one kind, and that kind is what
        // jCal names.
        Some(IcalValueKind::DateTimeList) => (*IcalValueKind::DateTime).to_ascii_lowercase(),
        Some(IcalValueKind::TextList) => (*IcalValueKind::Text).to_ascii_lowercase(),
        Some(kind) => (*kind).to_ascii_lowercase(),
        None => String::from("unknown"),
    }
}

/// One parameter as a `name: value` pair, a JSON array when it has several
/// values.
pub(crate) fn param_to_jcal(param: &IcalParam<'_>) -> (String, Value) {
    if let IcalParam::Unknown { name, values } = param {
        return (name.to_ascii_lowercase(), strings(values));
    }

    let name = param
        .kind()
        .map(|kind| (*kind).to_ascii_lowercase())
        .unwrap_or_default();

    let value = match param {
        IcalParam::DelegatedFrom(values)
        | IcalParam::DelegatedTo(values)
        | IcalParam::Member(values)
        | IcalParam::Feature(values) => strings(values),
        other => Value::String(param_scalar(other).into_owned()),
    };

    (name, value)
}

/// A list of strings as one JSON value: the string itself when there is one,
/// an array otherwise, which is what RFC 7265 3.5.2 asks for.
fn strings(values: &[Cow<'_, str>]) -> Value {
    match values {
        [one] => Value::String(one.to_string()),
        many => Value::Array(many.iter().map(|v| Value::String(v.to_string())).collect()),
    }
}

/// The text of a single-valued parameter.
pub(crate) fn param_scalar<'a>(param: &'a IcalParam<'_>) -> Cow<'a, str> {
    match param {
        IcalParam::AltRep(v)
        | IcalParam::Cn(v)
        | IcalParam::CuType(v)
        | IcalParam::Dir(v)
        | IcalParam::Encoding(v)
        | IcalParam::FmtType(v)
        | IcalParam::FbType(v)
        | IcalParam::Language(v)
        | IcalParam::PartStat(v)
        | IcalParam::Range(v)
        | IcalParam::Related(v)
        | IcalParam::RelType(v)
        | IcalParam::Role(v)
        | IcalParam::Rsvp(v)
        | IcalParam::SentBy(v)
        | IcalParam::TzId(v)
        | IcalParam::Value(v)
        | IcalParam::Display(v)
        | IcalParam::Email(v)
        | IcalParam::Label(v)
        | IcalParam::Order(v)
        | IcalParam::Schema(v)
        | IcalParam::Derived(v)
        | IcalParam::ScheduleAgent(v)
        | IcalParam::ScheduleForceSend(v)
        | IcalParam::ScheduleStatus(v)
        | IcalParam::LinkRel(v)
        | IcalParam::Gap(v)
        | IcalParam::Charset(v) => Cow::Borrowed(v.as_ref()),
        _ => Cow::Borrowed(""),
    }
}

/// One property read back.
///
/// The type slot resolves through the same spec vtable as the wire decoder,
/// so a list-kinded property returns as a list and an unknown slot leaves the
/// value undecoded. The slot goes back into a `VALUE` parameter only where it
/// says more than the default, inverting the move `to_jcal` made.
pub(crate) fn prop_from_jcal<'a>(entry: &'a Value, version: IcalVersion) -> IcalProp<'a> {
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
                .map(|(name, value)| param_from_jcal(name, value))
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

    // NOTE: The slot survives as a VALUE parameter whenever it says more than
    // the property's default would, so `VALUE=PERIOD` and `VALUE=BINARY` come
    // back rather than dissolving into the type they named.
    let default = spec
        .as_ref()
        .and_then(|spec| (spec.allowed_values)(version).first().copied())
        .map(default_slot)
        .unwrap_or(IcalValueKind::Text);
    let names_something = !slot.is_empty() && !slot.eq_ignore_ascii_case("unknown");

    if names_something && declared != Some(default) {
        // NOTE: Value kinds are case-insensitive tokens, and uppercase is how
        // the wire spells them.
        decoded.push(IcalParam::Value(Cow::Owned(slot.to_ascii_uppercase())));
    }

    IcalProp {
        name: uppercased(name),
        params: decoded,
        value: value_from_jcal(kind, values),
    }
}

/// Put an unknown name back in the case iCalendar writes it in.
///
/// jCal lowercases every name (RFC 7265 3.3), and a known name normalises to
/// its canonical spelling on its own. An unknown one has only the convention to
/// go on, and the convention is uppercase.
fn uppercased(name: IcalPropName<'_>) -> IcalPropName<'_> {
    match name {
        IcalPropName::Unknown(name) => IcalPropName::Unknown(Cow::Owned(name.to_ascii_uppercase())),
        known => known,
    }
}

/// The type slot a property with this default kind writes when nothing
/// declares otherwise, which is the inverse of the collapse [`type_slot`] does.
fn default_slot(kind: IcalValueKind) -> IcalValueKind {
    match kind {
        IcalValueKind::DateTimeList => IcalValueKind::DateTime,
        IcalValueKind::TextList => IcalValueKind::Text,
        other => other,
    }
}

/// One parameter read back, by name.
pub(crate) fn param_from_jcal<'a>(name: &'a str, value: &'a Value) -> IcalParam<'a> {
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

/// A JSON scalar as text: a string as itself, anything else through its JSON
/// spelling, so nothing is dropped for being the wrong type.
fn scalar_text(value: &Value) -> Cow<'_, str> {
    match value {
        Value::String(text) => Cow::Borrowed(text),
        Value::Null => Cow::Borrowed(""),
        other => Cow::Owned(other.to_string()),
    }
}

/// The value slots of a property: one per value, several for a list.
fn value_to_jcal(value: &IcalValue<'_>) -> Vec<Value> {
    match value {
        IcalValue::Binary(IcalBinary::Uri(v) | IcalBinary::Base64(v)) => {
            vec![Value::String(v.to_string())]
        }
        IcalValue::Boolean(v) => vec![match v.0.eq_ignore_ascii_case("TRUE") {
            true => Value::Bool(true),
            false => match v.0.eq_ignore_ascii_case("FALSE") {
                true => Value::Bool(false),
                false => Value::String(v.0.to_string()),
            },
        }],
        IcalValue::CalAddress(v) => vec![Value::String(v.0.to_string())],
        IcalValue::Date(v) => vec![Value::String(date_to_json(&v.0))],
        IcalValue::DateTime(v) => vec![Value::String(datetime_to_json(&v.0))],
        IcalValue::DateTimeList(v) => {
            v.0.iter()
                .map(|item| Value::String(period_to_json(item)))
                .collect()
        }
        IcalValue::Duration(v) => vec![Value::String(v.0.to_string())],
        IcalValue::Float(v) => vec![number(&v.0)],
        IcalValue::Geo(v) => vec![Value::Array(vec![
            number(&v.latitude),
            number(&v.longitude),
        ])],
        IcalValue::Integer(v) => vec![number(&v.0)],
        IcalValue::Period(v) => vec![Value::String(period_to_json(&v.0))],
        IcalValue::Recur(v) => vec![recur_to_json(&v.0)],
        IcalValue::RequestStatus(v) => {
            let mut parts = vec![
                Value::String(v.code.to_string()),
                Value::String(v.description.to_string()),
            ];
            if !v.extra.is_empty() {
                parts.push(Value::String(v.extra.to_string()));
            }
            vec![Value::Array(parts)]
        }
        IcalValue::Text(v) => vec![Value::String(v.0.to_string())],
        IcalValue::TextList(v) => {
            v.0.iter()
                .map(|item| Value::String(item.to_string()))
                .collect()
        }
        IcalValue::Time(v) => vec![Value::String(time_to_json(&v.0))],
        IcalValue::Uri(v) => vec![Value::String(v.0.to_string())],
        IcalValue::UtcOffset(v) => vec![Value::String(offset_to_json(&v.0))],
        IcalValue::Unknown(v) => unknown_to_jcal(v),
    }
}

/// An undecoded value, structurally: its components and their comma-separated
/// values, mirrored rather than re-escaped into one string. The decoded model
/// holds unescaped pieces, and re-escaping belongs to the wire codec.
fn unknown_to_jcal(value: &IcalUnknownValue<'_>) -> Vec<Value> {
    match value.components.as_slice() {
        [] => vec![Value::String(String::new())],
        [one] => one
            .iter()
            .map(|item| Value::String(item.to_string()))
            .collect(),
        many => many
            .iter()
            .map(|component| {
                Value::Array(
                    component
                        .iter()
                        .map(|item| Value::String(item.to_string()))
                        .collect(),
                )
            })
            .collect(),
    }
}

/// The value of a property, read back as the kind its type slot named.
fn value_from_jcal<'a>(kind: Option<IcalValueKind>, values: &'a [Value]) -> IcalValue<'a> {
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
        return IcalValue::Unknown(unknown_from_jcal(values));
    };

    match kind {
        // NOTE: RFC 7265 3.5.1 spells a binary value as base64 text; the URI
        // form of ATTACH arrives through the `uri` type slot instead.
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
        IcalValueKind::Period => IcalValue::Period(IcalPeriod(rewritten(text(), period_from_json))),
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

/// An undecoded value read back, mirroring [`unknown_to_jcal`].
fn unknown_from_jcal<'a>(values: &'a [Value]) -> IcalUnknownValue<'a> {
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

/// A number as JSON, or the raw text when it is not one. An integer stays an
/// integer: `5` and `5.0` are the same number to JSON but not to a reader.
fn number(text: &str) -> Value {
    if let Ok(whole) = text.parse::<i64>() {
        return Value::Number(whole.into());
    }

    text.parse::<f64>()
        .ok()
        .and_then(serde_json::Number::from_f64)
        .map(Value::Number)
        .unwrap_or_else(|| Value::String(text.to_string()))
}

/// `YYYYMMDD` to `YYYY-MM-DD`. Anything else passes through.
pub(crate) fn date_to_json(text: &str) -> String {
    match text.len() == 8 && text.bytes().all(|b| b.is_ascii_digit()) {
        true => format!("{}-{}-{}", &text[..4], &text[4..6], &text[6..8]),
        false => text.to_string(),
    }
}

/// The `from_json` side is written as "the rewritten text, or `None` when there
/// is nothing to rewrite", so a value that arrived borrowed stays borrowed and
/// only a value that actually changes is allocated.
pub(crate) fn date_from_json(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    (bytes.len() == 10 && bytes[4] == b'-' && bytes[7] == b'-')
        .then(|| format!("{}{}{}", &text[..4], &text[5..7], &text[8..10]))
}

/// `HHMMSS` to `HH:MM:SS`, with the UTC suffix kept. Anything else passes
/// through.
fn time_to_json(text: &str) -> String {
    let (body, zulu) = split_zulu(text);
    match body.len() == 6 && body.bytes().all(|b| b.is_ascii_digit()) {
        true => format!("{}:{}:{}{zulu}", &body[..2], &body[2..4], &body[4..6]),
        false => text.to_string(),
    }
}

fn time_from_json(text: &str) -> Option<String> {
    let (body, zulu) = split_zulu(text);
    let bytes = body.as_bytes();
    (bytes.len() == 8 && bytes[2] == b':' && bytes[5] == b':')
        .then(|| format!("{}{}{}{zulu}", &body[..2], &body[3..5], &body[6..8]))
}

/// `YYYYMMDDTHHMMSS` to `YYYY-MM-DDTHH:MM:SS`, with the UTC suffix kept. A
/// date-only value is re-spelled as a date. Anything else passes through.
pub(crate) fn datetime_to_json(text: &str) -> String {
    let (body, zulu) = split_zulu(text);

    let Some((date, time)) = split_t(body) else {
        return date_to_json(body) + zulu;
    };

    format!("{}T{}{zulu}", date_to_json(date), time_to_json(time))
}

/// The inverse of [`datetime_to_json`]: `YYYY-MM-DDTHH:MM:SS` back to
/// `YYYYMMDDTHHMMSS`, the UTC suffix kept.
///
/// `None` when neither half was in the JSON spelling, so a value already on
/// the wire form is left for the caller to pass through untouched.
pub(crate) fn datetime_from_json(text: &str) -> Option<String> {
    let (body, zulu) = split_zulu(text);

    let Some((date, time)) = split_t(body) else {
        return date_from_json(body).map(|date| format!("{date}{zulu}"));
    };

    let (rewritten_date, rewritten_time) = (date_from_json(date), time_from_json(time));
    if rewritten_date.is_none() && rewritten_time.is_none() {
        return None;
    }

    Some(format!(
        "{}T{}{zulu}",
        rewritten_date.as_deref().unwrap_or(date),
        rewritten_time.as_deref().unwrap_or(time),
    ))
}

/// A period (`start/end` or `start/duration`) with both halves re-spelled. A
/// value with no `/` is re-spelled as a date-time, which is what an `RDATE`
/// item is.
fn period_to_json(text: &str) -> String {
    match text.split_once('/') {
        Some((start, end)) => format!("{}/{}", datetime_to_json(start), datetime_to_json(end)),
        None => datetime_to_json(text),
    }
}

fn period_from_json(text: &str) -> Option<String> {
    let Some((start, end)) = text.split_once('/') else {
        return datetime_from_json(text);
    };

    let (rewritten_start, rewritten_end) = (datetime_from_json(start), datetime_from_json(end));
    if rewritten_start.is_none() && rewritten_end.is_none() {
        return None;
    }

    Some(format!(
        "{}/{}",
        rewritten_start.as_deref().unwrap_or(start),
        rewritten_end.as_deref().unwrap_or(end),
    ))
}

/// `+HHMM[SS]` to `+HH:MM[:SS]`. Anything else passes through.
fn offset_to_json(text: &str) -> String {
    let (sign, digits) = match text.as_bytes().first() {
        Some(b'+') | Some(b'-') => (&text[..1], &text[1..]),
        _ => ("", text),
    };

    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return text.to_string();
    }

    match digits.len() {
        4 => format!("{sign}{}:{}", &digits[..2], &digits[2..4]),
        6 => format!("{sign}{}:{}:{}", &digits[..2], &digits[2..4], &digits[4..6]),
        _ => text.to_string(),
    }
}

fn offset_from_json(text: &str) -> Option<String> {
    text.contains(':').then(|| text.replace(':', ""))
}

/// The rule parts in the order RFC 5545 3.3.10 states them, which is the order
/// a rule read back from JSON comes out in.
const RECUR_ORDER: [&str; 14] = [
    "FREQ",
    "UNTIL",
    "COUNT",
    "INTERVAL",
    "BYSECOND",
    "BYMINUTE",
    "BYHOUR",
    "BYDAY",
    "BYMONTHDAY",
    "BYYEARDAY",
    "BYWEEKNO",
    "BYMONTH",
    "BYSETPOS",
    "WKST",
];

/// A rule as the RFC 7265 3.5.6 object: one lowercase key per part, a number
/// where the part is numeric, an array where it has several values.
fn recur_to_json(text: &str) -> Value {
    let mut parts = Map::new();

    for part in text.split(';').filter(|part| !part.is_empty()) {
        let (name, value) = match part.split_once('=') {
            Some(split) => split,
            None => (part, ""),
        };

        let name = name.to_ascii_lowercase();
        let values: Vec<Value> = value
            .split(',')
            .map(|item| match name.as_str() {
                "until" => Value::String(datetime_to_json(item)),
                "freq" | "wkst" | "byday" | "rscale" | "skip" => {
                    Value::String(item.to_ascii_lowercase())
                }
                _ => number(item),
            })
            .collect();

        parts.insert(
            name,
            match values.as_slice() {
                [one] => one.clone(),
                many => Value::Array(many.to_vec()),
            },
        );
    }

    Value::Object(parts)
}

/// A rule read back, its parts in the RFC's order. A JSON object has no order
/// to preserve, so this is a normalisation, not a loss.
fn recur_from_json(value: Option<&Value>) -> Cow<'_, str> {
    let Some(Value::Object(parts)) = value else {
        return value.map(scalar_text).unwrap_or(Cow::Borrowed(""));
    };

    let mut names: Vec<&String> = parts.keys().collect();
    names.sort_by_key(|name| {
        RECUR_ORDER
            .iter()
            .position(|known| known.eq_ignore_ascii_case(name))
            .unwrap_or(RECUR_ORDER.len())
    });

    let mut rule = String::new();

    for name in names {
        let value = &parts[name];
        let text = match value {
            Value::Array(items) => items
                .iter()
                .map(|item| recur_part_text(name, item))
                .collect::<Vec<_>>()
                .join(","),
            other => recur_part_text(name, other),
        };

        if !rule.is_empty() {
            rule.push(';');
        }
        rule.push_str(&name.to_ascii_uppercase());
        rule.push('=');
        rule.push_str(&text);
    }

    Cow::Owned(rule)
}

/// One value of one rule part, back in its wire spelling.
fn recur_part_text(name: &str, value: &Value) -> String {
    let text = scalar_text(value);

    match name {
        "until" => datetime_from_json(&text).unwrap_or_else(|| text.into_owned()),
        "freq" | "wkst" | "byday" | "rscale" | "skip" => text.to_ascii_uppercase(),
        _ => text.into_owned(),
    }
}

/// Split the trailing UTC `Z` off a value.
fn split_zulu(text: &str) -> (&str, &str) {
    match text.as_bytes().last() {
        Some(b'Z') | Some(b'z') => (&text[..text.len() - 1], &text[text.len() - 1..]),
        _ => (text, ""),
    }
}

/// Split a date-time on its `T`.
fn split_t(text: &str) -> Option<(&str, &str)> {
    let at = text.find(['T', 't'])?;
    Some((&text[..at], &text[at + 1..]))
}
