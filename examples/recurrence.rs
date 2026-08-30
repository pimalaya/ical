//! Expand a recurrence rule, then the whole set an event actually happens on.
//!
//! A rule is only part of the answer. What a component denotes is its whole
//! recurrence set: `DTSTART` plus every `RRULE` and `RDATE`, minus every
//! `EXDATE` and `EXRULE`, with the `RECURRENCE-ID` overrides applied. Both
//! walks are lazy and both are civil, since RFC 5545 expands on the local
//! wall-clock time of `DTSTART` and never needs an offset.
//!
//! Run with: `cargo run --example recurrence`

use ical::{
    recur::{IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand, set::IcalRecurSet},
    tree::cst::IcalCst,
};

fn main() {
    // One rule on its own: the second Tuesday of every month.
    let rule = IcalRecurRule::parse("FREQ=MONTHLY;BYDAY=2TU;COUNT=4").unwrap();
    let start = IcalRecurDateTime::date(2026, 1, 1);

    println!("FREQ=MONTHLY;BYDAY=2TU from 2026-01-01:");
    for at in IcalRecurExpand::new(rule, start) {
        println!("  {}-{:02}-{:02}", at.year, at.month, at.day);
    }

    // A whole component, whose set is more than its rule: a weekly meeting with
    // one extra date bolted on, one date skipped, and one instance moved.
    let raw = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:standup@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260105T090000\r\n",
        "RRULE:FREQ=WEEKLY;COUNT=5\r\n",
        "RDATE:20260108T090000\r\n",
        "EXDATE:20260119T090000\r\n",
        "SUMMARY:Standup\r\n",
        "END:VEVENT\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:standup@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "RECURRENCE-ID:20260126T090000\r\n",
        "DTSTART:20260126T140000\r\n",
        "SUMMARY:Standup (moved to the afternoon)\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );
    let cst = IcalCst::parse(raw).unwrap();
    let cal = cst.decode();
    let set = IcalRecurSet::of_uid(&cal.components, "standup@example.com");

    println!("\nwhat the standup happens on:");
    for occurrence in set.expand() {
        let at = occurrence.start;
        let note = match occurrence.over {
            Some(_) => " (overridden)",
            None => "",
        };

        println!(
            "  {}-{:02}-{:02} {:02}:{:02}{note}",
            at.year, at.month, at.day, at.hour, at.minute,
        );
    }

    // The rule alone would have said five Mondays. The set is what the rule
    // says plus and minus everything else the component carries.
    println!("\nthe rule counted 5 instances, and the component then:");
    println!("  added {} by RDATE", set.dates.len());
    println!("  removed {} by EXDATE", set.exdates.len());
    println!("  moved {} by RECURRENCE-ID", set.overrides.len());
}
