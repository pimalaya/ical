//! The recurrence set a component denotes, over whole calendars.
//!
//! Each case is a calendar, parsed and decoded the way a client would, then
//! expanded through [`IcalRecurSet`]. The point is the combination: a rule is
//! only part of the answer, and what a caller needs is `DTSTART` plus every
//! `RRULE` and `RDATE`, minus every `EXDATE` and `EXRULE`, with the overrides
//! applied.

#![cfg(feature = "parser")]

use ical::{
    recur::{
        IcalRecurDateTime,
        set::{IcalRecurOverride, IcalRecurSet, IcalRecurZone},
    },
    tree::cst::IcalCst,
    tz::IcalTz,
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

/// Europe/Paris and America/New_York as a calendar carries them, the summer
/// of 2026 inside daylight time in both.
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

/// A calendar holding the zones, then the given components.
fn zoned(components: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n{ZONES}{components}END:VCALENDAR\r\n"
    )
}

/// A Paris series, daily at 09:00 local.
const SERIES: &str = "BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\nDTSTART;TZID=Europe/Paris:20260706T090000\r\n";

#[test]
fn ends_a_zoned_series_at_its_utc_until() {
    // NOTE: RFC 5545 3.3.10 has UNTIL in UTC beside a zoned DTSTART: 07:00Z
    // is 09:00 in Paris, so the third instance is the last, not dropped.
    let raw = zoned(&format!(
        "{SERIES}RRULE:FREQ=DAILY;UNTIL=20260708T070000Z\r\nEND:VEVENT\r\n"
    ));
    let set = set_of_uid(&raw, "1");

    assert_eq!(set.zone, IcalRecurZone::Tz("Europe/Paris".into()));
    assert_eq!(
        starts(&set, 10),
        ["20260706T090000", "20260707T090000", "20260708T090000"],
    );
}

#[test]
fn removes_an_instance_an_exdate_names_on_another_clock() {
    // NOTE: 07:00Z and 03:00 in New York are both 09:00 in Paris.
    let raw = zoned(&format!(
        "{SERIES}RRULE:FREQ=DAILY;COUNT=4\r\n\
         EXDATE:20260707T070000Z\r\n\
         EXDATE;TZID=America/New_York:20260708T030000\r\n\
         END:VEVENT\r\n"
    ));

    assert_eq!(
        starts(&set_of_uid(&raw, "1"), 10),
        ["20260706T090000", "20260709T090000"],
    );
}

#[test]
fn replaces_the_instance_a_utc_recurrence_id_names() {
    // NOTE: Read as written, the override named no instance and the series
    // showed the day twice, once moved and once not.
    let raw = zoned(&format!(
        "BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n\
         RECURRENCE-ID:20260707T070000Z\r\n\
         DTSTART;TZID=Europe/Paris:20260707T140000\r\n\
         END:VEVENT\r\n\
         {SERIES}RRULE:FREQ=DAILY;COUNT=3\r\nEND:VEVENT\r\n"
    ));
    let set = set_of_uid(&raw, "1");

    assert_eq!(
        starts(&set, 10),
        ["20260706T090000", "20260707T140000", "20260708T090000"],
    );
    assert_eq!(text(set.overrides[0].id), "20260707T090000");
}

#[test]
fn reads_one_override_on_the_series_clock() {
    let raw = zoned(concat!(
        "BEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "RECURRENCE-ID;TZID=America/New_York:20260707T030000\r\n",
        "DTSTART:20260707T120000Z\r\n",
        "END:VEVENT\r\n",
    ));
    let cst = IcalCst::parse(&raw).unwrap();
    let ical = cst.decode();
    let zones: Vec<IcalTz> = ical
        .components
        .iter()
        .filter_map(IcalTz::of_component)
        .collect();

    let paris = IcalRecurZone::Tz("Europe/Paris".into());
    let over = IcalRecurOverride::of_component(&ical.components[2], &paris, &zones).unwrap();

    assert_eq!(text(over.id), "20260707T090000");
    assert_eq!(text(over.start), "20260707T140000");
}

#[test]
fn compares_as_written_without_the_zones() {
    // NOTE: No VTIMEZONE to tell the clocks apart, so the UTC bound is read on
    // the start's clock, as before.
    let raw = event(concat!(
        "DTSTART;TZID=Europe/Paris:20260706T090000\r\n",
        "RRULE:FREQ=DAILY;UNTIL=20260708T070000Z\r\n",
    ));

    assert_eq!(
        starts(&set_of(&raw), 10),
        ["20260706T090000", "20260707T090000"]
    );
}
