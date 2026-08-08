//! # Import
//!
//! A JSCalendar `Group` back as a decoded calendar.
//!
//! Every member the export writes is read back under the name the escape
//! hatch recorded for it, or, where nothing was recorded, under the name
//! [`default_name`](crate::jscalendar::hatch::default_name) assumes. A member
//! no iCalendar property holds becomes a `JSPROP` property carrying its JSON,
//! located by a `JSPTR` parameter, which is the mirror hatch the conversion
//! draft defines (4.1.2, 4.2.2).

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
    jcal::datetime_from_json,
    jscalendar::{
        hatch::{IcalConverted, converted, hatch_of, kept_components, kept_props},
        patch,
    },
    param::IcalParam,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    recur::IcalRecurDateTime,
    value::{
        IcalValue,
        cal_address::IcalCalAddress,
        datetime::{IcalDate, IcalDateTime},
        geo::IcalGeo,
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

/// A Link as the property it came from (`ATTACH` unless recorded otherwise).
fn link(
    hatch: Option<&Map<String, Value>>,
    component: &str,
    key: &str,
    link: &Value,
) -> IcalProp<'static> {
    let pointer = format!("links/{key}");
    let href = link.get("href").and_then(Value::as_str).unwrap_or_default();

    let mut prop = text_prop(
        named(hatch, component, &pointer, IcalPropKind::Attach),
        href.to_owned(),
    );

    prop.value = IcalValue::Uri(IcalUri(Cow::Owned(href.to_owned())));

    if let Some(media) = link.get("contentType").and_then(Value::as_str) {
        prop.params
            .push(IcalParam::FmtType(Cow::Owned(media.to_owned())));
    }

    if let Some(display) = link.get("display").and_then(Value::as_str) {
        prop.params
            .push(IcalParam::Display(Cow::Owned(display.to_ascii_uppercase())));
    }

    if let Some(title) = link.get("title").and_then(Value::as_str) {
        prop.params
            .push(IcalParam::Label(Cow::Owned(title.to_owned())));
    }

    // NOTE: Only a LINK words its relation; an IMAGE's is always `icon` and an
    // ATTACH has none, so writing one back would invent a parameter.
    if let Some(rel) = link.get("rel").and_then(Value::as_str)
        && prop.name.eq_ignore_ascii_case("LINK")
    {
        prop.params
            .push(IcalParam::LinkRel(Cow::Owned(rel.to_ascii_uppercase())));
    }

    keyed(prop, key)
}

/// A Location as a `LOCATION` or `GEO` property, or as the `VLOCATION`
/// component it came from when it says more than one property can carry.
fn location(
    hatch: Option<&Map<String, Value>>,
    component: &str,
    key: &str,
    location: &Value,
) -> Result<IcalProp<'static>, IcalComponent<'static>> {
    let name = location.get("name").and_then(Value::as_str);
    let coordinates = location.get("coordinates").and_then(Value::as_str);
    let described =
        location.get("description").is_some() || location.get("locationTypes").is_some();

    if described || (name.is_some() && coordinates.is_some()) {
        return Err(vlocation(location, name, coordinates));
    }

    if let Some(coordinates) = coordinates {
        let pointer = format!("locations/{key}/coordinates");
        let mut prop = text_prop(
            named(hatch, component, &pointer, IcalPropKind::Geo),
            coordinates.to_owned(),
        );

        let pair = coordinates.trim_start_matches("geo:");
        let (latitude, longitude) = pair.split_once(',').unwrap_or((pair, ""));

        prop.value = IcalValue::Geo(IcalGeo {
            latitude: Cow::Owned(latitude.to_owned()),
            longitude: Cow::Owned(longitude.to_owned()),
        });

        return Ok(keyed(prop, key));
    }

    let pointer = format!("locations/{key}");
    let prop = text_prop(
        named(hatch, component, &pointer, IcalPropKind::Location),
        name.unwrap_or_default().to_owned(),
    );

    Ok(keyed(prop, key))
}

