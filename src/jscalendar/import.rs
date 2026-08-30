//! # Import
//!
//! A JSCalendar `Group` back as a decoded calendar.
//!
//! Every member the export writes is read back under the name the escape
//! hatch recorded for it, or, where nothing was recorded, under the name
//! [`default_name`](crate::jscalendar::hatch::default_name) assumes.
//!
//! A member no iCalendar property holds becomes a `JSPROP` property carrying
//! its JSON, located by a `JSPTR` parameter, which is the mirror hatch the
//! conversion draft defines (4.1.2, 4.2.2).

mod alert;
mod participant;
mod place;
mod temporal;

use alloc::{
    borrow::{Cow, ToOwned},
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value};

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    ical::Ical,
    jscalendar::{
        hatch::{IcalConverted, converted, hatch_of, kept_components, kept_props},
        import::{
            alert::alarm,
            participant::participant,
            place::{conference, link, location},
            temporal::{basic, ends, occurrence, rule_from_json, temporal},
        },
        patch,
    },
    param::IcalParam,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    value::{
        IcalValue,
        integer::IcalInteger,
        recur::IcalRecur,
        request_status::IcalRequestStatus,
        text::{IcalText, IcalTextList},
        uri::IcalUri,
    },
    version::IcalVersion,
};

/// The members a Group holds itself, so everything else in it is a JSCalendar
/// property with no iCalendar counterpart.
const GROUP_MEMBERS: [&str; 11] = [
    "@type",
    "entries",
    "iCalendar",
    "uid",
    "prodId",
    "title",
    "description",
    "source",
    "color",
    "created",
    "updated",
];

/// A JSCalendar `Group` as a decoded calendar.
pub(crate) fn ical(group: &Map<String, Value>) -> Ical<'static> {
    let hatch = hatch_of(group);
    let kept = kept_props(hatch, IcalVersion::V2_0);

    let version = kept
        .iter()
        .find(|prop| prop.name.eq_ignore_ascii_case("VERSION"))
        .and_then(|prop| match &prop.value {
            IcalValue::Text(text) => text.0.parse().ok(),
            _ => None,
        })
        .unwrap_or(IcalVersion::V2_0);

    let mut props: Vec<IcalProp<'static>> = kept
        .into_iter()
        .filter(|prop| !prop.name.eq_ignore_ascii_case("VERSION"))
        .map(IcalProp::into_owned)
        .collect();

    for (member, value) in group {
        if GROUP_MEMBERS.contains(&member.as_str()) {
            continue;
        }

        match member.as_str() {
            "keywords" => props.extend(keys(value).map(|keyword| {
                text_prop(
                    named(hatch, "vcalendar", "keywords", IcalPropKind::Categories),
                    keyword,
                )
            })),
            "method" => props.push(text_prop(
                named(hatch, "vcalendar", "method", IcalPropKind::Method),
                value.as_str().unwrap_or_default().to_ascii_uppercase(),
            )),
            _ => props.push(jsprop(member, value)),
        }
    }

    for (member, kind) in [
        ("uid", IcalPropKind::Uid),
        ("prodId", IcalPropKind::ProdId),
        ("title", IcalPropKind::Name),
        ("description", IcalPropKind::Description),
        ("source", IcalPropKind::Source),
        ("color", IcalPropKind::Color),
        ("created", IcalPropKind::Created),
        ("updated", IcalPropKind::LastModified),
    ] {
        let Some(text) = group.get(member).and_then(Value::as_str) else {
            continue;
        };

        let record = named(hatch, "vcalendar", member, kind);
        let text = match member {
            "created" | "updated" => basic(text),
            _ => text.to_owned(),
        };

        props.push(text_prop(record, text));
    }

    let mut components: Vec<IcalComponent<'_>> = group
        .get("entries")
        .and_then(Value::as_array)
        .map(|entries| entries.iter().flat_map(entry).collect())
        .unwrap_or_default();

    components.extend(
        kept_components(hatch, version)
            .into_iter()
            .map(IcalComponent::into_owned),
    );

    Ical {
        version,
        props,
        components,
    }
}

/// A lone Event or Task as the calendar holding it.
pub(crate) fn of_entry(object: &Value) -> Ical<'static> {
    Ical {
        version: IcalVersion::V2_0,
        props: Vec::new(),
        components: entry(object),
    }
}

