//! # PatchObject
//!
//! The RFC 8984 1.4.9 patch: an unordered set of JSON pointers onto the values
//! they set, with `null` meaning "remove this member".
//!
//! A recurrence override is stored as the patch that turns the series into that
//! one instance (RFC 8984 4.3.5), so converting an overriding `VEVENT` needs
//! the difference between two converted objects, and reading one back needs the
//! difference applied. Both live here.

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec::Vec,
};

use serde_json::{Map, Value};

/// The members a recurrence override patch must not touch (RFC 8984 4.3.5).
///
/// They belong to the series rather than to one instance, so a difference
/// falling on one of them is dropped rather than written into the override.
const UNPATCHABLE: [&str; 14] = [
    "@type",
    "excludedRecurrenceRules",
    "method",
    "privacy",
    "prodId",
    "recurrenceId",
    "recurrenceIdTimeZone",
    "recurrenceOverrides",
    "recurrenceRules",
    "relatedTo",
    "replyTo",
    "sentBy",
    "timeZones",
    "uid",
];

/// The patch that turns `base` into `over`.
///
/// Nested objects are walked so the patch names the smallest member that
/// actually changed, which is what makes an override readable; an array is
/// replaced whole, since RFC 8984 forbids a pointer into one.
pub(crate) fn diff(base: &Map<String, Value>, over: &Map<String, Value>) -> Map<String, Value> {
    let mut patch = Map::new();
    walk(base, over, "", &mut patch);
    patch.retain(|pointer, _| !unpatchable(pointer));
    patch
}

/// Apply a patch to an object, in place.
///
/// Liberal, as everything on the import side is: a pointer whose parent does
/// not exist is skipped rather than created, which is RFC 8984's second
/// validity condition read as a rule about what to ignore.
pub(crate) fn apply(target: &mut Map<String, Value>, patch: &Map<String, Value>) {
    // NOTE: A patch is unordered, but applying it must not be, or a member and
    // its parent would race. Shorter pointers first means a parent is always
    // set before anything reaching through it.
    let mut pointers: Vec<&String> = patch.keys().collect();
    pointers.sort_by_key(|pointer| (pointer.matches('/').count(), pointer.to_string()));

    for pointer in pointers {
        set(target, pointer, patch[pointer].clone());
    }
}

/// Collect the differences between two objects under one pointer prefix.
fn walk(
    base: &Map<String, Value>,
    over: &Map<String, Value>,
    prefix: &str,
    patch: &mut Map<String, Value>,
) {
    for (key, value) in over {
        let pointer = join(prefix, key);

        match (base.get(key), value) {
            (Some(old), _) if old == value => continue,
            // NOTE: An object that gains or changes members is compared member
            // by member; one that is emptied, and anything that is not an
            // object, is replaced whole.
            (Some(Value::Object(old)), Value::Object(new)) if !new.is_empty() => {
                walk(old, new, &pointer, patch)
            }
            _ => {
                patch.insert(pointer, value.clone());
            }
        }
    }

    for key in base.keys() {
        if !over.contains_key(key) {
            patch.insert(join(prefix, key), Value::Null);
        }
    }
}

/// Set (or, for a null value, remove) one pointer.
fn set(target: &mut Map<String, Value>, pointer: &str, value: Value) {
    let Some((head, rest)) = pointer.split_once('/') else {
        let key = unescape(pointer);

        match value {
            Value::Null => target.remove(&key),
            value => target.insert(key, value),
        };

        return;
    };

    let Some(Value::Object(inner)) = target.get_mut(&unescape(head)) else {
        return;
    };

    set(inner, rest, value);
}

/// Whether a pointer falls on a member an override may not carry.
fn unpatchable(pointer: &str) -> bool {
    let head = pointer.split('/').next().unwrap_or(pointer);
    UNPATCHABLE.contains(&head)
}

/// One pointer segment appended to a prefix.
fn join(prefix: &str, key: &str) -> String {
    match prefix.is_empty() {
        true => escape(key),
        false => format!("{prefix}/{}", escape(key)),
    }
}

/// Apply the RFC 6901 escapes to one pointer segment.
fn escape(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

/// Undo the RFC 6901 escapes of one pointer segment.
fn unescape(segment: &str) -> String {
    match segment.contains('~') {
        true => segment.replace("~1", "/").replace("~0", "~"),
        false => segment.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use serde_json::json;

    use crate::jscalendar::patch::{apply, diff};

    fn object(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
        value.as_object().expect("an object").clone()
    }

    #[test]
    fn names_the_smallest_member_that_changed() {
        let base = object(json!({
            "title": "Weekly", "start": "2024-01-01T09:00:00",
            "locations": {"1": {"@type": "Location", "name": "Room A"}}
        }));
        let over = object(json!({
            "title": "Weekly", "start": "2024-01-08T10:00:00",
            "locations": {"1": {"@type": "Location", "name": "Room B"}}
        }));

        assert_eq!(
            diff(&base, &over),
            object(json!({
                "start": "2024-01-08T10:00:00",
                "locations/1/name": "Room B"
            }))
        );
    }

    #[test]
    fn removes_a_member_the_override_dropped() {
        let base = object(json!({"title": "Weekly", "description": "Bring notes"}));
        let over = object(json!({"title": "Weekly"}));

        assert_eq!(diff(&base, &over), object(json!({"description": null})));
    }

    #[test]
    fn leaves_what_belongs_to_the_series_alone() {
        let base = object(json!({"uid": "a", "recurrenceRules": [{"frequency": "daily"}]}));
        let over = object(json!({"uid": "b"}));

        assert!(diff(&base, &over).is_empty());
    }

    #[test]
    fn replaces_an_array_whole() {
        let base = object(json!({"requestStatus": ["2.0;Success"]}));
        let over = object(json!({"requestStatus": ["2.0;Success", "3.7;Invalid"]}));

        assert_eq!(
            diff(&base, &over),
            object(json!({"requestStatus": ["2.0;Success", "3.7;Invalid"]}))
        );
    }

    #[test]
    fn round_trips_through_apply() {
        let base = object(json!({
            "title": "Weekly", "start": "2024-01-01T09:00:00", "description": "Bring notes",
            "participants": {"1": {"@type": "Participant", "name": "Ada", "roles": {"attendee": true}}}
        }));
        let over = object(json!({
            "title": "Weekly", "start": "2024-01-08T10:00:00",
            "participants": {"1": {"@type": "Participant", "name": "Grace", "roles": {"attendee": true}}}
        }));

        let mut patched = base.clone();
        apply(&mut patched, &diff(&base, &over));

        assert_eq!(patched, over);
    }

    #[test]
    fn skips_a_pointer_whose_parent_is_missing() {
        let mut target = object(json!({"title": "Weekly"}));
        apply(&mut target, &object(json!({"locations/1/name": "Room B"})));

        assert_eq!(target, object(json!({"title": "Weekly"})));
    }

    #[test]
    fn escapes_a_key_holding_a_solidus() {
        let base = object(json!({"links": {}}));
        let over = object(json!({"links": {"a/b": {"href": "https://example.com"}}}));

        let patch = diff(&base, &over);
        assert_eq!(patch.keys().next().map(String::as_str), Some("links/a~1b"));

        let mut patched = base.clone();
        apply(&mut patched, &patch);
        assert_eq!(patched, over);
    }
}
