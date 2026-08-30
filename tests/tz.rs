//! Offset resolution from the `VTIMEZONE` a calendar carries.
//!
//! Expansion is civil, so a caller holding an occurrence still has to turn it
//! into an instant. These cases drive that step from the zone rules inside the
//! calendar, and pin the two answers that are not a single offset: the local
//! time a spring-forward skips, and the one a fall-back gives twice.

#![cfg(feature = "parser")]

use ical::{
    recur::IcalRecurDateTime,
    tree::cst::IcalCst,
    tz::{IcalTz, IcalTzOffset},
};

/// The zone a calendar defines under `tzid`.
fn zone(raw: &str, tzid: &str) -> IcalTz {
    let cst = IcalCst::parse(raw).expect("parse");
    let ical = cst.decode();

    IcalTz::of_calendar(&ical, tzid).expect("a VTIMEZONE")
}

/// The RFC 5545 3.6.5 America/New_York zone, with the rules real calendars
/// carry: daylight from the second Sunday in March, standard from the first
/// Sunday in November.
const US: &str = concat!(
    "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
    "BEGIN:VTIMEZONE\r\nTZID:America/New_York\r\n",
    "BEGIN:DAYLIGHT\r\n",
    "DTSTART:20070311T020000\r\n",
    "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU\r\n",
    "TZOFFSETFROM:-0500\r\nTZOFFSETTO:-0400\r\nTZNAME:EDT\r\n",
    "END:DAYLIGHT\r\n",
    "BEGIN:STANDARD\r\n",
    "DTSTART:20071104T020000\r\n",
    "RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU\r\n",
    "TZOFFSETFROM:-0400\r\nTZOFFSETTO:-0500\r\nTZNAME:EST\r\n",
    "END:STANDARD\r\n",
    "END:VTIMEZONE\r\nEND:VCALENDAR\r\n",
);

fn at(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> IcalRecurDateTime {
    IcalRecurDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second: 0,
    }
}

#[test]
fn resolves_summer_and_winter() {
    let zone = zone(US, "America/New_York");

    assert_eq!(
        zone.resolve(at(2026, 7, 15, 12, 0)),
        IcalTzOffset::One(-4 * 3600)
    );
    assert_eq!(
        zone.resolve(at(2026, 1, 15, 12, 0)),
        IcalTzOffset::One(-5 * 3600)
    );
}

#[test]
fn reports_the_spring_forward_gap() {
    let zone = zone(US, "America/New_York");

    // 2026-03-08 is the second Sunday of March: 02:00 becomes 03:00, so 02:30
    // never happens.
    assert_eq!(
        zone.resolve(at(2026, 3, 8, 2, 30)),
        IcalTzOffset::Gap {
            before: -5 * 3600,
            after: -4 * 3600,
        }
    );

    // Either side of the gap is unambiguous.
    assert_eq!(
        zone.resolve(at(2026, 3, 8, 1, 59)),
        IcalTzOffset::One(-5 * 3600)
    );
    assert_eq!(
        zone.resolve(at(2026, 3, 8, 3, 0)),
        IcalTzOffset::One(-4 * 3600)
    );
}

#[test]
fn reports_the_fall_back_fold() {
    let zone = zone(US, "America/New_York");

    // 2026-11-01 is the first Sunday of November: 02:00 becomes 01:00, so 01:30
    // happens twice.
    assert_eq!(
        zone.resolve(at(2026, 11, 1, 1, 30)),
        IcalTzOffset::Fold {
            earlier: -4 * 3600,
            later: -5 * 3600,
        }
    );

    assert_eq!(
        zone.resolve(at(2026, 11, 1, 0, 59)),
        IcalTzOffset::One(-4 * 3600)
    );
    assert_eq!(
        zone.resolve(at(2026, 11, 1, 2, 0)),
        IcalTzOffset::One(-5 * 3600)
    );
}

#[test]
fn a_gap_and_a_fold_have_no_single_offset() {
    let zone = zone(US, "America/New_York");

    assert_eq!(zone.resolve(at(2026, 3, 8, 2, 30)).unambiguous(), None);
    assert_eq!(zone.resolve(at(2026, 11, 1, 1, 30)).unambiguous(), None);
    assert_eq!(
        zone.resolve(at(2026, 7, 15, 12, 0)).unambiguous(),
        Some(-4 * 3600)
    );
}

#[test]
fn a_time_before_every_transition_takes_the_offset_that_came_before() {
    let zone = zone(US, "America/New_York");

    // Before the first onset the zone states, which is the March 2007 one: the
    // offset in force is what that transition says it replaced.
    assert_eq!(
        zone.resolve(at(2000, 1, 1, 12, 0)),
        IcalTzOffset::One(-5 * 3600)
    );
}

#[test]
fn resolves_from_the_rfc_fixture() {
    // The committed RFC fixture states one onset per observance and no rule, so
    // its transitions happen exactly once.
    let raw = include_str!("corpus/rfc/rfc5545_timezone.ics");
    let zone = zone(raw, "America/New_York");

    assert_eq!(zone.observances.len(), 2);
    assert_eq!(
        zone.resolve(at(2007, 6, 1, 12, 0)),
        IcalTzOffset::One(-4 * 3600)
    );
    assert_eq!(
        zone.resolve(at(2008, 1, 1, 12, 0)),
        IcalTzOffset::One(-5 * 3600)
    );
    assert_eq!(
        zone.resolve(at(2007, 3, 11, 2, 30)),
        IcalTzOffset::Gap {
            before: -5 * 3600,
            after: -4 * 3600,
        }
    );
}

#[test]
fn a_zone_that_never_shifts_resolves_to_its_one_offset() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
        "BEGIN:VTIMEZONE\r\nTZID:Asia/Kolkata\r\n",
        "BEGIN:STANDARD\r\n",
        "DTSTART:19700101T000000\r\n",
        "TZOFFSETFROM:+0530\r\nTZOFFSETTO:+0530\r\nTZNAME:IST\r\n",
        "END:STANDARD\r\n",
        "END:VTIMEZONE\r\nEND:VCALENDAR\r\n",
    );

    let zone = zone(raw, "Asia/Kolkata");

    assert_eq!(
        zone.resolve(at(2026, 7, 15, 12, 0)),
        IcalTzOffset::One(19_800)
    );
    assert_eq!(
        zone.resolve(at(1900, 7, 15, 12, 0)),
        IcalTzOffset::One(19_800)
    );
}

#[test]
fn an_unknown_tzid_is_no_zone() {
    let cst = IcalCst::parse(US).expect("parse");
    let ical = cst.decode();

    assert!(IcalTz::of_calendar(&ical, "Europe/Paris").is_none());
}
