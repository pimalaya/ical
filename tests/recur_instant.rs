//! Occurrences as instants, over whole calendars.
//!
//! Each case is a calendar, parsed and decoded the way a client would, then
//! walked through [`IcalRecurTimeline`] over a window. The expected instants
//! are written out as UTC civil times, worked by hand from the zone rules.

#![cfg(feature = "parser")]

use ical::{
    recur::{
        IcalRecurDateTime,
        instant::{IcalRecurInstant, IcalRecurTimeline},
    },
    tree::cst::IcalCst,
    tz::IcalTz,
};

/// Europe/Paris and America/New_York as a calendar carries them.
const ZONES: &str = concat!(
    "BEGIN:VTIMEZONE\r\nTZID:Europe/Paris\r\n",
    "BEGIN:DAYLIGHT\r\nDTSTART:19810329T020000\r\n",
    "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU\r\n",
    "TZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nEND:DAYLIGHT\r\n",
    "BEGIN:STANDARD\r\nDTSTART:19961027T030000\r\n",
    "RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU\r\n",
    "TZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nEND:STANDARD\r\n",
    "END:VTIMEZONE\r\n",
    "BEGIN:VTIMEZONE\r\nTZID:America/New_York\r\n",
    "BEGIN:DAYLIGHT\r\nDTSTART:20070311T020000\r\n",
    "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU\r\n",
    "TZOFFSETFROM:-0500\r\nTZOFFSETTO:-0400\r\nEND:DAYLIGHT\r\n",
    "BEGIN:STANDARD\r\nDTSTART:20071104T020000\r\n",
    "RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU\r\n",
    "TZOFFSETFROM:-0400\r\nTZOFFSETTO:-0500\r\nEND:STANDARD\r\n",
    "END:VTIMEZONE\r\n",
);

/// A calendar holding the given components.
fn calendar(components: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n{components}END:VCALENDAR\r\n"
    )
}

/// One `VEVENT` of `UID:1` with the given properties.
fn event(props: &str) -> String {
    format!("BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n{props}END:VEVENT\r\n")
}

/// The occurrences of `UID:1` over `[from, until)`, through the calendar's
/// own zones.
fn walk(raw: &str, from: i64, until: i64) -> Vec<IcalRecurInstant> {
    let cst = IcalCst::parse(raw).expect("parse");
    let ical = cst.decode();
    let zones = IcalTz::of_components(&ical.components);

    IcalRecurTimeline::of_uid(&ical.components, "1", &zones)
        .expand(from, until)
        .collect()
}

/// A UTC civil time as Unix seconds.
fn utc(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> i64 {
    IcalRecurDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second: 0,
    }
    .seconds()
}

/// Every start and end of a walk.
fn spans(instants: &[IcalRecurInstant]) -> Vec<(i64, i64)> {
    instants
        .iter()
        .map(|instant| (instant.start, instant.end))
        .collect()
}

#[test]
fn a_one_day_duration_across_a_transition_lasts_twenty_five_hours() {
    let raw = calendar(&format!(
        "{ZONES}{}",
        event("DTSTART;TZID=Europe/Paris:20261024T120000\r\nDURATION:P1D\r\n")
    ));

    let instants = walk(&raw, utc(2026, 10, 1, 0, 0), utc(2026, 11, 1, 0, 0));

    // NOTE: 12:00 CEST is 10:00Z; 12:00 CET the next day is 11:00Z.
    assert_eq!(
        spans(&instants),
        [(utc(2026, 10, 24, 10, 0), utc(2026, 10, 25, 11, 0))]
    );
}

#[test]
fn an_exact_duration_across_the_same_transition_lasts_its_hours() {
    let raw = calendar(&format!(
        "{ZONES}{}",
        event("DTSTART;TZID=Europe/Paris:20261024T120000\r\nDURATION:PT24H\r\n")
    ));

    let instants = walk(&raw, utc(2026, 10, 1, 0, 0), utc(2026, 11, 1, 0, 0));

    // NOTE: 24 hours later is 11:00 Paris time, CET.
    assert_eq!(
        spans(&instants),
        [(utc(2026, 10, 24, 10, 0), utc(2026, 10, 25, 10, 0))]
    );
}

