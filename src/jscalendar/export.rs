//! # Export
//!
//! The decoded calendar as a JSCalendar `Group`.
//!
//! A `VCALENDAR` converts to a Group (RFC 8984 5.3), its `VEVENT`s to Events
//! (2.1) and its `VTODO`s to Tasks (2.2). Everything else, at every level,
//! goes to the escape hatch rather than being dropped.

mod alert;
mod descriptive;
mod participant;
mod place;
mod temporal;

use alloc::{
    borrow::{Cow, ToOwned},
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value, json};

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    ical::Ical,
    jscalendar::{export::temporal::utc, hatch::IcalHatch, patch},
    param::{IcalParam, IcalParamKind},
    prop::{IcalProp, IcalPropKind, IcalPropName},
    value::IcalValue,
    value::binary::IcalBinary,
    version::IcalVersion,
};

/// The calendar as a JSCalendar `Group` value.
pub(crate) fn group(ical: &Ical<'_>) -> Value {
    let mut group = Map::new();
    group.insert("@type".to_owned(), Value::String("Group".to_owned()));

    let mut hatch = IcalHatch::new("vcalendar");

    // NOTE: VERSION is hoisted out of the model (see `Ical::props`), so the
    // hatch is the only place it can survive. Only a calendar that is not
    // iCalendar 2.0 needs it kept: 2.0 is what a Group with no version means,
    // and writing it back would make a Group that never was iCalendar grow one.
    if ical.version != IcalVersion::V2_0 {
        hatch.keep_value(json!(["version", {}, "text", (*ical.version).to_string()]));
    }

    let mut jsprops = Map::new();

    for prop in &ical.props {
        calendar_prop(&mut group, &mut hatch, &mut jsprops, prop);
    }

    let converted: Vec<Option<Entry>> = ical
        .components
        .iter()
        .map(|component| entry_kind(component).map(|task| entry(component, task)))
        .collect();

    // NOTE: An overriding component folds into the series it overrides, so its
    // main has to be located before any object is handed out (draft 2.1.2).
    let mains = mains(&converted);
    let bases: Vec<Option<Map<String, Value>>> = converted
        .iter()
        .map(|entry| entry.as_ref().map(|entry| entry.object.clone()))
        .collect();

    let mut objects: Vec<Option<Map<String, Value>>> = bases.clone();

    for (index, main) in mains.iter().enumerate() {
        let (Some(main), Some(over)) = (main, converted[index].as_ref()) else {
            continue;
        };

        let (Some(base), Some(id)) = (bases[*main].as_ref(), over.recurrence_id.as_ref()) else {
            continue;
        };

        let overrides = objects[*main]
            .as_mut()
            .expect("a main entry that was converted")
            .entry("recurrenceOverrides")
            .or_insert_with(|| Value::Object(Map::new()));

        if let Some(overrides) = overrides.as_object_mut() {
            let mut merged = overrides
                .get(id)
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();

            // NOTE: An excluded occurrence does not happen, so an override of
            // it describes nothing; RFC 8984 4.3.5 forbids the patch that would
            // say both. A calendar carrying an EXDATE and an overriding
            // component for one date is contradicting itself, and the exclusion
            // wins.
            if merged.get("excluded") == Some(&Value::Bool(true)) {
                continue;
            }

            merged.extend(patch::diff(base, &over.object));
            overrides.insert(id.clone(), Value::Object(merged));
        }
    }

    let mut entries = Vec::new();

    for (index, component) in ical.components.iter().enumerate() {
        match (objects[index].take(), mains[index]) {
            // NOTE: An override that folded is already inside its series.
            (Some(_), Some(_)) => continue,
            (Some(object), None) => entries.push(Value::Object(object)),
            (None, _) => hatch.keep_component(component),
        }
    }

    if !entries.is_empty() {
        group.insert("entries".to_owned(), Value::Array(entries));
    }

    if let Some(hatch) = hatch.into_value() {
        group.insert("iCalendar".to_owned(), hatch);
    }

    graft(&mut group, jsprops);

    Value::Object(group)
}

/// One converted `VEVENT` or `VTODO`, with what folding a recurrence override
/// needs to know about it: the `UID` two components share, the recurrence id
/// an override carries, and whether either is a series or a task.
struct Entry {
    object: Map<String, Value>,
    uid: String,
    recurrence_id: Option<String>,
    series: bool,
    task: bool,
}

