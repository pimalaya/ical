//! Offset resolution from the `VTIMEZONE` a calendar carries.
//!
//! Expansion is civil, so a caller holding an occurrence still has to turn it
//! into an instant. These cases drive that step from the zone rules inside the
//! calendar, and pin the two answers that are not a single offset: the local
//! time a spring-forward skips, and the one a fall-back gives twice.

#![cfg(feature = "parser")]

use ical::{
    component::IcalComponent,
    recur::{IcalRecurDateTime, instant::IcalRecurTimeline},
    tree::cst::IcalCst,
    tz::{IcalTz, IcalTzOffset, IcalTzTransitions},
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

#[test]
fn reads_tzuntil_without_applying_it() {
    let raw = US.replace(
        "TZID:America/New_York\r\n",
        "TZID:America/New_York\r\nTZUNTIL:20200101T000000Z\r\n",
    );
    let zone = zone(&raw, "America/New_York");

    assert_eq!(zone.until, Some(1_577_836_800));
    assert_eq!(
        zone.resolve(at(2024, 7, 15, 12, 0)),
        IcalTzOffset::One(-14_400)
    );
    assert_eq!(self::zone(US, "America/New_York").until, None);
}

#[test]
fn reads_a_written_time_by_rfc_5545_3_3_5() {
    let zone = zone(US, "America/New_York");

    // NOTE: The section's own two examples: 02:30 on the day the clock
    // springs forward is read at the offset before the gap, so as 03:30 EDT;
    // 01:30 on the day it falls back is the first of the two, in EDT.
    let gap = at(2007, 3, 11, 2, 30);
    assert_eq!(zone.resolve(gap).instant(gap), None);
    assert_eq!(zone.resolve(gap).literal_offset(), -5 * 3600);
    assert_eq!(
        zone.resolve(gap).literal_instant(gap),
        at(2007, 3, 11, 7, 30).seconds()
    );
    assert_eq!(
        zone.local(zone.resolve(gap).literal_instant(gap)),
        at(2007, 3, 11, 3, 30)
    );

    let fold = at(2007, 11, 4, 1, 30);
    assert_eq!(zone.resolve(fold).literal_offset(), -4 * 3600);
    assert_eq!(
        zone.resolve(fold).instant(fold),
        Some(zone.resolve(fold).literal_instant(fold))
    );
}

#[test]
fn tells_the_local_time_an_instant_shows() {
    let zone = zone(US, "America/New_York");

    // NOTE: Both sides of each transition, and both passes through the
    // repeated hour, which a resolution alone cannot tell apart.
    let cases = [
        (at(2026, 7, 15, 16, 0), at(2026, 7, 15, 12, 0)),
        (at(2026, 1, 15, 17, 0), at(2026, 1, 15, 12, 0)),
        (at(2026, 3, 8, 6, 59), at(2026, 3, 8, 1, 59)),
        (at(2026, 3, 8, 7, 0), at(2026, 3, 8, 3, 0)),
        (at(2026, 11, 1, 5, 30), at(2026, 11, 1, 1, 30)),
        (at(2026, 11, 1, 6, 30), at(2026, 11, 1, 1, 30)),
        (at(2026, 11, 1, 7, 0), at(2026, 11, 1, 2, 0)),
    ];

    for (utc, local) in cases {
        assert_eq!(zone.local(utc.seconds()), local, "{utc:?}");
    }
}

/// A daylight observance taking effect by `rule` from 1970, a standard one
/// in 1980, and an event in June 2026, as pimdir's hostile zone vector.
fn hostile(tzid: &str, until: &str, rule: &str) -> String {
    format!(
        concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
            "BEGIN:VTIMEZONE\r\nTZID:{tzid}\r\n{until}",
            "BEGIN:DAYLIGHT\r\nDTSTART:19700101T000000\r\n",
            "TZOFFSETFROM:+0000\r\nTZOFFSETTO:+0100\r\n{rule}\r\n",
            "END:DAYLIGHT\r\n",
            "BEGIN:STANDARD\r\nDTSTART:19800101T000000\r\n",
            "TZOFFSETFROM:+0100\r\nTZOFFSETTO:+0000\r\nEND:STANDARD\r\n",
            "END:VTIMEZONE\r\n",
            "BEGIN:VEVENT\r\nUID:hostile@example.org\r\n",
            "DTSTAMP:20260801T000000Z\r\n",
            "DTSTART;TZID={tzid}:20260601T120000\r\n",
            "DTEND;TZID={tzid}:20260601T130000\r\n",
            "END:VEVENT\r\nEND:VCALENDAR\r\n",
        ),
        tzid = tzid,
        until = until,
        rule = rule,
    )
}

