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

pub(crate) mod datetime;
mod export;
mod import;
mod json;
mod recur;

use core::{error, fmt};

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value, json};

use crate::{
    component::IcalComponent, ical::Ical, jcal::import::split_component, prop::IcalProp,
    value::IcalValue, version::IcalVersion,
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
        props.extend(self.props.iter().map(IcalProp::to_jcal));

        let components: Vec<Value> = self.components.iter().map(IcalComponent::to_jcal).collect();

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
            let prop = IcalProp::from_jcal(entry, version);

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
                .map(|component| IcalComponent::from_jcal(component, version))
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, vec};

    use crate::{
        component::{IcalComponent, IcalComponentKind},
        ical::Ical,
        jcal::IcalJcalError,
        param::IcalParam,
        prop::{IcalProp, IcalPropKind},
        value::{IcalValue, datetime::IcalDateTime, text::IcalText},
        version::IcalVersion,
    };

    /// A hand-built calendar, so the codec is exercised with no parser.
    fn calendar() -> Ical<'static> {
        Ical {
            version: IcalVersion::V2_0,
            props: vec![IcalProp {
                name: IcalPropKind::ProdId.into(),
                params: vec![],
                value: IcalValue::Text(IcalText(Cow::Borrowed("-//Example//EN"))),
            }],
            components: vec![IcalComponent {
                name: IcalComponentKind::VEvent.into(),
                props: vec![
                    IcalProp {
                        name: IcalPropKind::Uid.into(),
                        params: vec![],
                        value: IcalValue::Text(IcalText(Cow::Borrowed("42@example.com"))),
                    },
                    IcalProp {
                        name: IcalPropKind::DtStart.into(),
                        params: vec![],
                        value: IcalValue::DateTime(IcalDateTime(Cow::Borrowed("20260102T120000Z"))),
                    },
                    IcalProp {
                        name: IcalPropKind::Summary.into(),
                        params: vec![IcalParam::Language(Cow::Borrowed("en"))],
                        value: IcalValue::Text(IcalText(Cow::Borrowed("Lunch"))),
                    },
                ],
                components: vec![],
            }],
        }
    }

    #[test]
    fn round_trips_a_calendar_built_with_no_parser() {
        let cal = calendar();
        let jcal = cal.to_jcal();

        assert_eq!(Ical::from_jcal(&jcal).expect("a vcalendar"), cal);
    }

    #[test]
    fn refuses_a_document_that_is_not_a_vcalendar() {
        let jcal = serde_json::json!(["vevent", [], []]);

        assert_eq!(
            Ical::from_jcal(&jcal),
            Err(IcalJcalError::NotACalendar("vevent".into()))
        );
    }
}
