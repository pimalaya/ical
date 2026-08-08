//! # Export
//!
//! The decoded calendar as a JSCalendar `Group`.
//!
//! A `VCALENDAR` converts to a Group (RFC 8984 5.3), its `VEVENT`s to Events
//! (2.1) and its `VTODO`s to Tasks (2.2). Everything else, at every level,
//! goes to the [`IcalHatch`](crate::jscalendar::hatch::IcalHatch) rather than
//! being dropped.

use alloc::{
    borrow::{Cow, ToOwned},
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value, json};

use crate::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    ical::Ical,
    jcal::{datetime_to_json, param_scalar},
    jscalendar::{hatch::IcalHatch, patch},
    param::{IcalParam, IcalParamKind},
    prop::{IcalProp, IcalPropKind, IcalPropName},
    recur::IcalRecurDateTime,
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

            // NOTE: An excluded occurrence does not happen, so an override of it
            // describes nothing; RFC 8984 4.3.5 forbids the patch that would say
            // both. A calendar carrying an EXDATE and an overriding component
            // for one date is contradicting itself, and the exclusion wins.
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
/// needs to know about it.
struct Entry {
    /// The Event or Task object.
    object: Map<String, Value>,
    /// The `UID` the component carries, the identity two components share.
    uid: String,
    /// The recurrence id, when this component overrides one instance.
    recurrence_id: Option<String>,
    /// Whether the component carries an `RRULE`, which is what makes it a
    /// series others may override.
    series: bool,
    /// Whether this is a `VTODO` rather than a `VEVENT`.
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
/// stand-alone instance, and converts to an entry of its own.
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
struct Builder {
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

    /// `STYLED-DESCRIPTION` carries the description with its media type.
    fn styled_description(&mut self, prop: &IcalProp<'_>) {
        let Some(text) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let media = param(prop, IcalParamKind::FmtType).unwrap_or(Cow::Borrowed("text/html"));

        self.object.insert(
            "descriptionContentType".to_owned(),
            Value::String(media.into_owned()),
        );
        self.member(
            "description",
            Value::String(text),
            prop,
            &[IcalParamKind::FmtType],
        );
    }

    /// `CLASS` is the privacy of the object, with `CONFIDENTIAL` spelled
    /// `secret` (RFC 8984 4.4.3).
    fn privacy(&mut self, prop: &IcalProp<'_>) {
        let Some(class) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let privacy = match class.to_ascii_uppercase().as_str() {
            "CONFIDENTIAL" => "secret".to_owned(),
            _ => class.to_lowercase(),
        };

        self.member("privacy", Value::String(privacy), prop, &[]);
    }

    /// `STATUS` is the Event's status, and the Task's progress (draft 2.3.39).
    fn status(&mut self, prop: &IcalProp<'_>) {
        let Some(status) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let pointer = match self.task {
            true => "progress",
            false => "status",
        };

        self.member(pointer, Value::String(status.to_lowercase()), prop, &[]);
    }

    /// `TRANSP` is the free/busy status (RFC 8984 4.4.2).
    fn free_busy(&mut self, prop: &IcalProp<'_>) {
        let Some(transp) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let status = match transp.to_ascii_uppercase().as_str() {
            "TRANSPARENT" => "free",
            "OPAQUE" => "busy",
            _ => return self.hatch.keep(prop),
        };

        self.member(
            "freeBusyStatus",
            Value::String(status.to_owned()),
            prop,
            &[],
        );
    }

    /// `CATEGORIES` are the keywords, a set rather than a list.
    fn keywords(&mut self, prop: &IcalProp<'_>) {
        self.keywords.extend(set(list(prop)));
        self.hatch.note("keywords", prop, &[]);
    }

    /// `CONCEPT` is a categorisation by URI (RFC 9253 8.3).
    fn concept(&mut self, prop: &IcalProp<'_>) {
        match text(prop) {
            Some(concept) => {
                self.categories.insert(concept, Value::Bool(true));
                self.hatch.note("categories", prop, &[]);
            }
            None => self.hatch.keep(prop),
        }
    }