/// A Location that says more than a `LOCATION` line can, as a `VLOCATION`.
fn vlocation(
    location: &Value,
    name: Option<&str>,
    coordinates: Option<&str>,
) -> IcalComponent<'static> {
    let mut props = Vec::new();

    if let Some(name) = name {
        props.push(plain(IcalPropKind::Name, name.to_owned()));
    }

    if let Some(description) = location.get("description").and_then(Value::as_str) {
        props.push(plain(IcalPropKind::Description, description.to_owned()));
    }

    if let Some(coordinates) = coordinates {
        props.push(plain(IcalPropKind::Url, coordinates.to_owned()));
    }

    let types: Vec<Cow<'static, str>> = keys(location.get("locationTypes").unwrap_or(&Value::Null))
        .map(Cow::Owned)
        .collect();

    if !types.is_empty() {
        let mut prop = plain(IcalPropKind::LocationType, String::new());
        prop.value = IcalValue::TextList(IcalTextList(types));
        props.push(prop);
    }

    IcalComponent {
        name: IcalComponentName::Kind(IcalComponentKind::VLocation),
        props,
        components: Vec::new(),
    }
}

/// A VirtualLocation as a `CONFERENCE` property.
fn conference(
    hatch: Option<&Map<String, Value>>,
    component: &str,
    key: &str,
    location: &Value,
) -> IcalProp<'static> {
    let pointer = format!("virtualLocations/{key}");
    let uri = location
        .get("uri")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let mut prop = text_prop(
        named(hatch, component, &pointer, IcalPropKind::Conference),
        uri.to_owned(),
    );

    prop.value = IcalValue::Uri(IcalUri(Cow::Owned(uri.to_owned())));

    if let Some(name) = location.get("name").and_then(Value::as_str) {
        prop.params
            .push(IcalParam::Label(Cow::Owned(name.to_owned())));
    }

    let features: Vec<Cow<'static, str>> = keys(location.get("features").unwrap_or(&Value::Null))
        .map(|feature| Cow::Owned(feature.to_ascii_uppercase()))
        .collect();

    if !features.is_empty() {
        prop.params.push(IcalParam::Feature(features));
    }

    keyed(prop, key)
}