/// One Group entry as the component (or components) it converts back to.
///
/// A series carrying overrides comes back as several components: the series
/// itself, then one overriding component per patch, which is how iCalendar
/// states what JSCalendar folds into one object (draft 2.1.2).
fn entry(entry: &Value) -> Vec<IcalComponent<'static>> {
    let Some(object) = entry.as_object() else {
        return Vec::new();
    };

    let task = matches!(object.get("@type").and_then(Value::as_str), Some("Task"));
    let name = match task {
        true => IcalComponentKind::VTodo,
        false => IcalComponentKind::VEvent,
    };

    let overrides = object
        .get("recurrenceOverrides")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut series = object.clone();
    series.remove("recurrenceOverrides");

    let zone = series.get("timeZone").and_then(Value::as_str);
    let date_only = all_day(&series);
    let mut components = vec![component(&series, name, task)];

    for (id, over) in &overrides {
        let Some(patch) = over.as_object() else {
            continue;
        };

        // NOTE: An empty patch is an added occurrence and an `excluded` one a
        // removed occurrence; neither is a component of its own.
        if patch.is_empty() {
            components[0]
                .props
                .push(occurrence(IcalPropKind::RDate, id, zone, date_only));
            continue;
        }

        if patch.get("excluded") == Some(&Value::Bool(true)) {
            components[0]
                .props
                .push(occurrence(IcalPropKind::ExDate, id, zone, date_only));
            continue;
        }

        let mut instance = series.clone();
        patch::apply(&mut instance, patch);
        instance.remove("recurrenceRules");
        instance.remove("excludedRecurrenceRules");

        let mut overriding = component(&instance, name, task);
        overriding
            .props
            .push(occurrence(IcalPropKind::RecurrenceId, id, zone, date_only));
        components.push(overriding);
    }

    components
}

/// The members an Event or Task holds itself.
const ENTRY_MEMBERS: [&str; 13] = [
    "@type",
    "iCalendar",
    "duration",
    "timeZone",
    "recurrenceIdTimeZone",
    "showWithoutTime",
    "descriptionContentType",
    "keywords",
    "categories",
    "relatedTo",
    "replyTo",
    "recurrenceOverrides",
    "requestStatus",
];

