//! # The escape hatch
//!
//! What the mapping cannot express, kept rather than dropped.
//!
//! Every JSCalendar object this conversion writes may carry an `iCalendar`
//! member holding an `ICalComponent` object ([the conversion draft] 5.1.1).
//!
//! It holds the component's name, the properties and subcomponents that did
//! not convert (in jCal syntax, so the sibling [`crate::jcal`] codec reads and
//! writes them), and a `convertedProperties` map recording, per JSCalendar
//! member, which iCalendar property it came from and what was left over on it.
//!
//! That last map is what makes the round trip exact rather than approximate.
//! Several iCalendar properties share one JSCalendar member (`updated` is
//! either `DTSTAMP` or `LAST-MODIFIED`, a link is any of `ATTACH`, `IMAGE`,
//! `LINK` or `URL`).
//!
//! A property may also carry parameters the JSCalendar object has nowhere to
//! put. [`default_name`] holds the assumption the writer makes, so a record is
//! written only where the assumption would be wrong.
//!
//! [the conversion draft]: https://datatracker.ietf.org/doc/draft-ietf-calext-jscalendar-icalendar/

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
    vec::Vec,
};

use serde_json::{Map, Value};

use crate::{
    component::IcalComponent,
    jcal::{
        component_from_jcal, component_to_jcal, param_from_jcal, param_to_jcal, prop_from_jcal,
        prop_to_jcal, type_slot,
    },
    param::{IcalParam, IcalParamKind},
    prop::IcalProp,
    version::IcalVersion,
};

/// The escape hatch of one component, under construction.
pub(crate) struct IcalHatch {
    /// The lowercase name of the iCalendar component this hatch belongs to.
    name: String,
    /// The properties that did not convert, in jCal syntax.
    properties: Vec<Value>,
    /// The subcomponents that did not convert, in jCal syntax.
    components: Vec<Value>,
    /// What each converted member came from, keyed by its pointer.
    converted: Map<String, Value>,
}

impl IcalHatch {
    /// A fresh hatch for a component of this name.
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_ascii_lowercase(),
            properties: Vec::new(),
            components: Vec::new(),
            converted: Map::new(),
        }
    }

    /// Keep a property whole, because nothing in JSCalendar holds it.
    pub(crate) fn keep(&mut self, prop: &IcalProp<'_>) {
        self.properties.push(prop_to_jcal(prop));
    }

    /// Keep a property already written as a jCal array.
    pub(crate) fn keep_value(&mut self, prop: Value) {
        self.properties.push(prop);
    }

    /// Keep a subcomponent whole, for the same reason.
    pub(crate) fn keep_component(&mut self, component: &IcalComponent<'_>) {
        self.components.push(component_to_jcal(component));
    }

    /// Record what a converted member came from, if that is not what a reader
    /// would otherwise assume: a property under an unexpected name, a declared
    /// value type, or parameters the mapping did not consume.
    pub(crate) fn note(&mut self, pointer: &str, prop: &IcalProp<'_>, consumed: &[IcalParamKind]) {
        let mut params = Map::new();

        for param in &prop.params {
            let kind = param.kind();

            // NOTE: VALUE is not a leftover: it is the type slot, recorded
            // below. Neither is JSID, which named the key the element took.
            if matches!(kind, Some(IcalParamKind::Value))
                || kind.is_some_and(|kind| consumed.contains(&kind))
                || matches!(param, IcalParam::Unknown { name, .. } if name.eq_ignore_ascii_case("JSID"))
            {
                continue;
            }

            let (name, value) = param_to_jcal(param);
            params.insert(name, value);
        }

        let name = prop.name.to_lowercase();
        let expected = default_name(&self.name, pointer);

        // NOTE: A declared VALUE the mapping consumed is already said by the
        // member it converted to (`showWithoutTime` says DATE), so recording it
        // would only repeat the member back at the reader.
        let declared = !consumed.contains(&IcalParamKind::Value)
            && prop
                .params
                .iter()
                .any(|param| matches!(param.kind(), Some(IcalParamKind::Value)));

        if params.is_empty() && !declared && expected == Some(name.as_str()) {
            return;
        }

        let mut record = Map::new();
        record.insert("@type".to_owned(), Value::String("ICalProperty".to_owned()));
        record.insert("name".to_owned(), Value::String(name));

        if declared {
            record.insert("valueType".to_owned(), Value::String(type_slot(prop)));
        }

        if !params.is_empty() {
            record.insert("parameters".to_owned(), Value::Object(params));
        }

        self.converted
            .insert(pointer.to_owned(), Value::Object(record));
    }

    /// The hatch as an `ICalComponent` value, or nothing when it caught
    /// nothing and so would say only what the reader already knows.
    pub(crate) fn into_value(self) -> Option<Value> {
        if self.properties.is_empty() && self.components.is_empty() && self.converted.is_empty() {
            return None;
        }

        let mut object = Map::new();
        object.insert(
            "@type".to_owned(),
            Value::String("ICalComponent".to_owned()),
        );
        object.insert("name".to_owned(), Value::String(self.name));

        if !self.converted.is_empty() {
            object.insert(
                "convertedProperties".to_owned(),
                Value::Object(self.converted),
            );
        }

        if !self.properties.is_empty() {
            object.insert("properties".to_owned(), Value::Array(self.properties));
        }

        if !self.components.is_empty() {
            object.insert("components".to_owned(), Value::Array(self.components));
        }

        Some(Value::Object(object))
    }
}