/// A Participant as an `ATTENDEE` or `ORGANIZER` property, or as the
/// `PARTICIPANT` component it came from when it owns a hatch of its own.
fn participant(
    hatch: Option<&Map<String, Value>>,
    component: &str,
    key: &str,
    participant: &Value,
) -> Result<IcalProp<'static>, IcalComponent<'static>> {
    let address = participant
        .get("sendTo")
        .and_then(|to| to.get("imip"))
        .and_then(Value::as_str)
        .unwrap_or_default();

    if participant.get("iCalendar").is_some() {
        return Err(vparticipant(participant, address));
    }

    let owner = participant
        .get("roles")
        .and_then(Value::as_object)
        .is_some_and(|roles| roles.contains_key("owner"));

    // NOTE: An owning participant is the ORGANIZER, and that is where the export
    // recorded its leftovers: under the reply address it also wrote, not under
    // the participant.
    let (pointer, fallback) = match owner {
        true => ("replyTo/imip".to_owned(), IcalPropKind::Organizer),
        false => (format!("participants/{key}"), IcalPropKind::Attendee),
    };

    let mut prop = text_prop(
        named(hatch, component, &pointer, fallback),
        address.to_owned(),
    );
    prop.value = IcalValue::CalAddress(IcalCalAddress(Cow::Owned(address.to_owned())));

    let scalars = [
        ("name", 0usize),
        ("email", 1),
        ("language", 2),
        ("participationStatus", 3),
        ("kind", 4),
        ("scheduleAgent", 5),
        ("scheduleStatus", 6),
    ];

    for (member, slot) in scalars {
        let Some(text) = participant.get(member).and_then(Value::as_str) else {
            continue;
        };

        let upper = Cow::Owned(text.to_ascii_uppercase());

        prop.params.push(match slot {
            0 => IcalParam::Cn(Cow::Owned(text.to_owned())),
            1 => IcalParam::Email(Cow::Owned(text.to_owned())),
            2 => IcalParam::Language(Cow::Owned(text.to_owned())),
            3 => IcalParam::PartStat(upper),
            4 => IcalParam::CuType(match text {
                "location" => Cow::Borrowed("ROOM"),
                _ => upper,
            }),
            5 => IcalParam::ScheduleAgent(upper),
            _ => IcalParam::ScheduleStatus(Cow::Owned(text.to_owned())),
        });
    }

    let flags = [("expectReply", false), ("scheduleForceSend", true)];

    for (member, forced) in flags {
        let Some(flag) = participant.get(member).and_then(Value::as_bool) else {
            continue;
        };

        let text = match flag {
            true => Cow::Borrowed("TRUE"),
            false => Cow::Borrowed("FALSE"),
        };

        prop.params.push(match forced {
            true => IcalParam::ScheduleForceSend(text),
            false => IcalParam::Rsvp(text),
        });
    }

    let roles: Vec<String> = keys(participant.get("roles").unwrap_or(&Value::Null)).collect();

    if let Some(role) = role(&roles) {
        prop.params.push(IcalParam::Role(Cow::Owned(role)));
    }

    let sets = [
        ("delegatedFrom", 0usize),
        ("delegatedTo", 1),
        ("memberOf", 2),
    ];

    for (member, slot) in sets {
        let addresses: Vec<Cow<'static, str>> =
            keys(participant.get(member).unwrap_or(&Value::Null))
                .map(Cow::Owned)
                .collect();

        if addresses.is_empty() {
            continue;
        }

        prop.params.push(match slot {
            0 => IcalParam::DelegatedFrom(addresses),
            1 => IcalParam::DelegatedTo(addresses),
            _ => IcalParam::Member(addresses),
        });
    }

    Ok(keyed(prop, key))
}

/// A Participant that carries its own escape hatch, as a `PARTICIPANT`
/// component.
fn vparticipant(participant: &Value, address: &str) -> IcalComponent<'static> {
    let hatch = participant.as_object().and_then(hatch_of);

    let mut props = vec![plain(IcalPropKind::CalendarAddress, address.to_owned())];

    if let Some(name) = participant.get("name").and_then(Value::as_str) {
        props.push(plain(IcalPropKind::Summary, name.to_owned()));
    }

    if let Some(description) = participant.get("description").and_then(Value::as_str) {
        props.push(plain(IcalPropKind::Description, description.to_owned()));
    }

    for role in keys(participant.get("roles").unwrap_or(&Value::Null)) {
        props.push(plain(
            IcalPropKind::ParticipantType,
            role.to_ascii_uppercase(),
        ));
    }

    props.extend(
        kept_props(hatch, IcalVersion::V2_0)
            .into_iter()
            .map(IcalProp::into_owned),
    );

    IcalComponent {
        name: IcalComponentName::Kind(IcalComponentKind::Participant),
        props,
        components: kept_components(hatch, IcalVersion::V2_0)
            .into_iter()
            .map(IcalComponent::into_owned)
            .collect(),
    }
}

/// An Alert as the `VALARM` it came from.
fn alarm(key: &str, alert: &Value) -> IcalComponent<'static> {
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
fn trigger_prop(trigger: &Value) -> IcalProp<'static> {
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

