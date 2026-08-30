//! # Recurrence rules
//!
//! A `RECUR` value as the RFC 7265 3.5.6 object, and back.
//!
//! Out, one lowercase key per rule part, a number where the part is numeric
//! and an array where it has several values. Back, the parts come out in the
//! order RFC 5545 3.3.10 states them: a JSON object has no order to preserve,
//! so this is a normalisation rather than a loss.

use alloc::{borrow::Cow, string::String, vec::Vec};

use serde_json::{Map, Value};

use crate::jcal::{
    datetime::{datetime_from_json, datetime_to_json},
    json::{number, scalar_text},
};

/// The rule parts in the order RFC 5545 3.3.10 states them, which is the order
/// a rule read back from JSON comes out in.
const RECUR_ORDER: [&str; 14] = [
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
];

/// A rule as the RFC 7265 3.5.6 object.
pub(crate) fn recur_to_json(text: &str) -> Value {
    let mut parts = Map::new();

    for part in text.split(';').filter(|part| !part.is_empty()) {
        let (name, value) = match part.split_once('=') {
            Some(split) => split,
            None => (part, ""),
        };

        let name = name.to_ascii_lowercase();
        let values: Vec<Value> = value
            .split(',')
            .map(|item| match name.as_str() {
                "until" => Value::String(datetime_to_json(item)),
                "freq" | "wkst" | "byday" | "rscale" | "skip" => {
                    Value::String(item.to_ascii_lowercase())
                }
                _ => number(item),
            })
            .collect();

        parts.insert(
            name,
            match values.as_slice() {
                [one] => one.clone(),
                many => Value::Array(many.to_vec()),
            },
        );
    }

    Value::Object(parts)
}

/// A rule read back, its parts in the RFC's order.
pub(crate) fn recur_from_json(value: Option<&Value>) -> Cow<'_, str> {
    let Some(Value::Object(parts)) = value else {
        return value.map(scalar_text).unwrap_or(Cow::Borrowed(""));
    };

    let mut names: Vec<&String> = parts.keys().collect();
    names.sort_by_key(|name| {
        RECUR_ORDER
            .iter()
            .position(|known| known.eq_ignore_ascii_case(name))
            .unwrap_or(RECUR_ORDER.len())
    });

    let mut rule = String::new();

    for name in names {
        let value = &parts[name];
        let text = match value {
            Value::Array(items) => items
                .iter()
                .map(|item| part_text(name, item))
                .collect::<Vec<_>>()
                .join(","),
            other => part_text(name, other),
        };

        if !rule.is_empty() {
            rule.push(';');
        }
        rule.push_str(&name.to_ascii_uppercase());
        rule.push('=');
        rule.push_str(&text);
    }

    Cow::Owned(rule)
}

/// One value of one rule part, back in its wire spelling.
fn part_text(name: &str, value: &Value) -> String {
    let text = scalar_text(value);

    match name {
        "until" => datetime_from_json(&text).unwrap_or_else(|| text.into_owned()),
        "freq" | "wkst" | "byday" | "rscale" | "skip" => text.to_ascii_uppercase(),
        _ => text.into_owned(),
    }
}
