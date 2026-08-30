//! # Temporal members
//!
//! A JSCalendar start, duration and recurrence read back as `DTSTART`,
//! `DTEND` or `DUE`, `RRULE`, `RDATE` and `EXDATE`.
//!
//! RFC 8984 states a start as a local date-time plus a `timeZone` member, so
//! the zone comes back as a `TZID` parameter and the digits as the value. A
//! `duration` becomes the end the component version of the model wants.

use alloc::{
    borrow::{Cow, ToOwned},
    format,
    string::String,
    vec::Vec,
};

use serde_json::Value;

use crate::{
    jcal::datetime::datetime_from_json,
    jscalendar::{
        hatch::IcalConverted,
        import::{RULE_ORDER, scalar, text_prop},
    },
    param::IcalParam,
    prop::{IcalProp, IcalPropKind},
    recur::IcalRecurDateTime,
    value::{
        IcalValue,
        datetime::{IcalDate, IcalDateTime},
        duration::IcalDuration,
    },
};

/// A temporal property, as a `DATE` when the object is shown without a time
/// and a `DATE-TIME` otherwise.
pub(super) fn temporal(
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
pub(super) fn occurrence(
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
pub(super) fn ends(start: &str, span: &str) -> Option<String> {
    let start = IcalRecurDateTime::parse(&basic(start)).ok()?;
    let end =
        IcalRecurDateTime::from_seconds(start.seconds() + IcalDuration::from(span).seconds()?);

    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        end.year, end.month, end.day, end.hour, end.minute, end.second
    ))
}

/// A JSCalendar date-time back in the iCalendar basic spelling.
pub(super) fn basic(text: &str) -> String {
    datetime_from_json(text).unwrap_or_else(|| text.to_owned())
}

/// A RecurrenceRule object back in the `RECUR` spelling.
pub(super) fn rule_from_json(rule: &Value, zone: Option<&str>) -> String {
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

/// The RecurrenceRule member a `RECUR` part is held in.
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

/// One NDay object back in the `BYDAY` spelling.
pub(super) fn weekday(day: &Value) -> String {
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