/// A RecurrenceRule object back in the `RECUR` spelling.
fn rule_from_json(rule: &Value, zone: Option<&str>) -> String {
    let Some(object) = rule.as_object() else {
        return rule.as_str().unwrap_or_default().to_owned();
    };

    let mut parts: Vec<String> = Vec::new();

    for name in RULE_ORDER {
        let Some(value) = object.get(rule_member(name).unwrap_or(name)) else {
            continue;
        };

        let text = match (name, value) {
            ("UNTIL", Value::String(until)) => {
                let basic = basic(until);

                // NOTE: RFC 5545 states UNTIL in UTC whenever DTSTART is; a
                // floating or UTC object is the only case this can restore
                // exactly, which is the one normalisation the mapping makes.
                match zone.is_none() || zone == Some("Etc/UTC") {
                    true => format!("{basic}Z"),
                    false => basic,
                }
            }
            ("BYDAY", Value::Array(days)) => days.iter().map(weekday).collect::<Vec<_>>().join(","),
            (_, Value::Array(items)) => items.iter().map(scalar).collect::<Vec<_>>().join(","),
            ("FREQ" | "WKST" | "RSCALE" | "SKIP", Value::String(text)) => text.to_ascii_uppercase(),
            (_, value) => scalar(value),
        };

        parts.push(format!("{name}={text}"));
    }

    parts.join(";")
}

/// The `RECUR` parts in the order RFC 5545 3.3.10 states them.
const RULE_ORDER: [&str; 16] = [
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

/// The RecurrenceRule member a `RECUR` part is held in.
fn rule_member(part: &str) -> Option<&'static str> {
    let member = match part {
        "FREQ" => "frequency",
        "UNTIL" => "until",
        "COUNT" => "count",
        "INTERVAL" => "interval",
        "BYSECOND" => "bySecond",
        "BYMINUTE" => "byMinute",
        "BYHOUR" => "byHour",
        "BYDAY" => "byDay",
        "BYMONTHDAY" => "byMonthDay",
        "BYYEARDAY" => "byYearDay",
        "BYWEEKNO" => "byWeekNo",
        "BYMONTH" => "byMonth",
        "BYSETPOS" => "bySetPosition",
        "WKST" => "firstDayOfWeek",
        "RSCALE" => "rscale",
        "SKIP" => "skip",
        _ => return None,
    };

    Some(member)
}

/// One NDay object back in the `BYDAY` spelling.
fn weekday(day: &Value) -> String {
    let name = day
        .get("day")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_uppercase();

    match day.get("nthOfPeriod").and_then(Value::as_i64) {
        Some(nth) => format!("{nth}{name}"),
        None => name,
    }
}

/// The `ROLE` one set of JSCalendar roles states, if iCalendar has a word for
/// it (draft 2.3.4).
fn role(roles: &[String]) -> Option<String> {
    let has = |role: &str| roles.iter().any(|held| held == role);

    let role = match (has("chair"), has("optional"), has("informational")) {
        (true, _, _) => "CHAIR",
        (_, true, _) => "OPT-PARTICIPANT",
        (_, _, true) => "NON-PARTICIPANT",
        // NOTE: `attendee` alone is the default role, which iCalendar leaves
        // unwritten; `owner` is carried by ORGANIZER rather than by ROLE.
        _ => return None,
    };

    Some(role.to_owned())
}

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
fn text_prop(record: IcalConverted<'_>, text: String) -> IcalProp<'static> {
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
fn plain(kind: IcalPropKind, text: String) -> IcalProp<'static> {
    IcalProp {
        name: IcalPropName::Kind(kind),
        params: Vec::new(),
        value: IcalValue::Text(IcalText(Cow::Owned(text))),
    }
}

/// A temporal property, as a `DATE` when the object is shown without a time
/// and a `DATE-TIME` otherwise.
fn temporal(
    record: IcalConverted<'_>,
    text: &str,
    zone: Option<&str>,
    date_only: bool,
) -> IcalProp<'static> {
    let mut prop = text_prop(record, basic(text));
    let basic = match &prop.value {
        IcalValue::Text(text) => text.0.clone(),
        _ => Cow::Borrowed(""),
    };

    let declared = prop
        .params
        .iter()
        .any(|param| matches!(param, IcalParam::Value(_)));

    let date = date_only
        || (declared
            && prop.params.iter().any(|param| {
                matches!(param, IcalParam::Value(slot) if slot.eq_ignore_ascii_case("DATE"))
            }));

    match date {
        true => {
            let day = basic.split('T').next().unwrap_or_default().to_owned();
            prop.value = IcalValue::Date(IcalDate(Cow::Owned(day)));

            if !declared {
                prop.params.push(IcalParam::Value(Cow::Borrowed("DATE")));
            }
        }
        // NOTE: A UTC date-time says its zone in its own Z; no zone at all is
        // floating time (RFC 8984 4.7.1), a value with neither a Z nor a TZID.
        false if zone == Some("Etc/UTC") => {
            prop.value = IcalValue::DateTime(IcalDateTime(Cow::Owned(format!("{basic}Z"))))
        }
        false => prop.value = IcalValue::DateTime(IcalDateTime(basic)),
    }

    // NOTE: RFC 5545 gives a DATE no time zone to be in, but a calendar that
    // wrote one on a date is where the object's own zone came from, so it goes
    // back where it was found.
    if let Some(zone) = zone.filter(|zone| *zone != "Etc/UTC") {
        prop.params
            .push(IcalParam::TzId(Cow::Owned(zone.to_owned())));
    }

    prop
}

