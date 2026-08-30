//! # Temporal members
//!
//! When an object starts, how long it lasts, and what it recurs on.
//!
//! RFC 8984 states a start as a local date-time plus a `timeZone` member
//! (4.1.2, 4.1.3), never as a UTC instant with an offset, so a `DTSTART`'s
//! `TZID` becomes the zone and its digits the local time.
//!
//! A `DTEND` becomes a `duration` (4.1.4): the span between the two ends is
//! what JSCalendar carries, so an event that ended in another zone than it
//! started in comes back with the start's zone on both ends.
//!
//! An `RRULE`'s `UNTIL` is stated in UTC whenever `DTSTART` is, but RFC 8984
//! states it in the object's own time zone. Shifting between the two needs the
//! time-zone database, so the wall-clock digits are carried across unshifted.

use alloc::{borrow::ToOwned, format, string::String};

use serde_json::{Map, Value, json};

use crate::{
    jcal::datetime::datetime_to_json,
    jscalendar::export::{Builder, list, param, text},
    param::IcalParamKind,
    prop::IcalProp,
    recur::IcalRecurDateTime,
    value::{IcalValue, duration::IcalDuration},
};

impl Builder {
    /// `DTSTART` carries the start, its time zone and, for a date-only value,
    /// that the object is shown without a time (draft 2.3.16).
    pub(super) fn start(&mut self, prop: &IcalProp<'_>) {
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
    pub(super) fn due(&mut self, prop: &IcalProp<'_>) {
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
    pub(super) fn end(&mut self, prop: &IcalProp<'_>) {
        let start = self.object.get("start").and_then(Value::as_str);
        let (Some(start), Some(end)) = (start, local(prop)) else {
            return self.hatch.keep(prop);
        };

        let Some(span) = span(start, &end) else {
            return self.hatch.keep(prop);
        };

        self.member("duration", Value::String(span), prop, temporal_params(prop));
    }

    /// `RECURRENCE-ID` says which instance of a series this component is.
    pub(super) fn recurrence_id(&mut self, prop: &IcalProp<'_>) {
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

    /// `RRULE` and `EXRULE` are the recurrence rules (draft 2.3.36).
    pub(super) fn rule(&mut self, prop: &IcalProp<'_>, excluded: bool) {
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
    pub(super) fn dates(&mut self, prop: &IcalProp<'_>, excluded: bool) {
        let mut converted = false;

        for item in list(prop) {
            // NOTE: An RDATE may be a period; only its start names an
            // occurrence.
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
}

/// The parameters a temporal property's conversion consumes: its time zone
/// always, and its declared value type when that is a date, since
/// `showWithoutTime` is where the object says so.
pub(super) fn temporal_params(prop: &IcalProp<'_>) -> &'static [IcalParamKind] {
    match matches!(prop.value, IcalValue::Date(_)) {
        true => &[IcalParamKind::TzId, IcalParamKind::Value],
        false => &[IcalParamKind::TzId],
    }
}

/// The LocalDateTime a temporal property states.
pub(super) fn local(prop: &IcalProp<'_>) -> Option<String> {
    let text = text(prop)?;
    local_text(&text, matches!(prop.value, IcalValue::Date(_)))
}

/// The UTCDateTime a timestamp property states, always with its `Z`.
pub(super) fn utc(prop: &IcalProp<'_>) -> Option<String> {
    let at = local(prop)?;
    Some(format!("{at}Z"))
}

/// One date or date-time value in the JSCalendar spelling: extended, with the
/// time filled in for a date and the UTC suffix dropped, since a JSCalendar
/// date-time says its zone in a member of its own.
pub(super) fn local_text(text: &str, date_only: bool) -> Option<String> {
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
pub(super) fn zone(prop: &IcalProp<'_>) -> Option<String> {
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
pub(super) fn span(start: &str, end: &str) -> Option<String> {
    let start = IcalRecurDateTime::parse(&start.replace(['-', ':'], "")).ok()?;
    let end = IcalRecurDateTime::parse(&end.replace(['-', ':'], "")).ok()?;

    Some(
        IcalDuration::from_seconds(end.seconds() - start.seconds())
            .0
            .into_owned(),
    )
}

/// A `RECUR` value as a RecurrenceRule object (draft 2.3.36).
pub(super) fn rule_to_json(text: &str) -> Value {
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
pub(super) fn rule_member(part: &str) -> Option<&'static str> {
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
pub(super) fn nday(item: &str) -> Value {
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