    /// `DTSTART` carries the start, its time zone and, for a date-only value,
    /// that the object is shown without a time (draft 2.3.16).
    fn start(&mut self, prop: &IcalProp<'_>) {
        let Some(start) = local(prop) else {
            return self.hatch.keep(prop);
        };

        self.zone = zone(prop);

        if let Some(zone) = &self.zone {
            self.object
                .insert("timeZone".to_owned(), Value::String(zone.clone()));
        }

        if matches!(prop.value, IcalValue::Date(_)) {
            self.object
                .insert("showWithoutTime".to_owned(), Value::Bool(true));
        }

        self.member("start", Value::String(start), prop, temporal_params(prop));
    }

    /// `DUE` is a Task's due date-time.
    fn due(&mut self, prop: &IcalProp<'_>) {
        let Some(due) = local(prop) else {
            return self.hatch.keep(prop);
        };

        if self.zone.is_none() {
            self.zone = zone(prop);

            if let Some(zone) = &self.zone {
                self.object
                    .insert("timeZone".to_owned(), Value::String(zone.clone()));
            }
        }

        self.member("due", Value::String(due), prop, temporal_params(prop));
    }

    /// `DTEND` is the duration from the start, since JSCalendar states a span
    /// rather than an end (draft 2.3.14).
    fn end(&mut self, prop: &IcalProp<'_>) {
        let start = self.object.get("start").and_then(Value::as_str);
        let (Some(start), Some(end)) = (start, local(prop)) else {
            return self.hatch.keep(prop);
        };

        let Some(span) = span(start, &end) else {
            return self.hatch.keep(prop);
        };

        self.member("duration", Value::String(span), prop, temporal_params(prop));
    }

    /// `RRULE` and `EXRULE` are the recurrence rules (draft 2.3.36).
    fn rule(&mut self, prop: &IcalProp<'_>, excluded: bool) {
        let Some(text) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let pointer = match excluded {
            true => "excludedRecurrenceRules",
            false => "recurrenceRules",
        };

        match excluded {
            true => self.excluded_rules.push(rule_to_json(&text)),
            false => {
                self.series = true;
                self.rules.push(rule_to_json(&text));
            }
        }

        self.hatch.note(pointer, prop, &[]);
    }

    /// `RDATE` adds occurrences and `EXDATE` removes them, both as entries in
    /// `recurrenceOverrides` (draft 2.3.20, 2.3.33).
    fn dates(&mut self, prop: &IcalProp<'_>, excluded: bool) {
        let mut converted = false;

        for item in list(prop) {
            // NOTE: An RDATE may be a period; only its start names an occurrence.
            let start = item.split('/').next().unwrap_or(&item);

            let Some(id) = local_text(start, matches!(prop.value, IcalValue::Date(_))) else {
                continue;
            };

            let patch = match excluded {
                true => json!({"excluded": true}),
                false => json!({}),
            };

            self.overrides.insert(id, patch);
            converted = true;
        }

        // NOTE: Nothing is recorded for these two: which of RDATE and EXDATE an
        // override came from is already said by its `excluded` flag, and a
        // record would only repeat it.
        if !converted {
            self.hatch.keep(prop);
        }
    }

    /// `RECURRENCE-ID` says which instance of a series this component is.
    fn recurrence_id(&mut self, prop: &IcalProp<'_>) {
        let Some(id) = local(prop) else {
            return self.hatch.keep(prop);
        };

        if let Some(zone) = zone(prop) {
            self.object
                .insert("recurrenceIdTimeZone".to_owned(), Value::String(zone));
        }

        self.recurrence_id = Some(id.clone());
        self.member(
            "recurrenceId",
            Value::String(id),
            prop,
            temporal_params(prop),
        );
    }

    /// `RELATED-TO` is a Relation keyed by the other object's `UID`.
    fn related(&mut self, prop: &IcalProp<'_>) {
        let Some(uid) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let relation = param(prop, IcalParamKind::RelType)
            .map(|kind| set(vec![kind.to_lowercase()]))
            .unwrap_or_default();

        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Relation".to_owned()));

        if !relation.is_empty() {
            object.insert("relation".to_owned(), Value::Object(relation));
        }

        let pointer = format!("relatedTo/{uid}");
        self.related_to.insert(uid, Value::Object(object));
        self.hatch.note(&pointer, prop, &[IcalParamKind::RelType]);
    }

    /// `LOCATION` is a named Location (RFC 8984 4.2.5).
    fn location(&mut self, prop: &IcalProp<'_>) {
        let Some(name) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let key = key(&self.locations, prop);
        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Location".to_owned()));
        object.insert("name".to_owned(), Value::String(name));

        let pointer = format!("locations/{key}");
        self.locations.insert(key, Value::Object(object));
        self.hatch.note(&pointer, prop, &[]);
    }