/// A `RDATE`, `EXDATE` or `RECURRENCE-ID` naming one occurrence, in the time
/// zone of the series it belongs to.
fn occurrence(
    kind: IcalPropKind,
    id: &str,
    zone: Option<&str>,
    date_only: bool,
) -> IcalProp<'static> {
    let record = IcalConverted {
        name: Cow::Owned((*kind).to_owned()),
        value_type: None,
        params: Vec::new(),
    };

    temporal(record, id, zone, date_only)
}

/// The date-time a start and a duration end at, in the JSCalendar spelling.
fn ends(start: &str, span: &str) -> Option<String> {
    let start = IcalRecurDateTime::parse(&basic(start)).ok()?;
    let end = IcalRecurDateTime::from_seconds(start.seconds() + seconds(span)?);

    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        end.year, end.month, end.day, end.hour, end.minute, end.second
    ))
}

/// An RFC 5545 duration in seconds, `None` when it is not one.
fn seconds(span: &str) -> Option<i64> {
    let (sign, span) = match span.strip_prefix('-') {
        Some(span) => (-1, span),
        None => (1, span.strip_prefix('+').unwrap_or(span)),
    };

    let mut total: i64 = 0;
    let mut amount = String::new();

    for character in span.strip_prefix('P')?.chars() {
        if character.is_ascii_digit() {
            amount.push(character);
            continue;
        }

        // NOTE: The T only separates the date part from the time part; every
        // other letter closes the number before it.
        if character == 'T' {
            continue;
        }

        let unit = match character {
            'W' => 604_800,
            'D' => 86_400,
            'H' => 3_600,
            'M' => 60,
            'S' => 1,
            _ => return None,
        };

        total += amount.parse::<i64>().ok()? * unit;
        amount.clear();
    }

    Some(sign * total)
}

/// Tag a property with the JSCalendar key it converts back from, so the key
/// survives a further conversion (draft 4.2.1).
fn keyed(mut prop: IcalProp<'static>, key: &str) -> IcalProp<'static> {
    prop.params.push(IcalParam::Unknown {
        name: Cow::Owned("JSID".to_owned()),
        values: vec![Cow::Owned(key.to_owned())],
    });

    prop
}

/// The record a hatch holds for a member, or the property the mapping assumes.
fn named<'a>(
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
fn keys(value: &Value) -> impl Iterator<Item = String> + '_ {
    value
        .as_object()
        .into_iter()
        .flat_map(|set| set.keys().map(String::clone))
}

/// A JSON scalar as the text iCalendar writes.
fn scalar(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// A JSCalendar date-time back in the iCalendar basic spelling.
fn basic(text: &str) -> String {
    datetime_from_json(text).unwrap_or_else(|| text.to_owned())
}
