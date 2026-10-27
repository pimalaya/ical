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