    /// `GEO` is a Location holding only coordinates, as a `geo:` URI.
    fn geo(&mut self, prop: &IcalProp<'_>) {
        let IcalValue::Geo(geo) = &prop.value else {
            return self.hatch.keep(prop);
        };

        let key = key(&self.locations, prop);
        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Location".to_owned()));
        object.insert(
            "coordinates".to_owned(),
            Value::String(format!("geo:{},{}", geo.latitude, geo.longitude)),
        );

        let pointer = format!("locations/{key}/coordinates");
        self.locations.insert(key, Value::Object(object));
        self.hatch.note(&pointer, prop, &[]);
    }

    /// `CONFERENCE` is a VirtualLocation (RFC 8984 4.2.6).
    fn conference(&mut self, prop: &IcalProp<'_>) {
        let Some(uri) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let key = key(&self.virtual_locations, prop);
        let mut object = Map::new();
        object.insert(
            "@type".to_owned(),
            Value::String("VirtualLocation".to_owned()),
        );
        object.insert("uri".to_owned(), Value::String(uri));

        if let Some(label) = param(prop, IcalParamKind::Label) {
            object.insert("name".to_owned(), Value::String(label.into_owned()));
        }

        let features = set(values(prop, IcalParamKind::Feature)
            .iter()
            .map(|feature| feature.to_lowercase())
            .collect());

        if !features.is_empty() {
            object.insert("features".to_owned(), Value::Object(features));
        }

        let pointer = format!("virtualLocations/{key}");
        self.virtual_locations.insert(key, Value::Object(object));
        self.hatch.note(
            &pointer,
            prop,
            &[
                IcalParamKind::Label,
                IcalParamKind::Feature,
                IcalParamKind::Value,
            ],
        );
    }

    /// `ATTACH`, `IMAGE` and `LINK` are Links, told apart by their relation
    /// (RFC 8984 1.4.11).
    fn link(&mut self, prop: &IcalProp<'_>, rel: Option<&str>) {
        let Some(href) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let key = key(&self.links, prop);
        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Link".to_owned()));
        object.insert("href".to_owned(), Value::String(href));

        if let Some(rel) = rel {
            object.insert("rel".to_owned(), Value::String(rel.to_lowercase()));
        }

        if let Some(media) = param(prop, IcalParamKind::FmtType) {
            object.insert("contentType".to_owned(), Value::String(media.into_owned()));
        }

        if let Some(display) = param(prop, IcalParamKind::Display) {
            object.insert("display".to_owned(), Value::String(display.to_lowercase()));
        }

        if let Some(label) = param(prop, IcalParamKind::Label) {
            object.insert("title".to_owned(), Value::String(label.into_owned()));
        }

        let pointer = format!("links/{key}");
        self.links.insert(key, Value::Object(object));
        self.hatch.note(
            &pointer,
            prop,
            &[
                IcalParamKind::FmtType,
                IcalParamKind::Display,
                IcalParamKind::Label,
                IcalParamKind::LinkRel,
            ],
        );
    }

    /// `ORGANIZER` is both the reply address and a participant owning the
    /// object (draft 2.3.29).
    fn organizer(&mut self, prop: &IcalProp<'_>) {
        let Some(address) = text(prop) else {
            return self.hatch.keep(prop);
        };

        self.reply_to
            .insert("imip".to_owned(), Value::String(address.clone()));

        let key = key(&self.participants, prop);
        let mut object = participant(prop, &address);
        object.insert(
            "roles".to_owned(),
            Value::Object(set(vec!["owner".to_owned()])),
        );

        self.participants.insert(key, Value::Object(object));
        self.hatch.note("replyTo/imip", prop, PARTICIPANT_PARAMS);
    }

    /// `ATTENDEE` is a Participant (draft 2.3.4).
    fn attendee(&mut self, prop: &IcalProp<'_>) {
        let Some(address) = text(prop) else {
            return self.hatch.keep(prop);
        };

        let key = key(&self.participants, prop);
        let mut object = participant(prop, &address);

        let roles = set(values(prop, IcalParamKind::Role)
            .iter()
            .flat_map(|role| roles(role))
            .collect());

        object.insert(
            "roles".to_owned(),
            // NOTE: A participant must hold at least one role (RFC 8984 4.4.6).
            Value::Object(match roles.is_empty() {
                true => set(vec!["attendee".to_owned()]),
                false => roles,
            }),
        );

        if self.task
            && let Some(progress) = progress(prop)
        {
            self.object
                .insert("progress".to_owned(), Value::String(progress));
        }

        let pointer = format!("participants/{key}");
        self.participants.insert(key, Value::Object(object));
        self.hatch.note(&pointer, prop, PARTICIPANT_PARAMS);
    }

