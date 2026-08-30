//! The instances RFC 5545 3.3.10 forbids counting.
//!
//! A rule-generated instance at a local time the clock jumps over never
//! happens, so it is dropped, and dropping it costs no `COUNT` slot: a rule
//! bounded by `COUNT` yields as many occurrences as it names and runs further
//! in time to do so.
//!
//! Each case is the same zone, the RFC 5545 3.6.5 America/New_York, whose
//! spring-forward jumps 2026-03-08 02:00 to 03:00 and whose fall-back repeats
//! 2026-11-01 01:00.

#![cfg(feature = "parser")]

use ical::{
    recur::{IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand, set::IcalRecurSet},
    tree::cst::IcalCst,
    tz::{IcalTz, IcalTzOffset},
};

/// The zone every case here runs against.
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

fn zone() -> IcalTz {
    let cst = IcalCst::parse(US).expect("parse");
    let ical = cst.decode();

    IcalTz::of_calendar(&ical, "America/New_York").expect("a VTIMEZONE")
}

/// One `VEVENT` inside the same calendar the zone comes from, so a set and its
/// zone can be read from one parse.
fn set(props: &str) -> IcalRecurSet {
    let raw = format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\nBEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n{props}END:VEVENT\r\nEND:VCALENDAR\r\n"
    );
    let cst = IcalCst::parse(&raw).expect("parse");
    let ical = cst.decode();

    IcalRecurSet::of_component(&ical.components[0])
}

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

fn rule(text: &str) -> IcalRecurRule {
    IcalRecurRule::parse(text).expect("a rule")
}

#[test]
fn without_a_zone_a_gap_instance_is_yielded_like_any_other() {
    // The behaviour every caller had before the zone became an argument, and
    // the one an expansion given no zone keeps: a validity it cannot check is
    // one it does not apply.
    let dates: Vec<_> =
        IcalRecurExpand::new(rule("FREQ=DAILY;COUNT=5"), at(2026, 3, 6, 2, 30)).collect();

    assert_eq!(
        dates,
        [
            at(2026, 3, 6, 2, 30),
            at(2026, 3, 7, 2, 30),
            at(2026, 3, 8, 2, 30),
            at(2026, 3, 9, 2, 30),
            at(2026, 3, 10, 2, 30),
        ]
    );
}

#[test]
fn a_gap_instance_is_dropped_and_costs_no_count_slot() {
    // 2026-03-08 02:30 never happens, so the rule names five occurrences and
    // runs one day further to yield five.
    let dates: Vec<_> = IcalRecurExpand::new(rule("FREQ=DAILY;COUNT=5"), at(2026, 3, 6, 2, 30))
        .in_zone(zone())
        .collect();

    assert_eq!(
        dates,
        [
            at(2026, 3, 6, 2, 30),
            at(2026, 3, 7, 2, 30),
            at(2026, 3, 9, 2, 30),
            at(2026, 3, 10, 2, 30),
            at(2026, 3, 11, 2, 30),
        ]
    );
}

#[test]
fn an_until_bound_is_not_pushed_out_by_a_gap() {
    // UNTIL names an instant, not a tally, so dropping an instance leaves the
    // end of the series where it was and the count one short.
    let dates: Vec<_> = IcalRecurExpand::new(
        rule("FREQ=DAILY;UNTIL=20260310T235959"),
        at(2026, 3, 6, 2, 30),
    )
    .in_zone(zone())
    .collect();

    assert_eq!(
        dates,
        [
            at(2026, 3, 6, 2, 30),
            at(2026, 3, 7, 2, 30),
            at(2026, 3, 9, 2, 30),
            at(2026, 3, 10, 2, 30),
        ]
    );
}

#[test]
fn a_fold_instance_is_kept() {
    // 2026-11-01 01:30 happens twice, which is one time too many rather than
    // none: the RFC drops what never happens, not what happens ambiguously.
    let dates: Vec<_> = IcalRecurExpand::new(rule("FREQ=DAILY;COUNT=3"), at(2026, 10, 31, 1, 30))
        .in_zone(zone())
        .collect();

    assert_eq!(
        dates,
        [
            at(2026, 10, 31, 1, 30),
            at(2026, 11, 1, 1, 30),
            at(2026, 11, 2, 1, 30),
        ]
    );
}

#[test]
fn a_rule_whose_every_instance_is_in_a_gap_yields_nothing() {
    // The second Sunday of March at 02:30 is the transition itself, every
    // year: no period comes up barren, so the year cap is what ends the walk.
    let mut dates = IcalRecurExpand::new(
        rule("FREQ=YEARLY;BYMONTH=3;BYDAY=2SU;BYHOUR=2;BYMINUTE=30"),
        at(2026, 3, 8, 2, 30),
    )
    .in_zone(zone());

    assert_eq!(dates.next(), None);
}

#[test]
fn a_set_drops_what_its_rule_generates_in_a_gap() {
    let set = set(concat!(
        "DTSTART;TZID=America/New_York:20260306T023000\r\n",
        "RRULE:FREQ=DAILY;COUNT=5\r\n",
    ));

    let ids: Vec<_> = set
        .expand_in_zone(&zone())
        .map(|occurrence| occurrence.id)
        .collect();

    assert_eq!(
        ids,
        [
            at(2026, 3, 6, 2, 30),
            at(2026, 3, 7, 2, 30),
            at(2026, 3, 9, 2, 30),
            at(2026, 3, 10, 2, 30),
            at(2026, 3, 11, 2, 30),
        ]
    );
}

#[test]
fn a_set_keeps_the_gap_an_rdate_names() {
    // The RFC's clause is about what rules generate. An RDATE names a date
    // deliberately, as a lone DTSTART does, so both survive.
    let set = set(concat!(
        "DTSTART;TZID=America/New_York:20260308T023000\r\n",
        "RDATE;TZID=America/New_York:20260308T024500\r\n",
    ));

    let ids: Vec<_> = set
        .expand_in_zone(&zone())
        .map(|occurrence| occurrence.id)
        .collect();

    assert_eq!(ids, [at(2026, 3, 8, 2, 30), at(2026, 3, 8, 2, 45)]);
}

#[test]
fn a_zone_names_the_local_times_it_jumps_over() {
    let zone = zone();

    assert!(zone.is_gap(at(2026, 3, 8, 2, 30)));
    assert!(!zone.is_gap(at(2026, 3, 8, 1, 30)));
    assert!(!zone.is_gap(at(2026, 3, 8, 3, 30)));
    // A fold is not a gap: the time happens, twice.
    assert!(!zone.is_gap(at(2026, 11, 1, 1, 30)));
}

#[test]
fn the_crossing_to_an_instant_is_named_once() {
    let zone = zone();

    // Noon on 2026-07-15, at -0400, is 16:00 UTC.
    let local = at(2026, 7, 15, 12, 0);
    assert_eq!(
        zone.resolve(local).instant(local),
        Some(local.seconds() + 4 * 3600)
    );

    // A gap names no instant, which is the RFC's answer rather than a refusal
    // to answer.
    let local = at(2026, 3, 8, 2, 30);
    assert_eq!(zone.resolve(local).instant(local), None);

    // A fold takes the earlier of its two, a default the variant's own fields
    // still let a caller override.
    let local = at(2026, 11, 1, 1, 30);
    assert_eq!(
        zone.resolve(local).instant(local),
        Some(local.seconds() + 4 * 3600)
    );
    assert_eq!(
        IcalTzOffset::Fold {
            earlier: -4 * 3600,
            later: -5 * 3600,
        }
        .instant(local),
        Some(local.seconds() + 4 * 3600)
    );
}