/// One Event or Task as a `VEVENT` or `VTODO`.
fn component(
    object: &Map<String, Value>,
    name: IcalComponentKind,
    task: bool,
) -> IcalComponent<'static> {
    let hatch = hatch_of(object);
    let component = match task {
        true => "vtodo",
        false => "vevent",
    };

    let zone = object.get("timeZone").and_then(Value::as_str);
    let mut props: Vec<IcalProp<'static>> = Vec::new();
    let mut components: Vec<IcalComponent<'static>> = Vec::new();
    let mut organizer = false;

    for (member, value) in object {
        if ENTRY_MEMBERS.contains(&member.as_str()) {
            continue;
        }

        let record = |kind: IcalPropKind| named(hatch, component, member, kind);

        match (member.as_str(), value) {
            ("uid", Value::String(text)) => {
                props.push(text_prop(record(IcalPropKind::Uid), text.clone()))
            }
            ("title", Value::String(text)) => {
                props.push(text_prop(record(IcalPropKind::Summary), text.clone()))
            }
            ("description", Value::String(text)) => {
                props.push(text_prop(record(IcalPropKind::Description), text.clone()))
            }
            ("color", Value::String(text)) => {
                props.push(text_prop(record(IcalPropKind::Color), text.clone()))
            }
            ("method", Value::String(text)) => props.push(text_prop(
                record(IcalPropKind::Method),
                text.to_ascii_uppercase(),
            )),
            ("privacy", Value::String(text)) => props.push(text_prop(
                record(IcalPropKind::Class),
                match text.as_str() {
                    "secret" => "CONFIDENTIAL".to_owned(),
                    other => other.to_ascii_uppercase(),
                },
            )),
            ("status" | "progress", Value::String(text)) => props.push(text_prop(
                record(IcalPropKind::Status),
                text.to_ascii_uppercase(),
            )),
            ("freeBusyStatus", Value::String(text)) => props.push(text_prop(
                record(IcalPropKind::Transp),
                match text.as_str() {
                    "free" => "TRANSPARENT".to_owned(),
                    _ => "OPAQUE".to_owned(),
                },
            )),
            ("created" | "updated" | "progressUpdated" | "acknowledged", Value::String(text)) => {
                let kind = match member.as_str() {
                    "created" => IcalPropKind::Created,
                    "updated" => IcalPropKind::DtStamp,
                    _ => IcalPropKind::Completed,
                };

                props.push(text_prop(record(kind), basic(text)));
            }
            ("sequence" | "priority" | "percentComplete", Value::Number(number)) => {
                let kind = match member.as_str() {
                    "sequence" => IcalPropKind::Sequence,
                    "priority" => IcalPropKind::Priority,
                    _ => IcalPropKind::PercentComplete,
                };

                let mut prop = text_prop(record(kind), number.to_string());
                prop.value = IcalValue::Integer(IcalInteger(Cow::Owned(number.to_string())));
                props.push(prop);
            }
            ("start" | "due" | "recurrenceId", Value::String(text)) => {
                let kind = match member.as_str() {
                    "start" => IcalPropKind::DtStart,
                    "due" => IcalPropKind::Due,
                    _ => IcalPropKind::RecurrenceId,
                };

                let zone = match member.as_str() {
                    "recurrenceId" => object.get("recurrenceIdTimeZone").and_then(Value::as_str),
                    _ => zone,
                };

                props.push(temporal(record(kind), text, zone, all_day(object)));
            }
            ("recurrenceRules" | "excludedRecurrenceRules", Value::Array(rules)) => {
                let kind = match member.as_str() {
                    "recurrenceRules" => IcalPropKind::RRule,
                    _ => IcalPropKind::ExRule,
                };

                for rule in rules {
                    let mut prop = text_prop(record(kind), rule_from_json(rule, zone));
                    prop.value = IcalValue::Recur(IcalRecur(match &prop.value {
                        IcalValue::Text(text) => text.0.clone(),
                        _ => Cow::Borrowed(""),
                    }));
                    props.push(prop);
                }
            }
            ("links", Value::Object(links)) => props.extend(
                links
                    .iter()
                    .map(|(key, link)| self::link(hatch, component, key, link)),
            ),
            ("locations", Value::Object(locations)) => {
                for (key, location) in locations {
                    match self::location(hatch, component, key, location) {
                        Ok(prop) => props.push(prop),
                        Err(nested) => components.push(nested),
                    }
                }
            }
            ("virtualLocations", Value::Object(locations)) => props.extend(
                locations
                    .iter()
                    .map(|(key, location)| conference(hatch, component, key, location)),
            ),
            ("participants", Value::Object(participants)) => {
                for (key, participant) in participants {
                    match self::participant(hatch, component, key, participant) {
                        Ok(prop) => {
                            organizer |= prop.name.eq_ignore_ascii_case("ORGANIZER");
                            props.push(prop)
                        }
                        Err(nested) => components.push(nested),
                    }
                }
            }
            ("alerts", Value::Object(alerts)) => {
                components.extend(alerts.iter().map(|(key, alert)| alarm(key, alert)))
            }
            _ => props.push(jsprop(member, value)),
        }
    }

    if let Some(span) = object.get("duration").and_then(Value::as_str) {
        let record = named(hatch, component, "duration", IcalPropKind::Duration);
        let start = object
            .get("start")
            .or_else(|| object.get("due"))
            .and_then(Value::as_str);

        // NOTE: A duration that was a DTEND has to become one again, and an end
        // is a start plus a span rather than the span itself.
        match (record.name.eq_ignore_ascii_case("DTEND"), start) {
            (true, Some(start)) => match ends(start, span) {
                Some(end) => props.push(temporal(record, &end, zone, all_day(object))),
                None => props.push(text_prop(record, span.to_owned())),
            },
            _ => props.push(text_prop(record, span.to_owned())),
        }
    }

    if let Some(keywords) = object.get("keywords").and_then(Value::as_object) {
        let mut prop = text_prop(
            named(hatch, component, "keywords", IcalPropKind::Categories),
            String::new(),
        );

        prop.value = IcalValue::TextList(IcalTextList(
            keywords.keys().map(|key| Cow::Owned(key.clone())).collect(),
        ));

        props.push(prop);
    }

    for concept in keys(object.get("categories").unwrap_or(&Value::Null)) {
        let mut prop = text_prop(
            named(hatch, component, "categories", IcalPropKind::Concept),
            concept,
        );

        prop.value = IcalValue::Uri(IcalUri(match &prop.value {
            IcalValue::Text(text) => text.0.clone(),
            _ => Cow::Borrowed(""),
        }));

        props.push(prop);
    }

    if let Some(related) = object.get("relatedTo").and_then(Value::as_object) {
        for (uid, relation) in related {
            let pointer = format!("relatedTo/{uid}");
            let mut prop = text_prop(
                named(hatch, component, &pointer, IcalPropKind::RelatedTo),
                uid.clone(),
            );

            let kind = relation
                .get("relation")
                .and_then(Value::as_object)
                .and_then(|kinds| kinds.keys().next());

            if let Some(kind) = kind {
                prop.params
                    .push(IcalParam::RelType(Cow::Owned(kind.to_ascii_uppercase())));
            }

            props.push(prop);
        }
    }

    // NOTE: The organizer is both a reply address and a participant owning the
    // object, and iCalendar has one line for the pair. The participant is where
    // it comes from when there is one, since only that carries the parameters;
    // `replyTo` alone writes a bare ORGANIZER.
    if let Some(address) = object
        .get("replyTo")
        .and_then(|to| to.get("imip"))
        .and_then(Value::as_str)
        .filter(|_| !organizer)
    {
        props.push(text_prop(
            named(hatch, component, "replyTo/imip", IcalPropKind::Organizer),
            address.to_owned(),
        ));
    }

    for status in object
        .get("requestStatus")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let Some(text) = status.as_str() else {
            continue;
        };
        let mut parts = text.splitn(3, ';');

        props.push(IcalProp {
            name: IcalPropName::Kind(IcalPropKind::RequestStatus),
            params: Vec::new(),
            value: IcalValue::RequestStatus(IcalRequestStatus {
                code: Cow::Owned(parts.next().unwrap_or_default().to_owned()),
                description: Cow::Owned(parts.next().unwrap_or_default().to_owned()),
                extra: Cow::Owned(parts.next().unwrap_or_default().to_owned()),
            }),
        });
    }

    props.extend(
        kept_props(hatch, IcalVersion::V2_0)
            .into_iter()
            .map(IcalProp::into_owned),
    );
    components.extend(
        kept_components(hatch, IcalVersion::V2_0)
            .into_iter()
            .map(IcalComponent::into_owned),
    );

    IcalComponent {
        name: IcalComponentName::Kind(name),
        props,
        components,
    }
}

