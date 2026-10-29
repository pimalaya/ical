//! When alarms fire, over whole calendars.
//!
//! Each case is a calendar, parsed and decoded the way a client would, then
//! resolved through [`IcalAlarms`] over a window. The expected instants are
//! written out as UTC civil times, worked by hand from the zone rules.

#![cfg(feature = "parser")]

use ical::{
    alarm::{IcalAlarmFiring, IcalAlarmProblem, IcalAlarmProblemKind, IcalAlarms},
    recur::IcalRecurDateTime,
    tree::cst::IcalCst,
    tz::IcalTz,
};

/// Europe/Paris as a calendar carries it.
const PARIS: &str = concat!(
    "BEGIN:VTIMEZONE\r\nTZID:Europe/Paris\r\n",
    "BEGIN:DAYLIGHT\r\nDTSTART:19810329T020000\r\n",
    "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU\r\n",
    "TZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nEND:DAYLIGHT\r\n",
    "BEGIN:STANDARD\r\nDTSTART:19961027T030000\r\n",
    "RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU\r\n",
    "TZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nEND:STANDARD\r\n",
    "END:VTIMEZONE\r\n",
);

/// A calendar holding the given components.
fn calendar(components: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n{components}END:VCALENDAR\r\n"
    )
}

/// A component of `UID:1` with the given properties and alarms.
fn item(kind: &str, props: &str, alarms: &[&str]) -> String {
    let alarms: String = alarms.iter().map(|alarm| valarm(alarm)).collect();
    format!("BEGIN:{kind}\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n{props}{alarms}END:{kind}\r\n")
}

fn event(props: &str, alarms: &[&str]) -> String {
    item("VEVENT", props, alarms)
}

fn todo(props: &str, alarms: &[&str]) -> String {
    item("VTODO", props, alarms)
}

/// A `VALARM` with the given properties.
fn valarm(props: &str) -> String {
    format!("BEGIN:VALARM\r\nACTION:DISPLAY\r\nDESCRIPTION:Hey\r\n{props}END:VALARM\r\n")
}

/// The alarms of `UID:1`, through the calendar's own zones.
fn alarms(raw: &str) -> IcalAlarms {
    let cst = IcalCst::parse(raw).expect("parse");
    let ical = cst.decode();
    let zones = IcalTz::of_components(&ical.components);
    IcalAlarms::of_uid(&ical.components, "1", &zones)
}

/// The firings of `UID:1` over `[from, until)`.
fn firings(raw: &str, from: i64, until: i64) -> Vec<IcalAlarmFiring> {
    alarms(raw).firings(from, until, None)
}

/// The instants of a list of firings.
fn instants(firings: &[IcalAlarmFiring]) -> Vec<i64> {
    firings.iter().map(|firing| firing.instant).collect()
}

