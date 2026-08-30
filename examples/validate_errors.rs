//! Every violation a calendar commits, and repairing one a strict server would
//! reject.
//!
//! Parsing is liberal on purpose, so a calendar that breaks the RFC still
//! parses and still round-trips. Validation is the other half of Postel's law:
//! a runtime check over the decoded model, reporting every violation rather
//! than the first, and minting an `IcalValid` proof when nothing is left.
//!
//! Extensions always pass. Validity and conformance to a closed vocabulary are
//! different questions, and a conformant calendar may carry `X-` properties.
//!
//! Run with: `cargo run --example validate_errors`

use ical::tree::cst::IcalCst;

fn main() {
    // A calendar with no PRODID, a VEVENT with no UID and no DTSTAMP, a
    // parameter that belongs on another property, a VTIMEZONE nested where it
    // may not be, and a recurrence rule RFC 5545 3.3.10 forbids.
    let broken = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "BEGIN:VEVENT\r\n",
        "SUMMARY;PARTSTAT=ACCEPTED:Planning\r\n",
        "DTSTART:20260105T090000Z\r\n",
        "RRULE:FREQ=MONTHLY;BYWEEKNO=3\r\n",
        "BEGIN:VTIMEZONE\r\n",
        "TZID:Europe/Paris\r\n",
        "END:VTIMEZONE\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    // It parses, and it round-trips: liberal in.
    let cst = IcalCst::parse(broken).unwrap();
    assert_eq!(cst.to_string(), broken);

    println!("what a strict server would reject:");
    match cst.decode().validate() {
        Ok(_) => println!("  nothing"),
        Err(problems) => {
            for problem in &problems {
                println!("  {problem}");
            }
        }
    }

    // The same calendar with every violation repaired.
    let repaired = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:planning@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "SUMMARY:Planning\r\n",
        "DTSTART:20260105T090000Z\r\n",
        "RRULE:FREQ=YEARLY;BYWEEKNO=3\r\n",
        "X-INTERNAL-ID:4711\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );
    let cst = IcalCst::parse(repaired).unwrap();
    let valid = cst.decode().validate().expect("a conformant calendar");

    // The proof derefs to the calendar it was minted from, and converts back
    // into a byte tree for free. The `X-` property passed untouched.
    println!("\nvalidated: {} component(s)", valid.components.len());
    print!("{}", IcalCst::from(valid));
}