#[test]
fn an_override_in_another_zone_keeps_the_identity_it_replaced() {
    let raw = calendar(&format!(
        "{ZONES}{}{}",
        event(concat!(
            "DTSTART;TZID=Europe/Paris:20260706T090000\r\n",
            "DTEND;TZID=Europe/Paris:20260706T100000\r\n",
            "RRULE:FREQ=DAILY;COUNT=3\r\n",
        )),
        event(concat!(
            "RECURRENCE-ID;TZID=Europe/Paris:20260707T090000\r\n",
            "DTSTART;TZID=America/New_York:20260707T100000\r\n",
            "DTEND;TZID=America/New_York:20260707T113000\r\n",
        )),
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));
    let moved = instants[1];

    assert_eq!(instants.len(), 3);
    assert!(moved.over);
    assert_eq!(
        moved.id,
        IcalRecurDateTime::parse("20260707T090000").unwrap()
    );
    assert_eq!(moved.id_instant, utc(2026, 7, 7, 7, 0));

    // NOTE: 10:00 EDT is 14:00Z, and the override's own end, not the
    // series' hour, closes it.
    assert_eq!(
        (moved.start, moved.end),
        (utc(2026, 7, 7, 14, 0), utc(2026, 7, 7, 15, 30))
    );
    assert_eq!(
        (instants[2].start, instants[2].end),
        (utc(2026, 7, 8, 7, 0), utc(2026, 7, 8, 8, 0))
    );
}