/// Whether the object states a date without a time, which is what turns its
/// temporal properties back into `DATE` values.
fn all_day(object: &Map<String, Value>) -> bool {
    object.get("showWithoutTime") == Some(&Value::Bool(true))
}

/// The `RECUR` parts in the order RFC 5545 3.3.10 states them.
pub(super) const RULE_ORDER: [&str; 16] = [
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
    "RSCALE",
    "SKIP",
];

/// A member with no iCalendar counterpart, as the `JSPROP` property that
/// carries its JSON (draft 4.1.2).
fn jsprop(member: &str, value: &Value) -> IcalProp<'static> {
    IcalProp {
        name: IcalPropName::Unknown(Cow::Owned("JSPROP".to_owned())),
        params: vec![IcalParam::Unknown {
            name: Cow::Owned("JSPTR".to_owned()),
            values: vec![Cow::Owned(member.to_owned())],
        }],
        value: IcalValue::Text(IcalText(Cow::Owned(value.to_string()))),
    }
}

/// A property under the name and parameters the hatch recorded for a member.
pub(super) fn text_prop(record: IcalConverted<'_>, text: String) -> IcalProp<'static> {
    let mut params: Vec<IcalParam<'static>> = record
        .params
        .into_iter()
        .map(IcalParam::into_owned)
        .collect();

    if let Some(slot) = record.value_type {
        params.push(IcalParam::Value(Cow::Owned(slot.into_owned())));
    }

    IcalProp {
        name: IcalPropName::from(Cow::Owned(record.name.into_owned())),
        params,
        value: IcalValue::Text(IcalText(Cow::Owned(text))),
    }
}

/// A property under a name the mapping fixes, with no recorded leftovers.
pub(super) fn plain(kind: IcalPropKind, text: String) -> IcalProp<'static> {
    IcalProp {
        name: IcalPropName::Kind(kind),
        params: Vec::new(),
        value: IcalValue::Text(IcalText(Cow::Owned(text))),
    }
}

/// Tag a property with the JSCalendar key it converts back from, so the key
/// survives a further conversion (draft 4.2.1).
pub(super) fn keyed(mut prop: IcalProp<'static>, key: &str) -> IcalProp<'static> {
    prop.params.push(IcalParam::Unknown {
        name: Cow::Owned("JSID".to_owned()),
        values: vec![Cow::Owned(key.to_owned())],
    });

    prop
}

/// The record a hatch holds for a member, or the property the mapping assumes.
pub(super) fn named<'a>(
    hatch: Option<&'a Map<String, Value>>,
    component: &str,
    pointer: &str,
    fallback: IcalPropKind,
) -> IcalConverted<'a> {
    converted(hatch, component, pointer).unwrap_or(IcalConverted {
        name: Cow::Owned((*fallback).to_owned()),
        value_type: None,
        params: Vec::new(),
    })
}

/// The keys of a JSCalendar set, in order.
pub(super) fn keys(value: &Value) -> impl Iterator<Item = String> + '_ {
    value
        .as_object()
        .into_iter()
        .flat_map(|set| set.keys().map(String::clone))
}

/// A JSON scalar as the text iCalendar writes.
pub(super) fn scalar(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}
