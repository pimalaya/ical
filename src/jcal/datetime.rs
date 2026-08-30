//! # Temporal spellings
//!
//! Dates, times, periods and UTC offsets re-spelled between the wire's basic
//! ISO 8601 and the extended form RFC 7265 3.5.1 to 3.5.5 asks for.
//!
//! Each pair is written as "out, always; back, only when there is something to
//! rewrite", so a value that arrived borrowed stays borrowed and only a value
//! that actually changes is allocated. Anything that is neither spelling
//! passes through untouched, which is Postel's law at this level too.

use alloc::{
    format,
    string::{String, ToString},
};

/// `YYYYMMDD` to `YYYY-MM-DD`. Anything else passes through.
pub(crate) fn date_to_json(text: &str) -> String {
    match text.len() == 8 && text.bytes().all(|b| b.is_ascii_digit()) {
        true => format!("{}-{}-{}", &text[..4], &text[4..6], &text[6..8]),
        false => text.to_string(),
    }
}

/// The inverse, `None` when the text was not in the JSON spelling.
pub(crate) fn date_from_json(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    (bytes.len() == 10 && bytes[4] == b'-' && bytes[7] == b'-')
        .then(|| format!("{}{}{}", &text[..4], &text[5..7], &text[8..10]))
}

/// `HHMMSS` to `HH:MM:SS`, with the UTC suffix kept. Anything else passes
/// through.
pub(crate) fn time_to_json(text: &str) -> String {
    let (body, zulu) = split_zulu(text);
    match body.len() == 6 && body.bytes().all(|b| b.is_ascii_digit()) {
        true => format!("{}:{}:{}{zulu}", &body[..2], &body[2..4], &body[4..6]),
        false => text.to_string(),
    }
}

/// The inverse, `None` when the text was not in the JSON spelling.
pub(crate) fn time_from_json(text: &str) -> Option<String> {
    let (body, zulu) = split_zulu(text);
    let bytes = body.as_bytes();
    (bytes.len() == 8 && bytes[2] == b':' && bytes[5] == b':')
        .then(|| format!("{}{}{}{zulu}", &body[..2], &body[3..5], &body[6..8]))
}

/// `YYYYMMDDTHHMMSS` to `YYYY-MM-DDTHH:MM:SS`, with the UTC suffix kept. A
/// date-only value is re-spelled as a date. Anything else passes through.
pub(crate) fn datetime_to_json(text: &str) -> String {
    let (body, zulu) = split_zulu(text);

    let Some((date, time)) = split_t(body) else {
        return date_to_json(body) + zulu;
    };

    format!("{}T{}{zulu}", date_to_json(date), time_to_json(time))
}

/// The inverse of [`datetime_to_json`], `None` when neither half was in the
/// JSON spelling, so a value already on the wire form is left for the caller
/// to pass through untouched.
pub(crate) fn datetime_from_json(text: &str) -> Option<String> {
    let (body, zulu) = split_zulu(text);

    let Some((date, time)) = split_t(body) else {
        return date_from_json(body).map(|date| format!("{date}{zulu}"));
    };

    let (rewritten_date, rewritten_time) = (date_from_json(date), time_from_json(time));
    if rewritten_date.is_none() && rewritten_time.is_none() {
        return None;
    }

    Some(format!(
        "{}T{}{zulu}",
        rewritten_date.as_deref().unwrap_or(date),
        rewritten_time.as_deref().unwrap_or(time),
    ))
}

/// A period (`start/end` or `start/duration`) with both halves re-spelled. A
/// value with no `/` is re-spelled as a date-time, which is what an `RDATE`
/// item is.
pub(crate) fn period_to_json(text: &str) -> String {
    match text.split_once('/') {
        Some((start, end)) => format!("{}/{}", datetime_to_json(start), datetime_to_json(end)),
        None => datetime_to_json(text),
    }
}

/// The inverse, `None` when neither half was in the JSON spelling.
pub(crate) fn period_from_json(text: &str) -> Option<String> {
    let Some((start, end)) = text.split_once('/') else {
        return datetime_from_json(text);
    };

    let (rewritten_start, rewritten_end) = (datetime_from_json(start), datetime_from_json(end));
    if rewritten_start.is_none() && rewritten_end.is_none() {
        return None;
    }

    Some(format!(
        "{}/{}",
        rewritten_start.as_deref().unwrap_or(start),
        rewritten_end.as_deref().unwrap_or(end),
    ))
}

/// `+HHMM[SS]` to `+HH:MM[:SS]`. Anything else passes through.
pub(crate) fn offset_to_json(text: &str) -> String {
    let (sign, digits) = match text.as_bytes().first() {
        Some(b'+') | Some(b'-') => (&text[..1], &text[1..]),
        _ => ("", text),
    };

    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return text.to_string();
    }

    match digits.len() {
        4 => format!("{sign}{}:{}", &digits[..2], &digits[2..4]),
        6 => format!("{sign}{}:{}:{}", &digits[..2], &digits[2..4], &digits[4..6]),
        _ => text.to_string(),
    }
}

/// The inverse, `None` when the text carried no separator to drop.
pub(crate) fn offset_from_json(text: &str) -> Option<String> {
    text.contains(':').then(|| text.replace(':', ""))
}

/// Split the trailing UTC `Z` off a value.
fn split_zulu(text: &str) -> (&str, &str) {
    match text.as_bytes().last() {
        Some(b'Z') | Some(b'z') => (&text[..text.len() - 1], &text[text.len() - 1..]),
        _ => (text, ""),
    }
}

/// Split a date-time on its `T`.
fn split_t(text: &str) -> Option<(&str, &str)> {
    let at = text.find(['T', 't'])?;
    Some((&text[..at], &text[at + 1..]))
}