/// Whether a component converts to an entry, and to which kind: `Some(true)`
/// for a Task, `Some(false)` for an Event, nothing for anything else.
fn entry_kind(component: &IcalComponent<'_>) -> Option<bool> {
    match component.name {
        IcalComponentName::Kind(IcalComponentKind::VEvent) => Some(false),
        IcalComponentName::Kind(IcalComponentKind::VTodo) => Some(true),
        _ => None,
    }
}

/// The main component each override belongs to, by index.
///
/// A component overrides a series when it carries a `RECURRENCE-ID` and
/// another component of the same kind carries the same `UID`, no
/// `RECURRENCE-ID` and an `RRULE` (draft 2.1.2). Without such a main it is a
/// stand-alone instance, converting to an entry of its own.
fn mains(converted: &[Option<Entry>]) -> Vec<Option<usize>> {
    converted
        .iter()
        .map(|entry| {
            let over = entry.as_ref()?;
            over.recurrence_id.as_ref()?;

            // NOTE: A main carries no RECURRENCE-ID, so it can never be the
            // override itself.
            converted.iter().position(|main| {
                main.as_ref().is_some_and(|main| {
                    main.task == over.task
                        && main.uid == over.uid
                        && main.recurrence_id.is_none()
                        && main.series
                })
            })
        })
        .collect()
}

/// One calendar-level property onto the Group.
fn calendar_prop(
    group: &mut Map<String, Value>,
    hatch: &mut IcalHatch,
    jsprops: &mut Map<String, Value>,
    prop: &IcalProp<'_>,
) {
    let IcalPropName::Kind(kind) = &prop.name else {
        match jsprop(prop) {
            Some((pointer, value)) => {
                jsprops.insert(pointer, value);
            }
            None => hatch.keep(prop),
        }

        return;
    };

    let (pointer, value) = match kind {
        IcalPropKind::Uid => ("uid", text(prop).map(Value::String)),
        IcalPropKind::ProdId => ("prodId", text(prop).map(Value::String)),
        IcalPropKind::Name => ("title", text(prop).map(Value::String)),
        IcalPropKind::Description => ("description", text(prop).map(Value::String)),
        IcalPropKind::Source => ("source", text(prop).map(Value::String)),
        IcalPropKind::Color => ("color", text(prop).map(Value::String)),
        IcalPropKind::Method => (
            "method",
            text(prop).map(|method| Value::String(method.to_lowercase())),
        ),
        IcalPropKind::Created => ("created", utc(prop).map(Value::String)),
        IcalPropKind::LastModified => ("updated", utc(prop).map(Value::String)),
        // NOTE: Keywords are a set, and a calendar may carry several CATEGORIES
        // lines, so they accumulate rather than replace one another.
        IcalPropKind::Categories => {
            let keywords = group
                .entry("keywords")
                .or_insert_with(|| Value::Object(Map::new()));

            if let Some(keywords) = keywords.as_object_mut() {
                keywords.extend(set(list(prop)));
            }

            return hatch.note("keywords", prop, &[]);
        }
        _ => return hatch.keep(prop),
    };

    let Some(value) = value else {
        return hatch.keep(prop);
    };

    group.insert(pointer.to_owned(), value);
    hatch.note(pointer, prop, &[]);
}

/// One `VEVENT` or `VTODO` as an Event or Task object.
fn entry(component: &IcalComponent<'_>, task: bool) -> Entry {
    let mut builder = Builder {
        object: Map::new(),
        hatch: IcalHatch::new(&component.name),
        links: Map::new(),
        locations: Map::new(),
        virtual_locations: Map::new(),
        participants: Map::new(),
        alerts: Map::new(),
        related_to: Map::new(),
        reply_to: Map::new(),
        keywords: Map::new(),
        categories: Map::new(),
        request_status: Vec::new(),
        rules: Vec::new(),
        excluded_rules: Vec::new(),
        overrides: Map::new(),
        uid: String::new(),
        recurrence_id: None,
        series: false,
        task,
        zone: None,
        jsprops: Map::new(),
    };

    builder.object.insert(
        "@type".to_owned(),
        Value::String(match task {
            true => "Task".to_owned(),
            false => "Event".to_owned(),
        }),
    );

    // NOTE: A DTEND is a span from the start, so it cannot convert until the
    // start has, and iCalendar does not say in which order the two are written.
    let end = |prop: &IcalProp<'_>| matches!(prop.name, IcalPropName::Kind(IcalPropKind::DtEnd));

    for prop in component.props.iter().filter(|prop| !end(prop)) {
        builder.prop(prop);
    }

    for prop in component.props.iter().filter(|prop| end(prop)) {
        builder.prop(prop);
    }

    for child in &component.components {
        builder.component(child);
    }

    builder.finish()
}