/// The `ICalComponent` a JSCalendar object carries, if any.
pub(crate) fn hatch_of(object: &Map<String, Value>) -> Option<&Map<String, Value>> {
    object.get("iCalendar")?.as_object()
}

/// The properties a hatch kept, decoded back.
pub(crate) fn kept_props<'a>(
    hatch: Option<&'a Map<String, Value>>,
    version: IcalVersion,
) -> Vec<IcalProp<'a>> {
    let Some(entries) = hatch
        .and_then(|hatch| hatch.get("properties"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    entries
        .iter()
        .map(|entry| prop_from_jcal(entry, version))
        .collect()
}

/// The subcomponents a hatch kept, decoded back.
pub(crate) fn kept_components<'a>(
    hatch: Option<&'a Map<String, Value>>,
    version: IcalVersion,
) -> Vec<IcalComponent<'a>> {
    let Some(entries) = hatch
        .and_then(|hatch| hatch.get("components"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    entries
        .iter()
        .map(|entry| component_from_jcal(entry, version))
        .collect()
}

/// What a converted member came from: the iCalendar property's name, its
/// declared value type, and the parameters that did not convert.
pub(crate) struct IcalConverted<'a> {
    /// The property name, uppercase as iCalendar spells it.
    pub name: Cow<'a, str>,
    /// The declared `VALUE`, uppercase, when the property carried one.
    pub value_type: Option<Cow<'a, str>>,
    /// The parameters the conversion left behind.
    pub params: Vec<IcalParam<'a>>,
}

