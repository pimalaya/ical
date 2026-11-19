//! Calendars as real clients write them, through JSCalendar and back.
//!
//! Every fixture of tests/corpus/clients (an iOS series, an Exchange meeting,
//! a Google invitation, a Thunderbird task) must come back from its Group line
//! for line once normalized (lines and parameters sorted, folds undone, names
//! upper-cased, the conversion's own `JSID` keys left out), in RFC 8984 and in
//! JSCalendar 2.0, with every `X-` property and parameter it carries.

#![cfg(all(feature = "jscalendar", feature = "parser"))]

mod common;

use ical::{ical::Ical, jscalendar::IcalJscalendarVersion, tree::cst::IcalCst};
use serde_json::Value;

/// A calendar's decoded model, from its wire bytes.
fn decode(ics: &[u8]) -> Ical<'static> {
    IcalCst::parse(ics)
        .expect("a readable calendar")
        .decode()
        .into_owned()
}

/// The calendar a JSCalendar value reads as, on the wire.
fn read(value: &Value) -> String {
    let ical = Ical::from_jscalendar(value).expect("a readable JSCalendar object");
    IcalCst::from(ical).to_string()
}

/// Splits `text` on `sep` outside double quotes.
fn split_unquoted(text: &str, sep: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut quoted = false;
    let mut start = 0;

    for (at, char) in text.char_indices() {
        match char {
            '"' => quoted = !quoted,
            _ if char == sep && !quoted => {
                out.push(&text[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }

    out.push(&text[start..]);
    out
}

/// The value type a property takes with no `VALUE` parameter, for the
/// properties the fixtures state it on (RFC 5545 3.8).
const DEFAULT_TYPES: [(&str, &str); 7] = [
    ("URL", "URI"),
    ("TRIGGER", "DURATION"),
    ("DTSTART", "DATE-TIME"),
    ("DTEND", "DATE-TIME"),
    ("DUE", "DATE-TIME"),
    ("EXDATE", "DATE-TIME"),
    ("ATTACH", "URI"),
];

/// The parameters a property means with none stated (RFC 5545 3.2.16,
/// 3.2.14), which a JSCalendar object has no way to state.
const DEFAULT_PARAMS: [(&str, &str); 2] = [
    ("ATTENDEE", "ROLE=REQ-PARTICIPANT"),
    ("TRIGGER", "RELATED=START"),
];

/// A parameter normalized: its name upper-cased, the quotes around a value
/// needing none dropped (RFC 5545 3.2).
fn param(param: &str) -> String {
    let Some((name, value)) = param.split_once('=') else {
        return param.to_ascii_uppercase();
    };

    let bare = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .filter(|inner| !inner.contains([':', ';', ',']))
        .unwrap_or(value);

    format!("{}={bare}", name.to_ascii_uppercase())
}

/// The content lines of a calendar, normalized as a round trip through
/// JSCalendar may change them, sorted.
///
/// Folds are undone, names upper-cased, parameters sorted, a parameter
/// stating the default dropped, the parts of a recurrence rule and the
/// items of a `CATEGORIES` sorted (keywords are a set), an `EXDATE` or
/// `RDATE` list one line per date, and the conversion's own `JSID` keys
/// left out.
fn lines(ics: &str) -> Vec<String> {
    let mut lines: Vec<String> = ics
        .replace("\r\n", "\n")
        .replace("\n ", "")
        .replace("\n\t", "")
        .split('\n')
        .filter(|line| !line.is_empty() && !line.starts_with("JSID:"))
        .flat_map(|line| {
            let colon = split_unquoted(line, ':')[0].len();
            let (head, value) = (&line[..colon], &line[colon + 1..]);
            let mut params: Vec<String> =
                split_unquoted(head, ';').into_iter().map(param).collect();
            let name = params.remove(0);
            let default = DEFAULT_TYPES
                .iter()
                .find(|(prop, _)| *prop == name)
                .map(|(_, kind)| format!("VALUE={kind}"));
            params.retain(|param| {
                !param.starts_with("JSID=")
                    && Some(param) != default.as_ref()
                    && !DEFAULT_PARAMS.contains(&(name.as_str(), param.as_str()))
            });
            params.sort_unstable();

            let value = match name.as_str() {
                "RRULE" | "EXRULE" => {
                    let mut parts: Vec<&str> = value.split(';').collect();
                    parts.sort_unstable();
                    parts.join(";")
                }
                "CATEGORIES" => {
                    let mut items: Vec<&str> = value.split(',').collect();
                    items.sort_unstable();
                    items.join(",")
                }
                _ => value.to_string(),
            };

            let head = match params.is_empty() {
                true => name.clone(),
                false => format!("{name};{}", params.join(";")),
            };

            match name.as_str() {
                "EXDATE" | "RDATE" => value
                    .split(',')
                    .map(|item| format!("{head}:{item}"))
                    .collect::<Vec<_>>(),
                _ => vec![format!("{head}:{value}")],
            }
        })
        .collect();

    lines.sort_unstable();
    lines
}

fn fixture(name: &str) -> Vec<u8> {
    let path = format!("{}/tests/corpus/clients/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(path).expect("fixture")
}

/// What a fixture lost or gained through JSCalendar, if anything.
fn round_trip(name: &str, version: IcalJscalendarVersion) -> Option<String> {
    let ics = fixture(name);
    let group = decode(&ics).to_jscalendar_as(version);
    let back = read(&group);
    let ics = String::from_utf8(ics).expect("utf-8");

    let expected = lines(&ics);
    let actual = lines(&back);
    let missing: Vec<_> = expected.iter().filter(|l| !actual.contains(l)).collect();
    let extra: Vec<_> = actual.iter().filter(|l| !expected.contains(l)).collect();

    if !missing.is_empty() || !extra.is_empty() {
        return Some(format!(
            "{name} {version:?}\nmissing: {missing:#?}\nextra: {extra:#?}"
        ));
    }

    if decode(back.as_bytes()).to_jscalendar_as(version) != group {
        return Some(format!(
            "{name} {version:?}: a second pass changed the Group"
        ));
    }

    None
}

const FIXTURES: [&str; 4] = [
    "ios-series.ics",
    "outlook-meeting.ics",
    "google-invite.ics",
    "thunderbird-todo.ics",
];

fn round_trips(version: IcalJscalendarVersion) {
    common::each_fixture("clients", FIXTURES.len(), |_, _| {});

    let failures: Vec<String> = FIXTURES
        .iter()
        .filter_map(|name| round_trip(name, version))
        .collect();

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_client_calendar_round_trips_through_rfc_8984() {
    round_trips(IcalJscalendarVersion::V1_0);
}

#[test]
fn every_client_calendar_round_trips_through_two_point_zero() {
    round_trips(IcalJscalendarVersion::V2_0);
}
