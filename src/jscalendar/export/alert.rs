//! # Alerts
//!
//! A `VALARM` as an Alert object (RFC 8984 4.5.2): when it fires, and whether
//! it displays or sends mail.

use alloc::{borrow::ToOwned, format, string::ToString};

use serde_json::{Map, Value, json};

use crate::{
    component::IcalComponent,
    jscalendar::{
        export::temporal::{local_text, utc},
        export::{Builder, component_key, param, text},
        hatch::IcalHatch,
    },
    param::IcalParamKind,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    value::IcalValue,
};

impl Builder {
    /// A `VALARM` as an Alert (draft 2.2.2).
    pub(super) fn alarm(&mut self, component: &IcalComponent<'_>) {
        let mut alert = Map::new();
        alert.insert("@type".to_owned(), Value::String("Alert".to_owned()));

        let mut hatch = IcalHatch::new("valarm");

        for prop in &component.props {
            let IcalPropName::Kind(kind) = &prop.name else {
                // NOTE: A JSID names the key its component took, and the key
                // is where that already is.
                if !prop.name.eq_ignore_ascii_case("JSID") {
                    hatch.keep(prop);
                }

                continue;
            };

            match kind {
                IcalPropKind::Trigger => match trigger(prop) {
                    Some(trigger) => {
                        alert.insert("trigger".to_owned(), trigger);
                        hatch.note(
                            "trigger",
                            prop,
                            &[IcalParamKind::Related, IcalParamKind::Value],
                        );
                    }
                    None => hatch.keep(prop),
                },
                IcalPropKind::Action => match text(prop).map(|action| action.to_lowercase()) {
                    // NOTE: Only these two actions have a JSCalendar meaning
                    // (RFC 8984 4.5.2); the rest stay whole in the hatch.
                    Some(action) if action == "display" || action == "email" => {
                        alert.insert("action".to_owned(), Value::String(action));
                        hatch.note("action", prop, &[]);
                    }
                    _ => hatch.keep(prop),
                },
                IcalPropKind::Acknowledged => match utc(prop) {
                    Some(at) => {
                        alert.insert("acknowledged".to_owned(), Value::String(at));
                        hatch.note("acknowledged", prop, &[]);
                    }
                    None => hatch.keep(prop),
                },
                _ => hatch.keep(prop),
            }
        }

        for child in &component.components {
            hatch.keep_component(child);
        }

        if let Some(hatch) = hatch.into_value() {
            alert.insert("iCalendar".to_owned(), hatch);
        }

        let key = component_key(&self.alerts, component);
        self.alerts.insert(key, Value::Object(alert));
    }
}

/// A `TRIGGER` as an OffsetTrigger or an AbsoluteTrigger (draft 2.3.44).
fn trigger(prop: &IcalProp<'_>) -> Option<Value> {
    let text = text(prop)?;

    if matches!(prop.value, IcalValue::DateTime(_) | IcalValue::Date(_)) {
        let when = local_text(&text, false).map(|when| format!("{when}Z"))?;
        return Some(json!({"@type": "AbsoluteTrigger", "when": when}));
    }

    let mut object = Map::new();
    object.insert(
        "@type".to_owned(),
        Value::String("OffsetTrigger".to_owned()),
    );
    object.insert("offset".to_owned(), Value::String(text.to_string()));

    if let Some(related) = param(prop, IcalParamKind::Related)
        && related.eq_ignore_ascii_case("END")
    {
        object.insert("relativeTo".to_owned(), Value::String("end".to_owned()));
    }

    Some(Value::Object(object))
}