/// An Event or Task under construction, one collection per JSCalendar member
/// that holds several converted elements.
pub(super) struct Builder {
    object: Map<String, Value>,
    hatch: IcalHatch,
    links: Map<String, Value>,
    locations: Map<String, Value>,
    virtual_locations: Map<String, Value>,
    participants: Map<String, Value>,
    alerts: Map<String, Value>,
    related_to: Map<String, Value>,
    reply_to: Map<String, Value>,
    keywords: Map<String, Value>,
    categories: Map<String, Value>,
    request_status: Vec<Value>,
    rules: Vec<Value>,
    excluded_rules: Vec<Value>,
    overrides: Map<String, Value>,
    uid: String,
    recurrence_id: Option<String>,
    series: bool,
    task: bool,
    /// The time zone `DTSTART` referenced, which every other temporal member
    /// is read against.
    zone: Option<String>,
    /// The members carried in JSPROP properties, grafted on once everything
    /// else has converted (draft 4.1.2).
    jsprops: Map<String, Value>,
}

impl Builder {
    /// Convert one property, keeping it whole when nothing holds it.
    fn prop(&mut self, prop: &IcalProp<'_>) {
        let IcalPropName::Kind(kind) = &prop.name else {
            match jsprop(prop) {
                Some((pointer, value)) => {
                    self.jsprops.insert(pointer, value);
                }
                None => self.hatch.keep(prop),
            }

            return;
        };

        match kind {
            IcalPropKind::Uid => {
                self.uid = text(prop).unwrap_or_default();
                self.member("uid", Value::String(self.uid.clone()), prop, &[]);
            }
            IcalPropKind::Summary => self.scalar("title", prop),
            IcalPropKind::Description => self.scalar("description", prop),
            IcalPropKind::StyledDescription => self.styled_description(prop),
            IcalPropKind::Color => self.scalar("color", prop),
            IcalPropKind::Created => self.timestamp("created", prop),
            IcalPropKind::DtStamp | IcalPropKind::LastModified => self.timestamp("updated", prop),
            IcalPropKind::Completed => self.timestamp("progressUpdated", prop),
            IcalPropKind::Sequence => self.number("sequence", prop),
            IcalPropKind::Priority => self.number("priority", prop),
            IcalPropKind::PercentComplete => self.number("percentComplete", prop),
            IcalPropKind::Class => self.privacy(prop),
            IcalPropKind::Status => self.status(prop),
            IcalPropKind::Transp => self.free_busy(prop),
            IcalPropKind::Categories => self.keywords(prop),
            IcalPropKind::Concept => self.concept(prop),
            IcalPropKind::DtStart => self.start(prop),
            IcalPropKind::Due => self.due(prop),
            IcalPropKind::Duration => self.scalar("duration", prop),
            IcalPropKind::DtEnd => self.end(prop),
            IcalPropKind::RRule => self.rule(prop, false),
            IcalPropKind::ExRule => self.rule(prop, true),
            IcalPropKind::RDate => self.dates(prop, false),
            IcalPropKind::ExDate => self.dates(prop, true),
            IcalPropKind::RecurrenceId => self.recurrence_id(prop),
            IcalPropKind::RelatedTo => self.related(prop),
            IcalPropKind::Location => self.location(prop),
            IcalPropKind::Geo => self.geo(prop),
            IcalPropKind::Conference => self.conference(prop),
            IcalPropKind::Attach => self.link(prop, None),
            IcalPropKind::Image => self.link(prop, Some("icon")),
            IcalPropKind::Link => self.link(prop, param(prop, IcalParamKind::LinkRel).as_deref()),
            IcalPropKind::Organizer => self.organizer(prop),
            IcalPropKind::Attendee => self.attendee(prop),
            IcalPropKind::RequestStatus => self.request_status(prop),
            IcalPropKind::Method => self.method(prop),
            _ => self.hatch.keep(prop),
        }
    }

    /// Convert one subcomponent, keeping it whole when nothing holds it.
    fn component(&mut self, component: &IcalComponent<'_>) {
        match component.name {
            IcalComponentName::Kind(IcalComponentKind::VAlarm) => self.alarm(component),
            IcalComponentName::Kind(IcalComponentKind::Participant) => self.participant(component),
            IcalComponentName::Kind(IcalComponentKind::VLocation) => self.vlocation(component),
            _ => self.hatch.keep_component(component),
        }
    }

