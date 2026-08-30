//! # Alerts
//!
//! An Alert object read back as a `VALARM`: what it does, and when it fires.

use alloc::{
    borrow::{Cow, ToOwned},
    vec::Vec,
};

use serde_json::Value;

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    jscalendar::{
        hatch::{hatch_of, kept_components, kept_props},
        import::{plain, temporal::basic},
    },
    param::IcalParam,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    value::{IcalValue, datetime::IcalDateTime, text::IcalText},
    version::IcalVersion,
};

/// An Alert as the `VALARM` it came from.
pub(super) fn alarm(key: &str, alert: &Value) -> IcalComponent<'static> {
    let hatch = alert.as_object().and_then(hatch_of);
    let mut props = Vec::new();

    if let Some(trigger) = alert.get("trigger") {
        props.push(trigger_prop(trigger));
    }

    if let Some(action) = alert.get("action").and_then(Value::as_str) {
        props.push(plain(IcalPropKind::Action, action.to_ascii_uppercase()));
    }

    if let Some(at) = alert.get("acknowledged").and_then(Value::as_str) {
        props.push(plain(IcalPropKind::Acknowledged, basic(at)));
    }

    let kept = kept_props(hatch, IcalVersion::V2_0);

    // NOTE: An alarm names itself with its UID where the hatch kept one, so a
    // JSID is needed only when the key says something the UID does not (draft
    // 2.2.2, 4.1.1).
    let named = kept.iter().any(|prop| {
        prop.name.eq_ignore_ascii_case("UID")
            && matches!(&prop.value, IcalValue::Text(text) if text.0 == key)
    });

    props.extend(kept.into_iter().map(IcalProp::into_owned));

    if !named {
        props.push(IcalProp {
            name: IcalPropName::Unknown(Cow::Owned("JSID".to_owned())),
            params: Vec::new(),
            value: IcalValue::Text(IcalText(Cow::Owned(key.to_owned()))),
        });
    }

    IcalComponent {
        name: IcalComponentName::Kind(IcalComponentKind::VAlarm),
        props,
        components: kept_components(hatch, IcalVersion::V2_0)
            .into_iter()
            .map(IcalComponent::into_owned)
            .collect(),
    }
}

/// A trigger object as the `TRIGGER` property it came from.
pub(super) fn trigger_prop(trigger: &Value) -> IcalProp<'static> {
    if let Some(when) = trigger.get("when").and_then(Value::as_str) {
        let mut prop = plain(IcalPropKind::Trigger, basic(when));
        prop.value = IcalValue::DateTime(IcalDateTime(match &prop.value {
            IcalValue::Text(text) => text.0.clone(),
            _ => Cow::Borrowed(""),
        }));
        prop.params
            .push(IcalParam::Value(Cow::Borrowed("DATE-TIME")));
        return prop;
    }

    let offset = trigger
        .get("offset")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut prop = plain(IcalPropKind::Trigger, offset.to_owned());

    if trigger.get("relativeTo").and_then(Value::as_str) == Some("end") {
        prop.params.push(IcalParam::Related(Cow::Borrowed("END")));
    }

    prop
}
