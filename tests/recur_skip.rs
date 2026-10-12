//! Skipping a walk ahead, the bound a windowed expansion needs.
//!
//! `skip_to` must yield exactly what the full walk yields from that point on,
//! whatever the rule's shape, and must cost the window rather than the history
//! since `DTSTART`: a `FREQ=SECONDLY` rule started in 1970 otherwise steps
//! through billions of occurrences before reaching today.

#![cfg(feature = "parser")]

use ical::{
    recur::{IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand, set::IcalRecurSet},
    tree::cst::IcalCst,
};

fn at(raw: &str) -> IcalRecurDateTime {
    IcalRecurDateTime::parse(raw).unwrap()
}

/// The first `take` occurrences at or after `from`, walked in full and skipped
/// to, which must agree.
fn both(rule: &str, start: &str, from: &str, take: usize) -> Vec<IcalRecurDateTime> {
    let rule = IcalRecurRule::parse(rule).unwrap();
    let (start, from) = (at(start), at(from));

    let walked: Vec<_> = IcalRecurExpand::new(rule.clone(), start)
        .skip_while(|occurrence| *occurrence < from)
        .take(take)
        .collect();

    let mut skipping = IcalRecurExpand::new(rule, start);
    let skipped: Vec<_> = skipping.skip_to(from).take(take).collect();

    assert_eq!(skipped, walked);
    skipped
}

#[test]
fn skipping_yields_what_the_full_walk_yields() {
    let rules = [
        "FREQ=SECONDLY;INTERVAL=7",
        "FREQ=MINUTELY;INTERVAL=13;BYSECOND=5,50",
        "FREQ=HOURLY;INTERVAL=5;BYMINUTE=0,30",
        "FREQ=DAILY;INTERVAL=3",
        "FREQ=DAILY;BYMONTH=2;BYHOUR=9,17",
        "FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,FR",
        "FREQ=WEEKLY;WKST=SU;BYDAY=SA,SU",
        "FREQ=MONTHLY;BYDAY=-1FR",
        "FREQ=MONTHLY;INTERVAL=5;BYMONTHDAY=31;RSCALE=GREGORIAN;SKIP=FORWARD",
        "FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1",
        "FREQ=YEARLY;BYWEEKNO=1,52,53;BYDAY=MO",
        "FREQ=YEARLY;INTERVAL=4;BYMONTH=2;BYMONTHDAY=29",
        "FREQ=YEARLY;BYMONTH=2;BYMONTHDAY=29;RSCALE=GREGORIAN;SKIP=FORWARD",
        "FREQ=DAILY;UNTIL=20260115T000000",
        "FREQ=WEEKLY;COUNT=200",
    ];
    let froms = [
        "19970101T000000",
        "20251231T235959",
        "20260101T000000",
        "20260102T093001",
        "20261228T000000",
        "20270301T000000",
    ];

    for rule in rules {
        for from in froms {
            both(rule, "20251231T093000", from, 20);
        }
    }
}

#[test]
fn skipping_a_secondly_rule_costs_the_window_only() {
    // NOTE: Walked in full, this is 1.7 billion occurrences.
    let found = both_skipped("FREQ=SECONDLY", "19700101T000000", "20260101T000000", 3);

    assert_eq!(
        found,
        [
            at("20260101T000000"),
            at("20260101T000001"),
            at("20260101T000002")
        ]
    );
}

#[test]
fn skipping_past_year_9999_ends_the_walk() {
    let found = both_skipped("FREQ=DAILY", "20260101T000000", "99991231T000000", 3);
    assert_eq!(found, [at("99991231T000000")]);

    let rule = IcalRecurRule::parse("FREQ=DAILY").unwrap();
    let mut expand = IcalRecurExpand::new(rule, at("20260101T000000"));
    let mut beyond = at("99991231T000000");
    beyond.year = 10_000;
    assert_eq!(expand.skip_to(beyond).next(), None);
}

#[test]
fn skipping_a_whole_set_yields_what_the_full_walk_yields() {
    let raw = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:1\r\n\
        DTSTART:20200106T090000\r\n\
        RRULE:FREQ=WEEKLY;BYDAY=MO\r\n\
        RRULE:FREQ=MONTHLY;BYMONTHDAY=15\r\n\
        RDATE:20260103T090000,20200301T090000\r\n\
        EXDATE:20260112T090000\r\n\
        EXRULE:FREQ=MONTHLY;BYDAY=1MO\r\n\
        END:VEVENT\r\nEND:VCALENDAR\r\n";
    let cst = IcalCst::parse(raw).unwrap();
    let ical = cst.decode();
    let set = IcalRecurSet::of_component(&ical.components[0]);
    let from = at("20260101T000000");

    let walked: Vec<_> = set
        .expand()
        .skip_while(|occurrence| occurrence.id < from)
        .take(10)
        .collect();

    let mut skipping = set.expand();
    let skipped: Vec<_> = skipping.skip_to(from).take(10).collect();
    assert_eq!(skipped, walked);

    // NOTE: Skipping mid-walk drops the heads already pulled too.
    let mut primed = set.expand();
    primed.next();
    let skipped: Vec<_> = primed.skip_to(from).take(10).collect();
    assert_eq!(skipped, walked);
}

/// The first `take` occurrences at or after `from`, skipped to alone: the full
/// walk would take minutes.
fn both_skipped(rule: &str, start: &str, from: &str, take: usize) -> Vec<IcalRecurDateTime> {
    let rule = IcalRecurRule::parse(rule).unwrap();
    let mut expand = IcalRecurExpand::new(rule, at(start));

    expand.skip_to(at(from)).take(take).collect()
}
