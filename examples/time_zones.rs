//! Turn a civil local time into a UTC offset, using only the `VTIMEZONE` the
//! calendar carries.
//!
//! Expansion is civil by design, so an occurrence is a wall-clock time with no
//! offset. This is the step after: RFC 5545 3.6.5 makes a calendar carry its
//! own transition rules, so the offset can be resolved with no time-zone
//! database and no extra dependency.
//!
//! A local clock is not a bijection, and the two hard cases are reported as
//! what they are rather than guessed at: a spring-forward jumps over times that
//! never happen, a fall-back repeats times that happen twice.
//!
//! Run with: `cargo run --example time_zones`

use ical::{
    recur::IcalRecurDateTime,
    tree::cst::IcalCst,
    tz::{IcalTz, IcalTzOffset},
};

fn main() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VTIMEZONE\r\n",
        "TZID:America/New_York\r\n",
        "BEGIN:DAYLIGHT\r\n",
        "TZOFFSETFROM:-0500\r\n",
        "TZOFFSETTO:-0400\r\n",
        "TZNAME:EDT\r\n",
        "DTSTART:20070311T020000\r\n",
        "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU\r\n",
        "END:DAYLIGHT\r\n",
        "BEGIN:STANDARD\r\n",
        "TZOFFSETFROM:-0400\r\n",
        "TZOFFSETTO:-0500\r\n",
        "TZNAME:EST\r\n",
        "DTSTART:20071104T020000\r\n",
        "RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU\r\n",
        "END:STANDARD\r\n",
        "END:VTIMEZONE\r\n",
        "END:VCALENDAR\r\n",
    );
    let cst = IcalCst::parse(raw).unwrap();
    let cal = cst.decode();
    let zone = IcalTz::of_calendar(&cal, "America/New_York").expect("the zone the calendar names");

    let describe = |label: &str, at: IcalRecurDateTime| {
        match zone.resolve(at) {
            IcalTzOffset::One(offset) => println!("{label}: {}", hours(offset)),
            IcalTzOffset::Gap { before, after } => println!(
                "{label}: never happens, the clock jumped {} to {}",
                hours(before),
                hours(after),
            ),
            IcalTzOffset::Fold { earlier, later } => println!(
                "{label}: happens twice, first at {} then at {}",
                hours(earlier),
                hours(later),
            ),
        };
    };

    // Ordinary times, either side of a transition.
    describe("2026-01-15 12:00", at(2026, 1, 15, 12, 0));
    describe("2026-07-15 12:00", at(2026, 7, 15, 12, 0));

    // The two the clock cannot answer with one offset. Choosing belongs to the
    // caller, who knows whether a skipped alarm should fire early or late.
    describe("2026-03-08 02:30", at(2026, 3, 8, 2, 30));
    describe("2026-11-01 01:30", at(2026, 11, 1, 1, 30));

    // `unambiguous` is the shortcut for a caller that only wants the easy case.
    let noon = zone.resolve(at(2026, 7, 15, 12, 0));
    println!("\nunambiguous at noon in July: {:?}", noon.unambiguous());

    let ambiguous = zone.resolve(at(2026, 11, 1, 1, 30));
    println!("unambiguous in the fold:     {:?}", ambiguous.unambiguous());
}

/// A civil date-time on the hour and minute given.
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

/// An offset in seconds east of UTC, as the wire spells it.
fn hours(offset: i32) -> String {
    let sign = if offset < 0 { '-' } else { '+' };
    let offset = offset.abs();

    format!("{sign}{:02}{:02}", offset / 3600, offset % 3600 / 60)
}