/// The starts of an event's occurrences within a day of `around`, walked as
/// a calendar reader walks them.
fn starts(
    raw: &str,
    zones: impl Fn(&[IcalComponent<'_>]) -> Vec<IcalTz>,
    around: IcalRecurDateTime,
) -> Vec<i64> {
    let cst = IcalCst::parse(raw).expect("parse");
    let ical = cst.decode();
    let zones = zones(&ical.components);
    let timeline = IcalRecurTimeline::of_uid(&ical.components, "hostile@example.org", &zones);

    timeline
        .expand(around.seconds() - 86_400, around.seconds() + 86_400)
        .map(|occurrence| occurrence.start)
        .collect()
}

#[test]
fn an_observance_taking_effect_every_second_resolves_on_a_bounded_walk() {
    let june = at(2026, 6, 1, 12, 0);

    for rule in [
        "RRULE:FREQ=SECONDLY",
        "RRULE:FREQ=SECONDLY;COUNT=4000000000",
    ] {
        let raw = hostile("Hostile", "", rule);
        let zone = zone(&raw, "Hostile");

        assert!(zone.transitions(2026).len() <= IcalTz::TRANSITIONS_MAX);
        assert_eq!(zone.resolve(june), IcalTzOffset::One(0), "{rule}");
        assert_eq!(zone.local(june.seconds()), june, "{rule}");
        assert_eq!(
            IcalTzTransitions::of_zone(zone).resolve(june),
            IcalTzOffset::One(0),
            "{rule}"
        );
        assert_eq!(
            starts(&raw, IcalTz::of_components, june),
            [june.seconds()],
            "{rule}"
        );
    }
}

#[test]
fn a_zone_ruled_from_1601_still_shifts_in_9999() {
    let raw = US
        .replace("20070311T020000", "16010311T020000")
        .replace("20071104T020000", "16011104T020000");
    let zone = zone(&raw, "America/New_York");

    let cases = [
        (at(9999, 1, 15, 12, 0), -5 * 3600),
        (at(9999, 7, 1, 12, 0), -4 * 3600),
    ];

    for (local, offset) in cases {
        let mut held = IcalTzTransitions::of_zone(zone.clone());

        assert_eq!(zone.resolve(local), IcalTzOffset::One(offset), "{local:?}");
        assert_eq!(held.resolve(local), IcalTzOffset::One(offset), "{local:?}");
    }
}

#[cfg(feature = "tzdb")]
#[test]
fn a_hostile_observance_under_a_database_name_resolves_on_a_bounded_walk() {
    let june = at(2026, 6, 1, 12, 0);
    let zones = |components: &[IcalComponent<'_>]| ical::tzdb::zones(components, 2026);

    let cases = [
        (
            "TZUNTIL:20000101T000000Z\r\n",
            "RRULE:FREQ=SECONDLY",
            -2 * 3600,
        ),
        ("", "RRULE:FREQ=SECONDLY;COUNT=4000000000", 0),
    ];

    for (until, rule, shift) in cases {
        let raw = hostile("Europe/Paris", until, rule);

        assert_eq!(
            starts(&raw, zones, june),
            [june.seconds() + shift],
            "{until}{rule}"
        );
    }
}
