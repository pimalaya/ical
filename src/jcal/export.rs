//! # Export
//!
//! The model-to-jCal half: a decoded component, property, parameter and value
//! written as their RFC 7265 slots.
//!
//! Out, the RFC is followed: names are lowercased (3.3), the `VALUE` parameter
//! moves into the type slot (3.5.4), and dates, times, periods, offsets and
//! recurrence rules are re-spelled in the JSON forms (3.5.1 to 3.5.7).

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value, json};

use crate::{
    component::IcalComponent,
    jcal::{
        datetime::{date_to_json, datetime_to_json, offset_to_json, period_to_json, time_to_json},
        json::{number, strings},
        recur::recur_to_json,
    },
    param::{IcalParam, IcalParamKind},
    prop::IcalProp,
    value::{IcalUnknownValue, IcalValue, IcalValueKind, binary::IcalBinary},
};

impl IcalComponent<'_> {
    /// The component as `[name, [properties], [components]]`.
    pub(crate) fn to_jcal(&self) -> Value {
        let props: Vec<Value> = self.props.iter().map(IcalProp::to_jcal).collect();
        let children: Vec<Value> = self.components.iter().map(IcalComponent::to_jcal).collect();

        json!([self.name.to_lowercase(), props, children])
    }
}

impl IcalProp<'_> {
    /// The property as `[name, {params}, type, value...]`.
    pub(crate) fn to_jcal(&self) -> Value {
        let mut params = Map::new();

        for param in &self.params {
            // NOTE: The declared VALUE moves into the type slot (RFC 7265
            // 3.5.4).
            if matches!(param.kind(), Some(IcalParamKind::Value)) {
                continue;
            }

            let (name, value) = param.to_jcal();
            params.insert(name, value);
        }

        let mut entry = vec![
            Value::String(self.name.to_lowercase()),
            Value::Object(params),
            Value::String(self.type_slot()),
        ];
        entry.extend(self.value.to_jcal());

        Value::Array(entry)
    }

    /// The type slot: what `VALUE` declared, else the value's kind, else
    /// `unknown`.
    ///
    /// The declared kind wins as the finer answer: `date-time-list` is a
    /// modelling detail rather than an RFC 7265 type, so `RDATE;VALUE=PERIOD`
    /// goes out as `period`, and import resolves it back through the spec.
    pub(crate) fn type_slot(&self) -> String {
        let declared = self.params.iter().find_map(|param| match param {
            IcalParam::Value(kind) => Some(kind.to_ascii_lowercase()),
            _ => None,
        });

        if let Some(declared) = declared {
            return declared;
        }

        match self.value.kind() {
            // NOTE: A list is several values of one kind, and that kind is
            // what jCal names.
            Some(IcalValueKind::DateTimeList) => (*IcalValueKind::DateTime).to_ascii_lowercase(),
            Some(IcalValueKind::TextList) => (*IcalValueKind::Text).to_ascii_lowercase(),
            Some(kind) => (*kind).to_ascii_lowercase(),
            None => String::from("unknown"),
        }
    }
}

impl IcalParam<'_> {
    /// The parameter as a `name: value` pair, a JSON array when it has several
    /// values.
    pub(crate) fn to_jcal(&self) -> (String, Value) {
        if let IcalParam::Unknown { name, values } = self {
            return (name.to_ascii_lowercase(), strings(values));
        }

        let name = self
            .kind()
            .map(|kind| (*kind).to_ascii_lowercase())
            .unwrap_or_default();

        let value = match self {
            IcalParam::DelegatedFrom(values)
            | IcalParam::DelegatedTo(values)
            | IcalParam::Member(values)
            | IcalParam::Feature(values) => strings(values),
            other => Value::String(other.scalar().into_owned()),
        };

        (name, value)
    }

    /// The text of a single-valued parameter.
    pub(crate) fn scalar(&self) -> Cow<'_, str> {
        match self {
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
}

impl IcalValue<'_> {
    /// The value slots of a property: one per value, several for a list.
    fn to_jcal(&self) -> Vec<Value> {
        match self {
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
            IcalValue::Unknown(v) => v.to_jcal(),
        }
    }
}

impl IcalUnknownValue<'_> {
    /// An undecoded value, structurally: its components and their
    /// comma-separated values, mirrored rather than re-escaped into one
    /// string. The decoded model holds unescaped pieces, and re-escaping
    /// belongs to the wire codec.
    fn to_jcal(&self) -> Vec<Value> {
        match self.components.as_slice() {
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
}