    /// Set a member and record what it came from.
    fn member(
        &mut self,
        pointer: &str,
        value: Value,
        prop: &IcalProp<'_>,
        consumed: &[IcalParamKind],
    ) {
        self.object.insert(pointer.to_owned(), value);
        self.hatch.note(pointer, prop, consumed);
    }

    /// A property whose text is the member.
    fn scalar(&mut self, pointer: &str, prop: &IcalProp<'_>) {
        match text(prop) {
            Some(text) => self.member(pointer, Value::String(text), prop, &[]),
            None => self.hatch.keep(prop),
        }
    }

    /// A property whose value is a UTC timestamp.
    fn timestamp(&mut self, pointer: &str, prop: &IcalProp<'_>) {
        match utc(prop) {
            Some(at) => self.member(pointer, Value::String(at), prop, &[]),
            None => self.hatch.keep(prop),
        }
    }

    /// A property whose value is a number.
    fn number(&mut self, pointer: &str, prop: &IcalProp<'_>) {
        match text(prop).and_then(|text| text.parse::<i64>().ok()) {
            Some(number) => self.member(pointer, Value::Number(number.into()), prop, &[]),
            None => self.hatch.keep(prop),
        }
    }

    /// The finished entry, with every non-empty collection folded in.
    fn finish(mut self) -> Entry {
        let collections = [
            ("keywords", self.keywords),
            ("categories", self.categories),
            ("links", self.links),
            ("locations", self.locations),
            ("virtualLocations", self.virtual_locations),
            ("participants", self.participants),
            ("alerts", self.alerts),
            ("relatedTo", self.related_to),
            ("replyTo", self.reply_to),
            ("recurrenceOverrides", self.overrides),
        ];

        for (member, collection) in collections {
            if !collection.is_empty() {
                self.object
                    .insert(member.to_owned(), Value::Object(collection));
            }
        }

        let lists = [
            ("recurrenceRules", self.rules),
            ("excludedRecurrenceRules", self.excluded_rules),
            ("requestStatus", self.request_status),
        ];

        for (member, list) in lists {
            if !list.is_empty() {
                self.object.insert(member.to_owned(), Value::Array(list));
            }
        }

        if let Some(hatch) = self.hatch.into_value() {
            self.object.insert("iCalendar".to_owned(), hatch);
        }

        graft(&mut self.object, self.jsprops);

        Entry {
            object: self.object,
            uid: self.uid,
            recurrence_id: self.recurrence_id,
            series: self.series,
            task: self.task,
        }
    }
}

/// The parameters an `ATTENDEE` or `ORGANIZER` conversion consumes, so only
/// what is genuinely left over is recorded.
pub(super) const PARTICIPANT_PARAMS: &[IcalParamKind] = &[
    IcalParamKind::Cn,
    IcalParamKind::CuType,
    IcalParamKind::DelegatedFrom,
    IcalParamKind::DelegatedTo,
    IcalParamKind::Email,
    IcalParamKind::Language,
    IcalParamKind::Member,
    IcalParamKind::PartStat,
    IcalParamKind::Role,
    IcalParamKind::Rsvp,
    IcalParamKind::ScheduleAgent,
    IcalParamKind::ScheduleForceSend,
    IcalParamKind::ScheduleStatus,
];

/// The key an element takes in the collection it joins: what its `JSID` says,
/// else its position, which is stable for as long as the source is (draft
/// 2.1.3).
pub(super) fn key(collection: &Map<String, Value>, prop: &IcalProp<'_>) -> String {
    jsid_param(prop).unwrap_or_else(|| (collection.len() + 1).to_string())
}

/// The same, for an element that is a whole component: its `JSID` property,
/// else its `UID`, else its position.
pub(super) fn component_key(
    collection: &Map<String, Value>,
    component: &IcalComponent<'_>,
) -> String {
    let jsid = component.props.iter().find_map(|prop| match &prop.name {
        IcalPropName::Unknown(name) if name.eq_ignore_ascii_case("JSID") => text(prop),
        _ => None,
    });

    let uid = component.props.iter().find_map(|prop| match &prop.name {
        IcalPropName::Kind(IcalPropKind::Uid) => text(prop),
        _ => None,
    });

    jsid.or(uid)
        .unwrap_or_else(|| (collection.len() + 1).to_string())
}

/// The `JSID` parameter a property carries, if any.
fn jsid_param(prop: &IcalProp<'_>) -> Option<String> {
    prop.params.iter().find_map(|param| match param {
        IcalParam::Unknown { name, values } if name.eq_ignore_ascii_case("JSID") => {
            values.first().map(|value| value.to_string())
        }
        _ => None,
    })
}