/// A UTC civil time as Unix seconds.
fn utc(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> i64 {
    civil(year, month, day, hour, minute).seconds()
}

fn civil(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> IcalRecurDateTime {
    IcalRecurDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second: 0,
    }
}

/// A month around 30 October 2026.
fn october() -> (i64, i64) {
    (utc(2026, 10, 1, 0, 0), utc(2026, 11, 1, 0, 0))
}

#[test]
fn a_single_event_fires_before_its_start() {
    let raw = calendar(&event(
        "DTSTART:20261030T090000Z\r\n",
        &["TRIGGER:-PT15M\r\n"],
    ));

    let (from, until) = october();
    let firings = firings(&raw, from, until);

    assert_eq!(
        firings,
        vec![IcalAlarmFiring {
            alarm: 0,
            uid: None,
            over: None,
            occurrence: None,
            repeat: 0,
            action: Some(String::from("DISPLAY")),
            instant: utc(2026, 10, 30, 8, 45),
            acknowledged: None,
        }]
    );
}

#[test]
fn a_trigger_related_to_the_end_counts_from_it() {
    let raw = calendar(&event(
        "DTSTART:20261030T090000Z\r\nDTEND:20261030T100000Z\r\n",
        &["TRIGGER;RELATED=END:-PT5M\r\n"],
    ));

    let (from, until) = october();
    assert_eq!(
        instants(&firings(&raw, from, until)),
        vec![utc(2026, 10, 30, 9, 55)]
    );
}

#[test]
fn an_event_end_from_its_duration() {
    let raw = calendar(&event(
        "DTSTART:20261030T090000Z\r\nDURATION:PT2H\r\n",
        &["TRIGGER;RELATED=END:PT0S\r\n"],
    ));

    let (from, until) = october();
    assert_eq!(
        instants(&firings(&raw, from, until)),
        vec![utc(2026, 10, 30, 11, 0)]
    );
}

#[test]
fn a_day_before_falls_at_the_same_local_time_across_a_transition() {
    let raw = calendar(&format!(
        "{PARIS}{}",
        event(
            "DTSTART;TZID=Europe/Paris:20261025T090000\r\n",
            &["TRIGGER:-P1D\r\n"],
        )
    ));

    let (from, until) = october();
    let firings = firings(&raw, from, until);

    // NOTE: 09:00 CET on 25 October is 08:00Z, 09:00 CEST on the 24th 07:00Z.
    assert_eq!(instants(&firings), vec![utc(2026, 10, 24, 7, 0)]);
    assert_eq!(utc(2026, 10, 25, 8, 0) - firings[0].instant, 25 * 3600);
}

#[test]
fn an_absolute_trigger_fires_once_on_a_series() {
    let raw = calendar(&event(
        "DTSTART:20261028T090000Z\r\nRRULE:FREQ=DAILY\r\n",
        &["TRIGGER;VALUE=DATE-TIME:20261030T080000Z\r\n"],
    ));

    let (from, until) = october();
    let firings = firings(&raw, from, until);

    assert_eq!(instants(&firings), vec![utc(2026, 10, 30, 8, 0)]);
    assert_eq!(firings[0].occurrence, None);
}

#[test]
fn a_relative_trigger_fires_once_per_occurrence() {
    let raw = calendar(&event(
        "DTSTART:20261028T090000Z\r\nRRULE:FREQ=DAILY;COUNT=3\r\n",
        &["TRIGGER:-PT15M\r\n"],
    ));

    let (from, until) = october();
    let firings = firings(&raw, from, until);

    assert_eq!(
        instants(&firings),
        vec![
            utc(2026, 10, 28, 8, 45),
            utc(2026, 10, 29, 8, 45),
            utc(2026, 10, 30, 8, 45),
        ]
    );

    assert_eq!(firings[1].occurrence, Some(civil(2026, 10, 29, 9, 0)));
}

#[test]
fn a_weekly_series_with_a_moved_instance_fires_the_override_alarms() {
    let raw = calendar(&format!(
        "{}{}",
        event(
            "DTSTART:20261005T090000Z\r\nRRULE:FREQ=WEEKLY;COUNT=3\r\n",
            &["TRIGGER:-PT10M\r\n"],
        ),
        event(
            "RECURRENCE-ID:20261012T090000Z\r\nDTSTART:20261012T100000Z\r\n",
            &["TRIGGER:-PT30M\r\n"],
        ),
    ));

    let (from, until) = october();
    let firings = firings(&raw, from, until);

    assert_eq!(
        instants(&firings),
        vec![
            utc(2026, 10, 5, 8, 50),
            utc(2026, 10, 12, 9, 30),
            utc(2026, 10, 19, 8, 50),
        ]
    );

    let moved = civil(2026, 10, 12, 9, 0);
    assert_eq!(firings[1].over, Some(moved));
    assert_eq!(firings[1].occurrence, Some(moved));
    assert_eq!(firings[2].over, None);
}

#[test]
fn an_override_without_alarms_fires_none() {
    let raw = calendar(&format!(
        "{}{}",
        event(
            "DTSTART:20261005T090000Z\r\nRRULE:FREQ=WEEKLY;COUNT=2\r\n",
            &["TRIGGER:-PT10M\r\n"],
        ),
        event(
            "RECURRENCE-ID:20261012T090000Z\r\nDTSTART:20261012T100000Z\r\n",
            &[],
        ),
    ));

    let (from, until) = october();
    assert_eq!(
        instants(&firings(&raw, from, until)),
        vec![utc(2026, 10, 5, 8, 50)]
    );
}

#[test]
fn a_this_and_future_override_lends_its_alarms_to_later_instances() {
    let raw = calendar(&format!(
        "{}{}",
        event(
            "DTSTART:20261005T090000Z\r\nRRULE:FREQ=WEEKLY;COUNT=3\r\n",
            &["TRIGGER:-PT10M\r\n"],
        ),
        event(
            "RECURRENCE-ID;RANGE=THISANDFUTURE:20261012T090000Z\r\nDTSTART:20261012T100000Z\r\n",
            &["TRIGGER:-PT30M\r\n"],
        ),
    ));

    let (from, until) = october();
    let firings = firings(&raw, from, until);

    // NOTE: The third instance is shifted an hour too, and fires the
    // override's alarm.
    assert_eq!(
        instants(&firings),
        vec![
            utc(2026, 10, 5, 8, 50),
            utc(2026, 10, 12, 9, 30),
            utc(2026, 10, 19, 9, 30),
        ]
    );

    assert_eq!(firings[2].over, Some(civil(2026, 10, 12, 9, 0)));
}

#[test]
fn a_repetition_adds_firings() {
    let raw = calendar(&event(
        "DTSTART:20261030T090000Z\r\n",
        &["TRIGGER:-PT15M\r\nREPEAT:3\r\nDURATION:PT5M\r\n"],
    ));

    let (from, until) = october();
    let firings = firings(&raw, from, until);

    assert_eq!(
        instants(&firings),
        vec![
            utc(2026, 10, 30, 8, 45),
            utc(2026, 10, 30, 8, 50),
            utc(2026, 10, 30, 8, 55),
            utc(2026, 10, 30, 9, 0),
        ]
    );

    let ranks: Vec<u32> = firings.iter().map(|firing| firing.repeat).collect();
    assert_eq!(ranks, vec![0, 1, 2, 3]);
}

#[test]
fn a_repeat_without_a_duration_fires_once_and_is_reported() {
    let raw = calendar(&event(
        "DTSTART:20261030T090000Z\r\n",
        &["TRIGGER:-PT15M\r\nREPEAT:3\r\n"],
    ));

    let alarms = alarms(&raw);
    let (from, until) = october();

    assert_eq!(
        instants(&alarms.firings(from, until, None)),
        vec![utc(2026, 10, 30, 8, 45)]
    );

    assert_eq!(
        alarms.problems(),
        &[IcalAlarmProblem {
            alarm: 0,
            over: None,
            kind: IcalAlarmProblemKind::LoneRepeat,
        }]
    );
}

#[test]
fn a_todo_with_no_dates_fires_nothing_and_is_reported() {
    let raw = calendar(&todo("SUMMARY:Call\r\n", &["TRIGGER:-PT15M\r\n"]));

    let alarms = alarms(&raw);
    let (from, until) = october();

    assert!(alarms.firings(from, until, None).is_empty());
    assert_eq!(
        alarms.problems(),
        &[IcalAlarmProblem {
            alarm: 0,
            over: None,
            kind: IcalAlarmProblemKind::NoStart,
        }]
    );
}

#[test]
fn a_todo_started_with_no_end_reports_an_end_trigger() {
    let raw = calendar(&todo(
        "DTSTART:20261030T090000Z\r\n",
        &["TRIGGER;RELATED=END:-PT15M\r\n", "TRIGGER:PT0S\r\n"],
    ));

    let alarms = alarms(&raw);
    let (from, until) = october();

    assert_eq!(
        instants(&alarms.firings(from, until, None)),
        vec![utc(2026, 10, 30, 9, 0)]
    );

    assert_eq!(
        alarms.problems(),
        &[IcalAlarmProblem {
            alarm: 0,
            over: None,
            kind: IcalAlarmProblemKind::NoEnd,
        }]
    );
}

#[test]
fn a_todo_end_is_its_due() {
    let raw = calendar(&todo(
        "DTSTART:20261030T090000Z\r\nDUE:20261030T170000Z\r\n",
        &["TRIGGER;RELATED=END:-PT1H\r\n"],
    ));

    let (from, until) = october();
    assert_eq!(
        instants(&firings(&raw, from, until)),
        vec![utc(2026, 10, 30, 16, 0)]
    );
}

#[test]
fn a_todo_due_and_not_started_counts_from_its_due_leniently() {
    let raw = calendar(&todo("DUE:20261030T170000Z\r\n", &["TRIGGER:-PT1H\r\n"]));

    let alarms = alarms(&raw);
    let (from, until) = october();

    assert_eq!(
        instants(&alarms.firings(from, until, None)),
        vec![utc(2026, 10, 30, 16, 0)]
    );
    assert!(alarms.problems().is_empty());
}

#[test]
fn a_floating_anchor_resolves_in_the_supplied_zone() {
    let raw = calendar(&format!(
        "{PARIS}{}",
        event("DTSTART:20261030T090000\r\n", &["TRIGGER:-PT15M\r\n"])
    ));

    let cst = IcalCst::parse(&raw).expect("parse");
    let ical = cst.decode();
    let zones = IcalTz::of_components(&ical.components);
    let alarms = IcalAlarms::of_uid(&ical.components, "1", &zones);
    let (from, until) = october();

    // NOTE: 09:00 CET on 30 October is 08:00Z.
    assert_eq!(
        instants(&alarms.firings(from, until, Some(&zones[0]))),
        vec![utc(2026, 10, 30, 7, 45)]
    );

    assert_eq!(
        instants(&alarms.firings(from, until, None)),
        vec![utc(2026, 10, 30, 8, 45)]
    );
}

#[test]
fn an_alarm_identity_and_acknowledgement_are_carried() {
    let raw = calendar(&event(
        "DTSTART:20261030T090000Z\r\n",
        &["UID:alarm-1\r\nTRIGGER:-PT15M\r\nACKNOWLEDGED:20261030T084600Z\r\n"],
    ));

    let (from, until) = october();
    let firings = firings(&raw, from, until);

    assert_eq!(firings.len(), 1);
    assert_eq!(firings[0].uid.as_deref(), Some("alarm-1"));
    assert_eq!(firings[0].acknowledged, Some(utc(2026, 10, 30, 8, 46)));
}

#[test]
fn a_firing_in_the_window_for_an_occurrence_after_it() {
    let raw = calendar(&event(
        "DTSTART:20261030T090000Z\r\n",
        &["TRIGGER:-PT15M\r\n"],
    ));

    let firings = firings(&raw, utc(2026, 10, 30, 8, 0), utc(2026, 10, 30, 9, 0));
    assert_eq!(instants(&firings), vec![utc(2026, 10, 30, 8, 45)]);
}

#[test]
fn a_firing_outside_the_window_is_left_out() {
    let raw = calendar(&event(
        "DTSTART:20261030T090000Z\r\n",
        &["TRIGGER:-PT15M\r\nREPEAT:2\r\nDURATION:PT10M\r\n"],
    ));

    let firings = firings(&raw, utc(2026, 10, 30, 8, 50), utc(2026, 10, 30, 9, 0));
    assert_eq!(instants(&firings), vec![utc(2026, 10, 30, 8, 55)]);
}

#[test]
fn an_alarm_with_no_trigger_is_reported() {
    let raw = calendar(&event("DTSTART:20261030T090000Z\r\n", &[""]));

    let alarms = alarms(&raw);
    let (from, until) = october();

    assert!(alarms.firings(from, until, None).is_empty());
    assert_eq!(alarms.problems()[0].kind, IcalAlarmProblemKind::NoTrigger);
}