#[test]
fn an_end_in_another_zone_is_read_on_its_own_clock() {
    let raw = calendar(&format!(
        "{ZONES}{}",
        event(concat!(
            "DTSTART;TZID=Europe/Paris:20260706T090000\r\n",
            "DTEND;TZID=America/New_York:20260706T090000\r\n",
            "RRULE:FREQ=DAILY;COUNT=2\r\n",
        ))
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    // NOTE: 09:00 EDT is 13:00Z, six hours after 09:00 CEST, and the next
    // instance lasts the same six (RFC 5545 3.8.5.3).
    assert_eq!(
        spans(&instants),
        [
            (utc(2026, 7, 6, 7, 0), utc(2026, 7, 6, 13, 0)),
            (utc(2026, 7, 7, 7, 0), utc(2026, 7, 7, 13, 0)),
        ]
    );
}

#[test]
fn an_all_day_series_spans_floating_days() {
    let raw = calendar(&event(
        "DTSTART;VALUE=DATE:20261024\r\nRRULE:FREQ=DAILY;COUNT=2\r\n",
    ));

    let instants = walk(&raw, utc(2026, 10, 1, 0, 0), utc(2026, 11, 1, 0, 0));

    assert_eq!(
        spans(&instants),
        [
            (utc(2026, 10, 24, 0, 0), utc(2026, 10, 25, 0, 0)),
            (utc(2026, 10, 25, 0, 0), utc(2026, 10, 26, 0, 0)),
        ]
    );
    assert!(
        instants
            .iter()
            .all(|instant| instant.date && instant.floating)
    );
}

#[test]
fn an_instance_in_a_gap_is_dropped_and_not_counted() {
    let raw = calendar(&format!(
        "{ZONES}{}",
        event("DTSTART;TZID=Europe/Paris:20260327T023000\r\nRRULE:FREQ=DAILY;COUNT=4\r\n")
    ));

    let instants = walk(&raw, utc(2026, 3, 1, 0, 0), utc(2026, 4, 1, 0, 0));
    let starts: Vec<i64> = instants.iter().map(|instant| instant.start).collect();

    // NOTE: 29 March at 02:30 never happens in Paris.
    assert_eq!(
        starts,
        [
            utc(2026, 3, 27, 1, 30),
            utc(2026, 3, 28, 1, 30),
            utc(2026, 3, 30, 0, 30),
            utc(2026, 3, 31, 0, 30),
        ]
    );
}

#[test]
fn a_window_far_from_the_start_yields_its_occurrences_alone() {
    let raw = calendar(&format!(
        "{ZONES}{}",
        event(concat!(
            "DTSTART;TZID=Europe/Paris:19900101T090000\r\n",
            "DURATION:PT1H\r\n",
            "RRULE:FREQ=DAILY\r\n",
        ))
    ));

    let instants = walk(&raw, utc(2026, 7, 6, 0, 0), utc(2026, 7, 9, 0, 0));

    assert_eq!(
        spans(&instants),
        [
            (utc(2026, 7, 6, 7, 0), utc(2026, 7, 6, 8, 0)),
            (utc(2026, 7, 7, 7, 0), utc(2026, 7, 7, 8, 0)),
            (utc(2026, 7, 8, 7, 0), utc(2026, 7, 8, 8, 0)),
        ]
    );
}

#[test]
fn an_occurrence_overlapping_the_window_start_is_yielded() {
    let raw = calendar(&event(concat!(
        "DTSTART:20260701T000000Z\r\n",
        "DURATION:P10D\r\n",
        "RRULE:FREQ=MONTHLY;COUNT=2\r\n",
    )));

    let instants = walk(&raw, utc(2026, 7, 9, 0, 0), utc(2026, 7, 20, 0, 0));

    assert_eq!(
        spans(&instants),
        [(utc(2026, 7, 1, 0, 0), utc(2026, 7, 11, 0, 0))]
    );
}

#[test]
fn an_override_moved_into_the_window_from_far_outside_is_yielded() {
    let raw = calendar(&format!(
        "{}{}",
        event("DTSTART:20200106T090000Z\r\nDURATION:PT1H\r\nRRULE:FREQ=WEEKLY\r\n"),
        event(concat!(
            "RECURRENCE-ID:20200113T090000Z\r\n",
            "DTSTART:20260708T090000Z\r\n",
            "DURATION:PT1H\r\n",
        )),
    ));

    let instants = walk(&raw, utc(2026, 7, 7, 0, 0), utc(2026, 7, 9, 0, 0));

    assert_eq!(instants.len(), 1);
    assert!(instants[0].over);
    assert_eq!(instants[0].id_instant, utc(2020, 1, 13, 9, 0));
    assert_eq!(instants[0].start, utc(2026, 7, 8, 9, 0));
}

#[test]
fn an_excluded_override_outside_the_window_stays_excluded() {
    let raw = calendar(&format!(
        "{}{}",
        event(concat!(
            "DTSTART:20200106T090000Z\r\n",
            "RRULE:FREQ=WEEKLY\r\n",
            "EXDATE:20200113T090000Z\r\n",
        )),
        event("RECURRENCE-ID:20200113T090000Z\r\nDTSTART:20260708T090000Z\r\n"),
    ));

    assert!(walk(&raw, utc(2026, 7, 7, 0, 0), utc(2026, 7, 9, 0, 0)).is_empty());
}

#[test]
fn a_series_with_no_start_yields_nothing() {
    let raw = calendar(&event("RRULE:FREQ=DAILY\r\nDURATION:PT1H\r\n"));

    assert!(walk(&raw, i64::MIN, i64::MAX).is_empty());
}

#[test]
fn a_task_with_a_due_and_no_start_is_one_point() {
    let raw = calendar(concat!(
        "BEGIN:VTODO\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "DUE:20260710T170000Z\r\nEND:VTODO\r\n",
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert_eq!(
        spans(&instants),
        [(utc(2026, 7, 10, 17, 0), utc(2026, 7, 10, 17, 0))]
    );
}

#[test]
fn an_rdate_period_ends_where_it_says() {
    let raw = calendar(&event(concat!(
        "DTSTART:20260706T090000Z\r\n",
        "DURATION:PT1H\r\n",
        "RDATE;VALUE=PERIOD:20260708T090000Z/20260708T120000Z,20260709T090000Z/PT30M\r\n",
    )));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert_eq!(
        spans(&instants),
        [
            (utc(2026, 7, 6, 9, 0), utc(2026, 7, 6, 10, 0)),
            (utc(2026, 7, 8, 9, 0), utc(2026, 7, 8, 12, 0)),
            (utc(2026, 7, 9, 9, 0), utc(2026, 7, 9, 9, 30)),
        ]
    );
}

#[test]
fn an_unknown_tzid_reads_floating() {
    let raw = calendar(&event(
        "DTSTART;TZID=Custom/Nowhere:20260706T090000\r\nDURATION:PT1H\r\n",
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert_eq!(
        spans(&instants),
        [(utc(2026, 7, 6, 9, 0), utc(2026, 7, 6, 10, 0))]
    );
    assert!(instants[0].floating);
}

#[test]
fn the_highest_sequence_is_the_series() {
    let raw = calendar(&format!(
        "{}{}",
        event("SEQUENCE:2\r\nDTSTART:20260706T090000Z\r\n"),
        event("SEQUENCE:1\r\nDTSTART:20260707T090000Z\r\n"),
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert_eq!(
        spans(&instants),
        [(utc(2026, 7, 6, 9, 0), utc(2026, 7, 6, 9, 0))]
    );
}

#[test]
fn of_two_overrides_of_one_instance_the_highest_sequence_wins() {
    let raw = calendar(&format!(
        "{}{}{}",
        event("DTSTART:20260706T090000Z\r\nRRULE:FREQ=DAILY;COUNT=2\r\n"),
        event("SEQUENCE:1\r\nRECURRENCE-ID:20260707T090000Z\r\nDTSTART:20260707T100000Z\r\n"),
        event("SEQUENCE:2\r\nRECURRENCE-ID:20260707T090000Z\r\nDTSTART:20260707T110000Z\r\n"),
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert_eq!(
        spans(&instants),
        [
            (utc(2026, 7, 6, 9, 0), utc(2026, 7, 6, 9, 0)),
            (utc(2026, 7, 7, 11, 0), utc(2026, 7, 7, 11, 0)),
        ]
    );
}

#[test]
fn a_former_series_left_behind_by_a_reschedule_is_dropped() {
    // NOTE: The shape of recurring-ical-events issue 253: the series was cut
    // short and raised to SEQUENCE 2, and the component it was before still
    // names an instance the new rule no longer reaches.
    let raw = calendar(&format!(
        "{}{}",
        event(concat!(
            "SEQUENCE:1\r\nRECURRENCE-ID;VALUE=DATE:20240729\r\n",
            "DTSTART;VALUE=DATE:20240729\r\nDTEND;VALUE=DATE:20240804\r\n",
            "RRULE:FREQ=WEEKLY;INTERVAL=2;BYDAY=MO\r\n",
        )),
        event(concat!(
            "SEQUENCE:2\r\nDTSTART;VALUE=DATE:20240701\r\nDTEND;VALUE=DATE:20240708\r\n",
            "RRULE:FREQ=WEEKLY;UNTIL=20240720;INTERVAL=2;BYDAY=MO\r\n",
        )),
    ));

    let instants = walk(&raw, utc(2024, 6, 1, 0, 0), utc(2024, 9, 1, 0, 0));

    assert_eq!(
        spans(&instants),
        [
            (utc(2024, 7, 1, 0, 0), utc(2024, 7, 8, 0, 0)),
            (utc(2024, 7, 15, 0, 0), utc(2024, 7, 22, 0, 0)),
        ]
    );
}

#[test]
fn an_override_naming_no_generated_instance_still_stands() {
    let raw = calendar(&format!(
        "{}{}",
        event("SEQUENCE:2\r\nDTSTART:20260706T090000Z\r\n"),
        event("SEQUENCE:1\r\nRECURRENCE-ID:20260708T090000Z\r\nDTSTART:20260708T100000Z\r\n"),
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert_eq!(instants.len(), 2);
    assert_eq!(instants[1].start, utc(2026, 7, 8, 10, 0));
}

#[test]
fn an_end_before_its_start_is_swapped_with_it() {
    let raw = calendar(&event(
        "DTSTART:20260706T090000Z\r\nDTEND:20260706T083000Z\r\nRRULE:FREQ=DAILY;COUNT=2\r\n",
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert_eq!(
        spans(&instants),
        [
            (utc(2026, 7, 6, 8, 30), utc(2026, 7, 6, 9, 0)),
            (utc(2026, 7, 7, 8, 30), utc(2026, 7, 7, 9, 0)),
        ]
    );
    assert_eq!(instants[0].id_instant, utc(2026, 7, 6, 9, 0));
}

#[test]
fn a_swapped_occurrence_starting_in_the_window_is_yielded() {
    // NOTE: Its identity falls after the window, its swapped start inside.
    let raw = calendar(&event(
        "DTSTART:20260710T090000Z\r\nDTEND:20260706T090000Z\r\n",
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 7, 7, 0, 0));

    assert_eq!(
        spans(&instants),
        [(utc(2026, 7, 6, 9, 0), utc(2026, 7, 10, 9, 0))]
    );
}

#[test]
fn a_task_with_no_start_recurs_from_its_due() {
    let raw = calendar(concat!(
        "BEGIN:VTODO\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "DUE:20260710T170000Z\r\nRRULE:FREQ=WEEKLY;COUNT=3\r\nEND:VTODO\r\n",
        "BEGIN:VTODO\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "RECURRENCE-ID:20260717T170000Z\r\nDUE:20260718T090000Z\r\nEND:VTODO\r\n",
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert_eq!(
        spans(&instants),
        [
            (utc(2026, 7, 10, 17, 0), utc(2026, 7, 10, 17, 0)),
            (utc(2026, 7, 18, 9, 0), utc(2026, 7, 18, 9, 0)),
            (utc(2026, 7, 24, 17, 0), utc(2026, 7, 24, 17, 0)),
        ]
    );
}

#[test]
fn an_event_with_no_start_does_not_recur_from_its_end() {
    let raw = calendar(&event(
        "DTEND:20260710T170000Z\r\nRRULE:FREQ=WEEKLY;COUNT=3\r\n",
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert!(instants.is_empty());
}

#[test]
fn a_component_with_no_uid_is_a_series_of_its_own() {
    let raw = calendar(concat!(
        "BEGIN:VEVENT\r\nDTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260706T090000Z\r\nDURATION:PT1H\r\nRRULE:FREQ=DAILY;COUNT=2\r\n",
        "END:VEVENT\r\n",
    ));
    let cst = IcalCst::parse(&raw).expect("parse");
    let ical = cst.decode();

    let instants: Vec<_> = IcalRecurTimeline::of_component(&ical.components[0], &[])
        .expand(utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0))
        .collect();

    assert_eq!(
        spans(&instants),
        [
            (utc(2026, 7, 6, 9, 0), utc(2026, 7, 6, 10, 0)),
            (utc(2026, 7, 7, 9, 0), utc(2026, 7, 7, 10, 0)),
        ]
    );
}

#[test]
fn a_this_and_future_override_carries_its_duration_forward() {
    // NOTE: The shape of recurring-ical-events issue 75: the override moves
    // the tail three hours earlier and makes it last seven hours, not two.
    let raw = calendar(&format!(
        "{}{}",
        event(concat!(
            "DTSTART:20240901T120000Z\r\nDTEND:20240901T140000Z\r\n",
            "RRULE:FREQ=DAILY;INTERVAL=2;COUNT=8\r\n",
        )),
        event(concat!(
            "RECURRENCE-ID;RANGE=THISANDFUTURE:20240905T120000Z\r\n",
            "DTSTART:20240905T090000Z\r\nDTEND:20240905T160000Z\r\n",
        )),
    ));

    let instants = walk(&raw, utc(2024, 9, 3, 0, 0), utc(2024, 9, 8, 0, 0));

    assert_eq!(
        spans(&instants),
        [
            (utc(2024, 9, 3, 12, 0), utc(2024, 9, 3, 14, 0)),
            (utc(2024, 9, 5, 9, 0), utc(2024, 9, 5, 16, 0)),
            (utc(2024, 9, 7, 9, 0), utc(2024, 9, 7, 16, 0)),
        ]
    );
}

#[test]
fn a_series_cut_before_its_start_has_no_instance() {
    // NOTE: The shape of recurring-ical-events issue 117: what a client
    // writes when the first instance and every one after it are deleted.
    let raw = calendar(&event(concat!(
        "DTSTART;VALUE=DATE:20231002\r\nDTEND;VALUE=DATE:20231009\r\n",
        "RRULE:FREQ=WEEKLY;UNTIL=20231001;INTERVAL=2;BYDAY=MO\r\n",
    )));

    let instants = walk(&raw, utc(2023, 9, 1, 0, 0), utc(2023, 12, 1, 0, 0));

    assert!(instants.is_empty());
}

#[test]
fn a_date_until_on_the_start_day_cuts_nothing() {
    let raw = calendar(&event(
        "DTSTART:20260706T090000Z\r\nRRULE:FREQ=DAILY;UNTIL=20260706\r\n",
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert_eq!(
        spans(&instants),
        [(utc(2026, 7, 6, 9, 0), utc(2026, 7, 6, 9, 0))]
    );
}

#[test]
fn a_malformed_part_is_dropped_rather_than_the_rule() {
    // NOTE: The shape of recurring-ical-events issue 128.
    let raw = calendar(&event(
        "DTSTART:20260706T090000Z\r\nRRULE:FREQ=WEEKLY;UNTIL=20260721T090000Z;COUNT=-1\r\n",
    ));

    let instants = walk(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

    assert_eq!(instants.len(), 3);
}

#[cfg(feature = "tzdb")]
mod tzdb {
    use ical::tzdb;

    use super::*;

    /// The occurrences of `UID:1` over `[from, until)`, through the
    /// calendar's own zones completed by the database.
    fn walk_with_database(raw: &str, from: i64, until: i64) -> Vec<IcalRecurInstant> {
        let cst = IcalCst::parse(raw).expect("parse");
        let ical = cst.decode();
        let zones = tzdb::zones(&ical.components, 2027);

        IcalRecurTimeline::of_uid(&ical.components, "1", &zones)
            .expand(from, until)
            .collect()
    }

    /// Europe/Berlin as a producer that truncates history writes it: two
    /// dated onsets and no rule, the earliest in October 2018.
    const TRUNCATED: &str = concat!(
        "BEGIN:VTIMEZONE\r\nTZID:Europe/Berlin\r\n",
        "BEGIN:STANDARD\r\nDTSTART:20181028T030000\r\n",
        "TZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nEND:STANDARD\r\n",
        "BEGIN:DAYLIGHT\r\nDTSTART:20190331T020000\r\n",
        "TZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nEND:DAYLIGHT\r\n",
        "END:VTIMEZONE\r\n",
    );

    #[test]
    fn a_windows_zone_name_reads_as_the_zone_cldr_maps_it_to() {
        let raw = calendar(&event(
            "DTSTART;TZID=Pacific Standard Time:20230105T100000\r\n",
        ));

        let instants = walk_with_database(&raw, utc(2023, 1, 1, 0, 0), utc(2023, 2, 1, 0, 0));

        assert_eq!(instants[0].start, utc(2023, 1, 5, 18, 0));
        assert!(!instants[0].floating);
    }

    #[test]
    fn a_time_before_a_truncated_vtimezone_reads_through_the_database() {
        let raw = calendar(&format!(
            "{TRUNCATED}{}",
            event("DTSTART;TZID=Europe/Berlin:20161203T140000\r\n"),
        ));

        let instants = walk_with_database(&raw, utc(2016, 12, 1, 0, 0), utc(2017, 1, 1, 0, 0));

        assert_eq!(instants[0].start, utc(2016, 12, 3, 13, 0));
    }

    #[test]
    fn a_time_a_truncated_vtimezone_covers_reads_through_it() {
        // NOTE: The database says +0100 in January 2019; the calendar's own
        // definition is what this checks is still read.
        let raw = calendar(&format!(
            "{}{}",
            TRUNCATED.replace("TZOFFSETTO:+0100", "TZOFFSETTO:+0300"),
            event("DTSTART;TZID=Europe/Berlin:20190110T140000\r\n"),
        ));

        let instants = walk_with_database(&raw, utc(2019, 1, 1, 0, 0), utc(2019, 2, 1, 0, 0));

        assert_eq!(instants[0].start, utc(2019, 1, 10, 11, 0));
    }

    /// The occurrences of `UID:1` once `X-WR-TIMEZONE` is applied, the zone
    /// it names completed by the database.
    fn walk_x_wr(raw: &str, from: i64, until: i64) -> Vec<IcalRecurInstant> {
        let cst = IcalCst::parse(raw).expect("parse");
        let mut ical = cst.decode();
        let mut zones = tzdb::zones(&ical.components, 2027);
        let name = IcalTz::x_wr_timezone(&ical).unwrap().to_string();
        zones.extend(tzdb::zone(&name, 2027));

        assert!(IcalTz::apply_x_wr_timezone(&mut ical, &zones));

        let zones = tzdb::zones(&ical.components, 2027);
        IcalRecurTimeline::of_uid(&ical.components, "1", &zones)
            .expand(from, until)
            .collect()
    }

    /// A calendar naming Europe/Brussels with the extension.
    fn x_wr(components: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n\
             X-WR-TIMEZONE:Europe/Brussels\r\n{components}END:VCALENDAR\r\n"
        )
    }

    #[test]
    fn a_utc_series_under_x_wr_timezone_keeps_its_wall_clock_across_a_transition() {
        let raw = x_wr(&event(
            "DTSTART:20261021T070000Z\r\nRRULE:FREQ=WEEKLY;COUNT=2\r\n",
        ));

        let instants = walk_x_wr(&raw, utc(2026, 10, 1, 0, 0), utc(2026, 11, 1, 0, 0));

        // NOTE: 09:00 in Brussels both weeks, the clock having gone back on
        // 25 October.
        assert_eq!(
            spans(&instants),
            [
                (utc(2026, 10, 21, 7, 0), utc(2026, 10, 21, 7, 0)),
                (utc(2026, 10, 28, 8, 0), utc(2026, 10, 28, 8, 0)),
            ]
        );
    }

    #[test]
    fn a_floating_time_under_x_wr_timezone_is_read_on_its_zone() {
        let raw = x_wr(&event(
            "DTSTART:20210916T210000\r\nDTEND:20210916T224500\r\n",
        ));

        let instants = walk_x_wr(&raw, utc(2021, 9, 1, 0, 0), utc(2021, 10, 1, 0, 0));

        assert_eq!(
            spans(&instants),
            [(utc(2021, 9, 16, 19, 0), utc(2021, 9, 16, 20, 45))]
        );
        assert!(!instants[0].floating);
    }

    #[test]
    fn x_wr_timezone_leaves_dates_and_named_zones_alone() {
        let raw = x_wr(&format!(
            "{ZONES}{}",
            event("DTSTART;TZID=Europe/Paris:20260706T090000\r\n"),
        ));
        let cst = IcalCst::parse(&raw).expect("parse");
        let mut ical = cst.decode();
        let mut zones = tzdb::zones(&ical.components, 2027);
        zones.extend(tzdb::zone("Europe/Brussels", 2027));

        assert_eq!(IcalTz::x_wr_timezone(&ical), Some("Europe/Brussels"));
        assert!(!IcalTz::apply_x_wr_timezone(&mut ical, &zones));
    }

    const SERIES: &str = concat!(
        "DTSTART;TZID=Europe/Paris:20260320T090000\r\n",
        "DTEND;TZID=America/New_York:20260320T090000\r\n",
        "RRULE:FREQ=DAILY;COUNT=20\r\n",
        "EXDATE:20260325T080000Z\r\n",
    );

    #[test]
    fn the_bundled_release_is_named() {
        let version = tzdb::version().expect("a release");

        assert_eq!(version.len(), 5, "{version}");
        assert!(version[..4].bytes().all(|byte| byte.is_ascii_digit()));
        assert!(version.as_bytes()[4].is_ascii_lowercase());
    }

    #[test]
    fn an_undefined_zone_resolves_as_the_defined_one() {
        let window = (utc(2026, 3, 1, 0, 0), utc(2026, 5, 1, 0, 0));

        let defined = walk(
            &calendar(&format!("{ZONES}{}", event(SERIES))),
            window.0,
            window.1,
        );
        let undefined = walk_with_database(&calendar(&event(SERIES)), window.0, window.1);

        // NOTE: Both zones move in the window, New York first, so every
        // instance counts on both being read, and the EXDATE on Paris.
        assert_eq!(defined.len(), 19);
        assert_eq!(undefined, defined);
        assert!(undefined.iter().all(|instant| !instant.floating));
    }

    #[test]
    fn a_defined_zone_wins_over_the_database() {
        // NOTE: A sender's own Paris, three hours east all year.
        let own = concat!(
            "BEGIN:VTIMEZONE\r\nTZID:Europe/Paris\r\n",
            "BEGIN:STANDARD\r\nDTSTART:19700101T000000\r\n",
            "TZOFFSETFROM:+0300\r\nTZOFFSETTO:+0300\r\nEND:STANDARD\r\n",
            "END:VTIMEZONE\r\n",
        );
        let raw = calendar(&format!(
            "{own}{}",
            event("DTSTART;TZID=Europe/Paris:20260706T090000\r\n")
        ));

        let instants = walk_with_database(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

        assert_eq!(
            spans(&instants),
            [(utc(2026, 7, 6, 6, 0), utc(2026, 7, 6, 6, 0))]
        );
    }

    #[test]
    fn a_name_the_database_does_not_know_still_reads_floating() {
        let raw = calendar(&event("DTSTART;TZID=Custom/Nowhere:20260706T090000\r\n"));

        let instants = walk_with_database(&raw, utc(2026, 7, 1, 0, 0), utc(2026, 8, 1, 0, 0));

        assert_eq!(instants[0].start, utc(2026, 7, 6, 9, 0));
        assert!(instants[0].floating);
    }

    #[test]
    fn a_database_zone_holds_every_era_up_to_its_bound() {
        let zone = tzdb::zone("America/New_York", 2026).unwrap();
        let noon = |year, month, day| IcalRecurDateTime {
            year,
            month,
            day,
            hour: 12,
            minute: 0,
            second: 0,
        };

        // NOTE: Summer time began on the first Sunday of April until 2006,
        // and on the second Sunday of March from 2007.
        assert_eq!(zone.resolve(noon(2006, 3, 20)).unambiguous(), Some(-18000));
        assert_eq!(zone.resolve(noon(2007, 3, 20)).unambiguous(), Some(-14400));
        assert_eq!(zone.resolve(noon(2026, 7, 6)).unambiguous(), Some(-14400));
        assert_eq!(zone.resolve(noon(2026, 12, 6)).unambiguous(), Some(-18000));
        assert!(tzdb::zone("Custom/Nowhere", 2026).is_none());
    }
}
