//! The recurrence set a component denotes, over whole calendars.
//!
//! Each case is a calendar, parsed and decoded the way a client would, then
//! expanded through [`IcalRecurSet`]. The point is the combination: a rule is
//! only part of the answer, and what a caller needs is `DTSTART` plus every
//! `RRULE` and `RDATE`, minus every `EXDATE` and `EXRULE`, with the overrides
//! applied.

#![cfg(feature = "parser")]

use ical::{
    recur::{IcalRecurDateTime, set::IcalRecurSet},
    tree::cst::IcalCst,
};

/// The recurrence set of a calendar's first component. The set owns its parts,
/// so it outlives the calendar it was read from.
fn set_of(raw: &str) -> IcalRecurSet {
    let cst = IcalCst::parse(raw).expect("parse");
    let ical = cst.decode();

    IcalRecurSet::of_component(&ical.components[0])
}

/// The recurrence set of one `UID` across a whole calendar, overrides included.
fn set_of_uid(raw: &str, uid: &str) -> IcalRecurSet {
    let cst = IcalCst::parse(raw).expect("parse");
    let ical = cst.decode();

    IcalRecurSet::of_uid(&ical.components, uid)
}

/// The starts a set yields, at most `take` of them, as `YYYYMMDDTHHMMSS`.
fn starts(set: &IcalRecurSet, take: usize) -> Vec<String> {
    set.expand()
        .take(take)
        .map(|occurrence| text(occurrence.start))
        .collect()
}

fn text(at: IcalRecurDateTime) -> String {
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}",
        at.year, at.month, at.day, at.hour, at.minute, at.second
    )
}

/// One `VEVENT`, with whatever recurrence properties the case needs.
fn event(props: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\nBEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n{props}END:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

#[test]
fn a_rule_plus_an_extra_date_minus_an_exception() {
    // The RFC 5545 3.8.5 combination: daily for five days, one extra date
    // outside the rule, and one occurrence of the rule excepted.
    let raw = event(concat!(
        "DTSTART:20260105T090000\r\n",
        "RRULE:FREQ=DAILY;COUNT=5\r\n",
        "RDATE:20260111T140000\r\n",
        "EXDATE:20260107T090000\r\n",
    ));

    let set = set_of(&raw);

    assert_eq!(
        starts(&set, 10),
        [
            "20260105T090000",
            "20260106T090000",
            // the 7th is excepted
            "20260108T090000",
            "20260109T090000",
            "20260111T140000",
        ]
    );
}

#[test]
fn several_rules_and_a_multi_valued_rdate_merge_in_order() {
    let raw = event(concat!(
        "DTSTART:20260105T090000\r\n",
        "RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=3\r\n",
        "RRULE:FREQ=WEEKLY;BYDAY=WE;COUNT=2\r\n",
        "RDATE:20260103T120000,20260104T120000\r\n",
    ));

    let set = set_of(&raw);

    assert_eq!(
        starts(&set, 10),
        [
            "20260103T120000",
            "20260104T120000",
            "20260105T090000",
            "20260107T090000",
            "20260112T090000",
            "20260114T090000",
            "20260119T090000",
        ]
    );
}

#[test]
fn an_rdate_period_contributes_its_start() {
    let raw = event(concat!(
        "DTSTART:20260105T090000\r\n",
        "RDATE;VALUE=PERIOD:20260106T100000/PT2H\r\n",
    ));

    let set = set_of(&raw);

    assert_eq!(starts(&set, 5), ["20260105T090000", "20260106T100000"]);
}

#[test]
fn an_exrule_takes_instances_away() {
    // Daily, minus every Saturday and Sunday: the deprecated spelling of a
    // weekday rule, and still on the wire.
    let raw = event(concat!(
        "DTSTART:20260105T090000\r\n",
        "RRULE:FREQ=DAILY;COUNT=10\r\n",
        "EXRULE:FREQ=WEEKLY;BYDAY=SA,SU\r\n",
    ));

    let set = set_of(&raw);

    assert_eq!(
        starts(&set, 10),
        [
            "20260105T090000",
            "20260106T090000",
            "20260107T090000",
            "20260108T090000",
            "20260109T090000",
            "20260112T090000",
            "20260113T090000",
            "20260114T090000",
        ]
    );
}

#[test]
fn an_unbounded_rule_is_taken_from_lazily() {
    let raw = event(concat!(
        "DTSTART:20260105T090000\r\n",
        "RRULE:FREQ=DAILY\r\n",
    ));

    let set = set_of(&raw);

    assert_eq!(set.expand().take(1000).count(), 1000);
    assert_eq!(starts(&set, 2), ["20260105T090000", "20260106T090000"]);
}

#[test]
fn an_override_replaces_the_instance_it_names() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260105T090000\r\nRRULE:FREQ=DAILY;COUNT=4\r\n",
        "END:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "RECURRENCE-ID:20260107T090000\r\nDTSTART:20260107T140000\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    let set = set_of_uid(raw, "1");

    assert_eq!(
        starts(&set, 10),
        [
            "20260105T090000",
            "20260106T090000",
            "20260107T140000",
            "20260108T090000",
        ]
    );

    // The moved instance keeps the identity the rule gave it, which is what a
    // second override would have to name to replace it again.
    let moved = set.expand().nth(2).unwrap();
    assert_eq!(text(moved.id), "20260107T090000");
    assert_eq!(moved.over, Some(0));
}

#[test]
fn this_and_future_moves_the_tail_too() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260105T090000\r\nRRULE:FREQ=DAILY;COUNT=4\r\n",
        "END:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "RECURRENCE-ID;RANGE=THISANDFUTURE:20260107T090000\r\n",
        "DTSTART:20260107T100000\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    let set = set_of_uid(raw, "1");

    assert_eq!(
        starts(&set, 10),
        [
            "20260105T090000",
            "20260106T090000",
            "20260107T100000",
            // shifted by the same hour the override moved its own instance
            "20260108T100000",
        ]
    );
}

#[test]
fn an_override_of_an_instance_no_rule_generates_is_still_an_instance() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260105T090000\r\nRRULE:FREQ=WEEKLY;COUNT=2\r\n",
        "END:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "RECURRENCE-ID:20260108T090000\r\nDTSTART:20260108T110000\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    let set = set_of_uid(raw, "1");

    assert_eq!(
        starts(&set, 10),
        ["20260105T090000", "20260108T110000", "20260112T090000"]
    );
}

#[test]
fn a_component_with_no_recurrence_denotes_its_start_alone() {
    let raw = event("DTSTART:20260105T090000\r\n");
    let set = set_of(&raw);

    assert_eq!(starts(&set, 5), ["20260105T090000"]);
}

#[test]
fn a_component_with_no_start_denotes_nothing() {
    let raw = event("RRULE:FREQ=DAILY;COUNT=3\r\n");
    let set = set_of(&raw);

    assert!(starts(&set, 5).is_empty());
}

#[test]
fn a_todo_expands_the_same_way() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
        "BEGIN:VTODO\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260105T090000\r\nRRULE:FREQ=MONTHLY;COUNT=3\r\n",
        "END:VTODO\r\nEND:VCALENDAR\r\n",
    );

    let set = set_of(raw);

    assert_eq!(
        starts(&set, 5),
        ["20260105T090000", "20260205T090000", "20260305T090000"]
    );
}