/// The member a `JSPROP` property carries, and where it belongs: its `JSPTR`
/// parameter is the pointer, its value is the member's JSON (draft 4.1.2).
fn jsprop(prop: &IcalProp<'_>) -> Option<(String, Value)> {
    if !prop.name.eq_ignore_ascii_case("JSPROP") {
        return None;
    }

    let pointer = prop.params.iter().find_map(|param| match param {
        IcalParam::Unknown { name, values } if name.eq_ignore_ascii_case("JSPTR") => values.first(),
        _ => None,
    })?;

    let value = serde_json::from_str(&text(prop)?).ok()?;

    Some((pointer.to_string(), value))
}

/// Graft the members `JSPROP` properties carried onto a converted object.
///
/// Only a member the conversion did not already write is set, since a pointer
/// onto an existing one is to be ignored (draft 4.1.2).
fn graft(object: &mut Map<String, Value>, jsprops: Map<String, Value>) {
    let fresh: Map<String, Value> = jsprops
        .into_iter()
        .filter(|(pointer, _)| {
            let head = pointer.split('/').next().unwrap_or(pointer);
            !object.contains_key(head)
        })
        .collect();

    patch::apply(object, &fresh);
}

/// A list of strings as a JSCalendar set: the strings as keys, all true.
pub(super) fn set(values: Vec<String>) -> Map<String, Value> {
    values
        .into_iter()
        .filter(|value| !value.is_empty())
        .map(|value| (value, Value::Bool(true)))
        .collect()
}

/// The scalar text of a property's value, for the value shapes that hold one.
pub(super) fn text(prop: &IcalProp<'_>) -> Option<String> {
    let text = match &prop.value {
        IcalValue::Binary(IcalBinary::Uri(value) | IcalBinary::Base64(value)) => value.as_ref(),
        IcalValue::Boolean(value) => value.0.as_ref(),
        IcalValue::CalAddress(value) => value.0.as_ref(),
        IcalValue::Date(value) => value.0.as_ref(),
        IcalValue::DateTime(value) => value.0.as_ref(),
        IcalValue::Duration(value) => value.0.as_ref(),
        IcalValue::Float(value) => value.0.as_ref(),
        IcalValue::Integer(value) => value.0.as_ref(),
        IcalValue::Period(value) => value.0.as_ref(),
        IcalValue::Recur(value) => value.0.as_ref(),
        IcalValue::Text(value) => value.0.as_ref(),
        IcalValue::Time(value) => value.0.as_ref(),
        IcalValue::Uri(value) => value.0.as_ref(),
        IcalValue::UtcOffset(value) => value.0.as_ref(),
        IcalValue::TextList(list) => list.0.first()?.as_ref(),
        IcalValue::DateTimeList(list) => list.0.first()?.as_ref(),
        IcalValue::Geo(_) | IcalValue::RequestStatus(_) => return None,
        IcalValue::Unknown(value) => value.components.first()?.first()?.as_ref(),
    };

    Some(text.to_owned())
}

/// Every item of a property's value, for the list shapes; one item for the
/// rest.
pub(super) fn list(prop: &IcalProp<'_>) -> Vec<String> {
    match &prop.value {
        IcalValue::TextList(list) => list.0.iter().map(|item| item.to_string()).collect(),
        IcalValue::DateTimeList(list) => list.0.iter().map(|item| item.to_string()).collect(),
        _ => text(prop).into_iter().collect(),
    }
}

/// The scalar text of a property's first parameter of this kind.
pub(super) fn param<'a>(prop: &'a IcalProp<'a>, kind: IcalParamKind) -> Option<Cow<'a, str>> {
    prop.params
        .iter()
        .find(|param| param.kind() == Some(kind))
        .map(IcalParam::scalar)
        .filter(|text| !text.is_empty())
}

/// Every value of a property's parameters of this kind.
pub(super) fn values(prop: &IcalProp<'_>, kind: IcalParamKind) -> Vec<String> {
    prop.params
        .iter()
        .filter(|param| param.kind() == Some(kind))
        .flat_map(|param| match param {
            IcalParam::DelegatedFrom(values)
            | IcalParam::DelegatedTo(values)
            | IcalParam::Member(values)
            | IcalParam::Feature(values) => values.iter().map(|value| value.to_string()).collect(),
            other => vec![other.scalar().into_owned()],
        })
        .filter(|value| !value.is_empty())
        .collect()
}
