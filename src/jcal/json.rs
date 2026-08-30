//! # JSON scalars
//!
//! The three conversions every direction of the codec needs: a number that
//! stays the number it was, a string list spelled the way RFC 7265 3.5.2 asks
//! for, and a scalar read as text whatever JSON type it arrived as.

use alloc::{borrow::Cow, string::ToString};

use serde_json::Value;

/// A number as JSON, or the raw text when it is not one. An integer stays an
/// integer: `5` and `5.0` are the same number to JSON but not to a reader.
pub(crate) fn number(text: &str) -> Value {
    if let Ok(whole) = text.parse::<i64>() {
        return Value::Number(whole.into());
    }

    text.parse::<f64>()
        .ok()
        .and_then(serde_json::Number::from_f64)
        .map(Value::Number)
        .unwrap_or_else(|| Value::String(text.to_string()))
}

/// A list of strings as one JSON value: the string itself when there is one,
/// an array otherwise, which is what RFC 7265 3.5.2 asks for.
pub(crate) fn strings(values: &[Cow<'_, str>]) -> Value {
    match values {
        [one] => Value::String(one.to_string()),
        many => Value::Array(many.iter().map(|v| Value::String(v.to_string())).collect()),
    }
}

/// A JSON scalar as text: a string as itself, anything else through its JSON
/// spelling, so nothing is dropped for being the wrong type.
pub(crate) fn scalar_text(value: &Value) -> Cow<'_, str> {
    match value {
        Value::String(text) => Cow::Borrowed(text),
        Value::Null => Cow::Borrowed(""),
        other => Cow::Owned(other.to_string()),
    }
}
