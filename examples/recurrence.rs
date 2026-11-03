//! Expand a recurrence rule, then the whole set an event actually happens on.
//!
//! A rule is only part of the answer. What a component denotes is its whole
//! recurrence set: `DTSTART` plus every `RRULE` and `RDATE`, minus every
//! `EXDATE` and `EXRULE`, with the `RECURRENCE-ID` overrides applied. Both
//! walks are lazy and both are civil, since RFC 5545 expands on the local
//! wall-clock time of `DTSTART` and never needs an offset.
//!
//! Last comes the timeline, which places each occurrence of a series on
//! instants, with its end, over a window of the caller's choosing.
//!
//! Run with: `cargo run --example recurrence`

use ical::{
    recur::{
        IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand, instant::IcalRecurTimeline,
        set::IcalRecurSet,
    },
    tree::cst::IcalCst,
    tz::IcalTz,
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

    // A rule with no end is walked from where the caller looks, not from its
    // start: `skip_to` jumps straight to the period holding that date.
    let rule = IcalRecurRule::parse("FREQ=DAILY").unwrap();
    let mut daily = IcalRecurExpand::new(rule, IcalRecurDateTime::date(1970, 1, 1));

    println!("\nFREQ=DAILY from 1970, skipped to 2026-06-01:");
    for at in daily.skip_to(IcalRecurDateTime::date(2026, 6, 1)).take(3) {
        println!("  {}-{:02}-{:02}", at.year, at.month, at.day);
    }

    // A timeline places each occurrence on the clock of the world, start and
    // end in seconds since the Unix epoch, over a window: what a calendar view
    // or an alarm scheduler asks of a series that never ends.
    let raw = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:review@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260102T150000Z\r\n",
        "DURATION:PT1H30M\r\n",
        "RRULE:FREQ=WEEKLY\r\n",
        "SUMMARY:Review\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );
    let cst = IcalCst::parse(raw).unwrap();
    let cal = cst.decode();

    // The zones a time resolves through: the calendar's own `VTIMEZONE`s,
    // none here since every time is UTC.
    let zones = IcalTz::of_components(&cal.components);
    let timeline = IcalRecurTimeline::of_uid(&cal.components, "review@example.com", &zones);

    let from = IcalRecurDateTime::date(2027, 3, 1).seconds();
    let until = IcalRecurDateTime::date(2027, 3, 22).seconds();

    println!("\nthe weekly review between 2027-03-01 and 2027-03-22, UTC:");
    for instant in timeline.expand(from, until) {
        println!("  {} to {}", utc(instant.start), utc(instant.end));
    }
}

/// An instant in seconds since the Unix epoch, as a UTC date-time.
fn utc(seconds: i64) -> String {
    let at = IcalRecurDateTime::from_seconds(seconds);

    format!(
        "{}-{:02}-{:02} {:02}:{:02}",
        at.year, at.month, at.day, at.hour, at.minute,
    )
}