    /// `REQUEST-STATUS` keeps its wire spelling (RFC 8984 4.4.7).
    fn request_status(&mut self, prop: &IcalProp<'_>) {
        let IcalValue::RequestStatus(status) = &prop.value else {
            return self.hatch.keep(prop);
        };

        let mut text = format!("{};{}", status.code, status.description);

        if !status.extra.is_empty() {
            text.push(';');
            text.push_str(&status.extra);
        }

        self.request_status.push(Value::String(text));
        self.hatch.note("requestStatus", prop, &[]);
    }

    /// `METHOD` is the scheduling method (RFC 8984 4.1.8).
    fn method(&mut self, prop: &IcalProp<'_>) {
        match text(prop) {
            Some(method) => self.member("method", Value::String(method.to_lowercase()), prop, &[]),
            None => self.hatch.keep(prop),
        }
    }

    /// A `VALARM` as an Alert (draft 2.2.2).
    fn alarm(&mut self, component: &IcalComponent<'_>) {
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

    /// A `PARTICIPANT` component as a Participant (draft 2.2.1).
    fn participant(&mut self, component: &IcalComponent<'_>) {
        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Participant".to_owned()));

        let mut hatch = IcalHatch::new("participant");
        let mut roles = Map::new();

        for prop in &component.props {
            let IcalPropName::Kind(kind) = &prop.name else {
                // NOTE: A JSID names the key its component took, and the key
                // is where that already is.
                if !prop.name.eq_ignore_ascii_case("JSID") {
                    hatch.keep(prop);
                }

                continue;
            };

            match (kind, text(prop)) {
                (IcalPropKind::CalendarAddress, Some(address)) => {
                    object.insert(
                        "sendTo".to_owned(),
                        json!({"imip": Value::String(address.clone())}),
                    );
                    hatch.note("sendTo/imip", prop, &[]);
                }
                (IcalPropKind::Summary, Some(name)) => {
                    object.insert("name".to_owned(), Value::String(name));
                    hatch.note("name", prop, &[]);
                }
                (IcalPropKind::Description, Some(description)) => {
                    object.insert("description".to_owned(), Value::String(description));
                    hatch.note("description", prop, &[]);
                }
                (IcalPropKind::ParticipantType, Some(kind)) => {
                    roles.insert(kind.to_lowercase(), Value::Bool(true));
                    hatch.note("roles", prop, &[]);
                }
                _ => hatch.keep(prop),
            }
        }

        for child in &component.components {
            hatch.keep_component(child);
        }

        object.insert(
            "roles".to_owned(),
            Value::Object(match roles.is_empty() {
                true => set(vec!["attendee".to_owned()]),
                false => roles,
            }),
        );

        if let Some(hatch) = hatch.into_value() {
            object.insert("iCalendar".to_owned(), hatch);
        }

        let key = component_key(&self.participants, component);
        self.participants.insert(key, Value::Object(object));
    }

    /// A `VLOCATION` component as a Location (draft 2.2.4).
    fn vlocation(&mut self, component: &IcalComponent<'_>) {
        let mut object = Map::new();
        object.insert("@type".to_owned(), Value::String("Location".to_owned()));

        let mut hatch = IcalHatch::new("vlocation");
        let mut types = Map::new();

        for prop in &component.props {
            let IcalPropName::Kind(kind) = &prop.name else {
                // NOTE: A JSID names the key its component took, and the key
                // is where that already is.
                if !prop.name.eq_ignore_ascii_case("JSID") {
                    hatch.keep(prop);
                }

                continue;
            };

            match (kind, text(prop)) {
                (IcalPropKind::Name, Some(name)) => {
                    object.insert("name".to_owned(), Value::String(name));
                    hatch.note("name", prop, &[]);
                }
                (IcalPropKind::Description, Some(description)) => {
                    object.insert("description".to_owned(), Value::String(description));
                    hatch.note("description", prop, &[]);
                }
                (IcalPropKind::LocationType, _) => {
                    types.extend(set(list(prop)));
                    hatch.note("locationTypes", prop, &[]);
                }
                (IcalPropKind::Url, Some(uri)) => {
                    object.insert("coordinates".to_owned(), Value::String(uri));
                    hatch.note("coordinates", prop, &[]);
                }
                _ => hatch.keep(prop),
            }
        }

        for child in &component.components {
            hatch.keep_component(child);
        }

        if !types.is_empty() {
            object.insert("locationTypes".to_owned(), Value::Object(types));
        }

        if let Some(hatch) = hatch.into_value() {
            object.insert("iCalendar".to_owned(), hatch);
        }

        let key = component_key(&self.locations, component);
        self.locations.insert(key, Value::Object(object));
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

/// The parameters a temporal property's conversion consumes: its time zone
/// always, and its declared value type when that is a date, since
/// `showWithoutTime` is where the object says so.
fn temporal_params(prop: &IcalProp<'_>) -> &'static [IcalParamKind] {
    match matches!(prop.value, IcalValue::Date(_)) {
        true => &[IcalParamKind::TzId, IcalParamKind::Value],
        false => &[IcalParamKind::TzId],
    }
}

/// The parameters an `ATTENDEE` or `ORGANIZER` conversion consumes, so only
/// what is genuinely left over is recorded.
const PARTICIPANT_PARAMS: &[IcalParamKind] = &[
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

/// The Participant an `ATTENDEE` or `ORGANIZER` line describes, without its
/// roles, which the two properties word differently.
fn participant(prop: &IcalProp<'_>, address: &str) -> Map<String, Value> {
    let mut object = Map::new();
    object.insert("@type".to_owned(), Value::String("Participant".to_owned()));
    object.insert(
        "sendTo".to_owned(),
        json!({"imip": Value::String(address.to_owned())}),
    );

    let scalars = [
        (IcalParamKind::Cn, "name", false),
        (IcalParamKind::Email, "email", false),
        (IcalParamKind::Language, "language", false),
        (IcalParamKind::PartStat, "participationStatus", true),
        (IcalParamKind::ScheduleAgent, "scheduleAgent", true),
        (IcalParamKind::ScheduleStatus, "scheduleStatus", false),
    ];

    for (kind, member, lower) in scalars {
        if let Some(value) = param(prop, kind) {
            let value = match lower {
                true => value.to_lowercase(),
                false => value.into_owned(),
            };

            object.insert(member.to_owned(), Value::String(value));
        }
    }

    if let Some(cutype) = param(prop, IcalParamKind::CuType) {
        // NOTE: A room is a location to JSCalendar (draft 2.3.4).
        let kind = match cutype.to_ascii_uppercase().as_str() {
            "ROOM" => "location".to_owned(),
            _ => cutype.to_lowercase(),
        };

        object.insert("kind".to_owned(), Value::String(kind));
    }

    let flags = [
        (IcalParamKind::Rsvp, "expectReply"),
        (IcalParamKind::ScheduleForceSend, "scheduleForceSend"),
    ];

    for (kind, member) in flags {
        if let Some(flag) = param(prop, kind) {
            object.insert(
                member.to_owned(),
                Value::Bool(flag.eq_ignore_ascii_case("TRUE")),
            );
        }
    }

    let sets = [
        (IcalParamKind::DelegatedFrom, "delegatedFrom"),
        (IcalParamKind::DelegatedTo, "delegatedTo"),
        (IcalParamKind::Member, "memberOf"),
    ];

    for (kind, member) in sets {
        let addresses = set(values(prop, kind));

        if !addresses.is_empty() {
            object.insert(member.to_owned(), Value::Object(addresses));
        }
    }

    object
}

/// The JSCalendar roles one `ROLE` parameter value stands for (draft 2.3.4).
fn roles(role: &str) -> Vec<String> {
    match role.to_ascii_uppercase().as_str() {
        "REQ-PARTICIPANT" => vec!["attendee".to_owned()],
        "OPT-PARTICIPANT" => vec!["attendee".to_owned(), "optional".to_owned()],
        "NON-PARTICIPANT" => vec!["informational".to_owned()],
        "CHAIR" => vec!["attendee".to_owned(), "chair".to_owned()],
        other => vec![other.to_lowercase()],
    }
}

/// The Task progress a `PARTSTAT` states, for the values that are specific to
/// a `VTODO` (draft 2.3.4).
fn progress(prop: &IcalProp<'_>) -> Option<String> {
    let status = param(prop, IcalParamKind::PartStat)?;

    match status.to_ascii_uppercase().as_str() {
        "COMPLETED" => Some("completed".to_owned()),
        "IN-PROCESS" => Some("in-process".to_owned()),
        "FAILED" => Some("failed".to_owned()),
        _ => None,
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

/// A `RECUR` value as a RecurrenceRule object (draft 2.3.36).
fn rule_to_json(text: &str) -> Value {
    let mut rule = Map::new();
    rule.insert(
        "@type".to_owned(),
        Value::String("RecurrenceRule".to_owned()),
    );

    for part in text.split(';').filter(|part| !part.is_empty()) {
        let (name, value) = part.split_once('=').unwrap_or((part, ""));
        let name = name.to_ascii_uppercase();

        let member = match rule_member(&name) {
            Some(member) => member,
            // NOTE: A part no RecurrenceRule member holds keeps its own name,
            // which RFC 8984 3.3 allows and a reader is told to preserve.
            None => {
                rule.insert(name.to_lowercase(), Value::String(value.to_owned()));
                continue;
            }
        };

        let converted = match name.as_str() {
            "FREQ" | "WKST" | "RSCALE" | "SKIP" => Value::String(value.to_lowercase()),
            "UNTIL" => Value::String(local_text(value, false).unwrap_or_else(|| value.to_owned())),
            "COUNT" | "INTERVAL" => match value.parse::<i64>() {
                Ok(number) => Value::Number(number.into()),
                Err(_) => Value::String(value.to_owned()),
            },
            "BYDAY" => Value::Array(value.split(',').map(nday).collect()),
            "BYMONTH" => Value::Array(
                value
                    .split(',')
                    .map(|month| Value::String(month.to_owned()))
                    .collect(),
            ),
            _ => Value::Array(
                value
                    .split(',')
                    .map(|item| match item.parse::<i64>() {
                        Ok(number) => Value::Number(number.into()),
                        Err(_) => Value::String(item.to_owned()),
                    })
                    .collect(),
            ),
        };

        rule.insert(member.to_owned(), converted);
    }

    Value::Object(rule)
}

/// The RecurrenceRule member a `RECUR` part converts to, if any.
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

/// One `BYDAY` item as an NDay object.
fn nday(item: &str) -> Value {
    let split = item.len().saturating_sub(2);
    let (ordinal, day) = item.split_at(split);

    let mut object = Map::new();
    object.insert("@type".to_owned(), Value::String("NDay".to_owned()));
    object.insert("day".to_owned(), Value::String(day.to_lowercase()));

    if let Ok(nth) = ordinal.parse::<i64>() {
        object.insert("nthOfPeriod".to_owned(), Value::Number(nth.into()));
    }

    Value::Object(object)
}

/// The key an element takes in the collection it joins: what its `JSID` says,
/// else its position, which is stable for as long as the source is (draft
/// 2.1.3).
fn key(collection: &Map<String, Value>, prop: &IcalProp<'_>) -> String {
    jsid_param(prop).unwrap_or_else(|| (collection.len() + 1).to_string())
}

/// The same, for an element that is a whole component: its `JSID` property,
/// else its `UID`, else its position.
fn component_key(collection: &Map<String, Value>, component: &IcalComponent<'_>) -> String {
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
fn set(values: Vec<String>) -> Map<String, Value> {
    values
        .into_iter()
        .filter(|value| !value.is_empty())
        .map(|value| (value, Value::Bool(true)))
        .collect()
}

/// The scalar text of a property's value, for the value shapes that hold one.
fn text(prop: &IcalProp<'_>) -> Option<String> {
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
fn list(prop: &IcalProp<'_>) -> Vec<String> {
    match &prop.value {
        IcalValue::TextList(list) => list.0.iter().map(|item| item.to_string()).collect(),
        IcalValue::DateTimeList(list) => list.0.iter().map(|item| item.to_string()).collect(),
        _ => text(prop).into_iter().collect(),
    }
}

/// The scalar text of a property's first parameter of this kind.
fn param<'a>(prop: &'a IcalProp<'a>, kind: IcalParamKind) -> Option<Cow<'a, str>> {
    prop.params
        .iter()
        .find(|param| param.kind() == Some(kind))
        .map(param_scalar)
        .map(unquoted)
        .filter(|text| !text.is_empty())
}

/// A parameter value without the quotes iCalendar wraps it in when it holds a
/// colon, a semicolon or a comma (RFC 5545 3.2).
///
/// The decoded model keeps them, since they are bytes the syntax tree has to
/// reproduce; JSON has no such rule, so this is where they come off.
fn unquoted(text: Cow<'_, str>) -> Cow<'_, str> {
    let trimmed = text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .map(ToOwned::to_owned);

    match trimmed {
        Some(trimmed) => Cow::Owned(trimmed),
        None => text,
    }
}

/// Every value of a property's parameters of this kind.
fn values(prop: &IcalProp<'_>, kind: IcalParamKind) -> Vec<String> {
    prop.params
        .iter()
        .filter(|param| param.kind() == Some(kind))
        .flat_map(|param| match param {
            IcalParam::DelegatedFrom(values)
            | IcalParam::DelegatedTo(values)
            | IcalParam::Member(values)
            | IcalParam::Feature(values) => values
                .iter()
                .map(|value| unquoted(Cow::Borrowed(value)).into_owned())
                .collect(),
            other => vec![unquoted(param_scalar(other)).into_owned()],
        })
        .filter(|value| !value.is_empty())
        .collect()
}

/// The LocalDateTime a temporal property states.
fn local(prop: &IcalProp<'_>) -> Option<String> {
    let text = text(prop)?;
    local_text(&text, matches!(prop.value, IcalValue::Date(_)))
}

/// The UTCDateTime a timestamp property states, always with its `Z`.
fn utc(prop: &IcalProp<'_>) -> Option<String> {
    let at = local(prop)?;
    Some(format!("{at}Z"))
}

/// One date or date-time value in the JSCalendar spelling: extended, with the
/// time filled in for a date and the UTC suffix dropped, since a JSCalendar
/// date-time says its zone in a member of its own.
fn local_text(text: &str, date_only: bool) -> Option<String> {
    if text.is_empty() {
        return None;
    }

    let extended = datetime_to_json(text);
    let extended = extended.strip_suffix(['Z', 'z']).unwrap_or(&extended);

    match date_only || !extended.contains('T') {
        true => Some(format!("{}T00:00:00", extended.split('T').next()?)),
        false => Some(extended.to_owned()),
    }
}

/// The time zone a temporal property references (draft 2.1.4).
fn zone(prop: &IcalProp<'_>) -> Option<String> {
    if let Some(zone) = param(prop, IcalParamKind::TzId) {
        return Some(zone.into_owned());
    }

    let text = text(prop)?;

    // NOTE: A trailing Z is the only other zone iCalendar states inline; a
    // floating value states none, and JSCalendar spells that as no member.
    match text.ends_with(['Z', 'z']) {
        true => Some("Etc/UTC".to_owned()),
        false => None,
    }
}

/// The duration between two JSCalendar date-times, in the iCalendar spelling
/// JSCalendar borrows for its `duration` member.
fn span(start: &str, end: &str) -> Option<String> {
    let start = IcalRecurDateTime::parse(&start.replace(['-', ':'], "")).ok()?;
    let end = IcalRecurDateTime::parse(&end.replace(['-', ':'], "")).ok()?;

    Some(duration(end.seconds() - start.seconds()))
}

/// A number of seconds as an RFC 5545 duration.
fn duration(seconds: i64) -> String {
    let sign = match seconds < 0 {
        true => "-",
        false => "",
    };

    let seconds = seconds.unsigned_abs();
    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    let (hours, rest) = (rest / 3_600, rest % 3_600);
    let (minutes, seconds) = (rest / 60, rest % 60);

    let mut duration = String::from(sign);
    duration.push('P');

    if days > 0 {
        duration.push_str(&format!("{days}D"));
    }

    if hours == 0 && minutes == 0 && seconds == 0 {
        // NOTE: A whole number of days needs no time part, but a zero-length
        // span still has to spell something.
        if days == 0 {
            duration.push_str("T0S");
        }

        return duration;
    }

    duration.push('T');

    for (amount, unit) in [(hours, 'H'), (minutes, 'M'), (seconds, 'S')] {
        if amount > 0 {
            duration.push_str(&format!("{amount}{unit}"));
        }
    }

    duration
}