/// The record a hatch holds for one member, read back.
///
/// The `component` and `pointer` are what [`default_name`] answers for, so a
/// member with no record still names the property it came from.
pub(crate) fn converted<'a>(
    hatch: Option<&'a Map<String, Value>>,
    component: &str,
    pointer: &str,
) -> Option<IcalConverted<'a>> {
    let record = hatch
        .and_then(|hatch| hatch.get("convertedProperties"))
        .and_then(Value::as_object)
        .and_then(|converted| converted.get(pointer))
        .and_then(Value::as_object);

    let Some(record) = record else {
        let name = default_name(component, pointer)?;
        return Some(IcalConverted {
            name: Cow::Owned(name.to_ascii_uppercase()),
            value_type: None,
            params: Vec::new(),
        });
    };

    let name = record
        .get("name")
        .and_then(Value::as_str)
        .map(|name| name.to_ascii_uppercase())
        .or_else(|| default_name(component, pointer).map(str::to_ascii_uppercase))?;

    let params = record
        .get("parameters")
        .and_then(Value::as_object)
        .map(|params| {
            params
                .iter()
                .map(|(name, value)| param_from_jcal(name, value))
                .collect()
        })
        .unwrap_or_default();

    Some(IcalConverted {
        name: Cow::Owned(name),
        value_type: record
            .get("valueType")
            .and_then(Value::as_str)
            .map(|slot| Cow::Owned(slot.to_ascii_uppercase())),
        params,
    })
}

/// The iCalendar property a member is assumed to have come from.
///
/// A record is needed only where the assumption is wrong. A pointer into a
/// collection is matched by shape, the key standing as `*`: every link is an
/// `ATTACH` unless recorded otherwise, a location's `coordinates` a `GEO`.
pub(crate) fn default_name(component: &str, pointer: &str) -> Option<&'static str> {
    let shape = shape(pointer);

    // NOTE: The calendar names itself with NAME (RFC 7986 5.1); a component
    // names itself with SUMMARY.
    if shape == "title" {
        return match component {
            "vcalendar" => Some("name"),
            _ => Some("summary"),
        };
    }

    let name = match shape.as_str() {
        "uid" => "uid",
        "prodId" => "prodid",
        "description" => "description",
        "created" => "created",
        "updated" => "dtstamp",
        "sequence" => "sequence",
        "priority" => "priority",
        "color" => "color",
        "keywords" => "categories",
        "categories" => "concept",
        "privacy" => "class",
        "status" => "status",
        "progress" => "status",
        "freeBusyStatus" => "transp",
        "start" => "dtstart",
        "due" => "due",
        "duration" => "duration",
        "percentComplete" => "percent-complete",
        "progressUpdated" => "completed",
        "recurrenceRules" => "rrule",
        "excludedRecurrenceRules" => "exrule",
        "recurrenceId" => "recurrence-id",
        "source" => "source",
        "method" => "method",
        "requestStatus" => "request-status",
        "trigger" => "trigger",
        "action" => "action",
        "acknowledged" => "acknowledged",
        "links/*" => "attach",
        "locations/*" => "location",
        "locations/*/coordinates" => "geo",
        "virtualLocations/*" => "conference",
        "participants/*" => "attendee",
        "relatedTo/*" => "related-to",
        "replyTo/imip" => "organizer",
        _ => return None,
    };

    Some(name)
}

/// A pointer with a collection key replaced by `*`, so one table entry covers
/// every member of a collection.
fn shape(pointer: &str) -> String {
    let mut segments: Vec<&str> = pointer.split('/').collect();

    let collection = matches!(
        segments.first().copied(),
        Some("links" | "locations" | "virtualLocations" | "participants" | "alerts" | "relatedTo")
    );

    if collection && segments.len() > 1 {
        segments[1] = "*";
    }

    segments.join("/")
}

#[cfg(test)]
mod tests {
    use crate::jscalendar::hatch::{default_name, shape};

    #[test]
    fn matches_a_collection_member_by_shape() {
        assert_eq!(shape("links/3"), "links/*");
        assert_eq!(
            shape("locations/a~1b/coordinates"),
            "locations/*/coordinates"
        );
        assert_eq!(shape("start"), "start");
    }

    #[test]
    fn names_the_calendar_title_apart_from_a_component_title() {
        assert_eq!(default_name("vcalendar", "title"), Some("name"));
        assert_eq!(default_name("vevent", "title"), Some("summary"));
    }

    #[test]
    fn answers_nothing_for_a_member_no_property_maps_onto() {
        assert_eq!(default_name("vevent", "timeZone"), None);
    }
}
