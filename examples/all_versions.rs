//! The same code reading a vCalendar 1.0 and an iCalendar 2.0 calendar, and
//! where the two genuinely differ.
//!
//! The version is a decoded indicator, never a type parameter: the syntax tree
//! ignores it, and only the codec and the per-property spec branch on it, where
//! escaping or a property's existence differ.
//!
//! Run with: `cargo run --example all_versions`

use ical::{
    component::vevent::VEVENT,
    prop::{description::DESCRIPTION, summary::SUMMARY},
    tree::{cst::IcalCst, param::altrep::ALTREP},
};

fn main() {
    // The versit format: no PRODID and no UID, and `\;` as the only value
    // escape it has.
    let versit = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:1.0\r\n",
        "BEGIN:VEVENT\r\n",
        "SUMMARY:Lunch\\; then a walk\r\n",
        "DESCRIPTION;ALTREP=\"cid:a^^b\":At the corner\r\n",
        "DTSTART:20260105T120000Z\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    // RFC 5545: the same three lines, in a calendar the modern rules govern.
    let modern = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:lunch@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "SUMMARY:Lunch\\; then a walk\r\n",
        "DESCRIPTION;ALTREP=\"cid:a^^b\":At the corner\r\n",
        "DTSTART:20260105T120000Z\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    for raw in [versit, modern] {
        let mut cst = IcalCst::parse(raw).unwrap();
        let version = cst.version();

        println!("version {}:", &*version);

        let event = cst.component_mut::<VEVENT>().expect("the event");

        // Both versions escape a literal `;` the same way, which is why one
        // model reads both.
        let summary = event.prop_mut::<SUMMARY>().expect("a summary");
        println!("  SUMMARY {:?}", summary.text());

        // Two rules the versions do not share, on one wire spelling. RFC 5545
        // 3.1 makes the double quotes a delimiter and RFC 6868 makes `^^` a
        // caret, and neither reaches vCalendar 1.0, which reads both as
        // content.
        let description = event.prop_mut::<DESCRIPTION>().expect("a description");
        println!("  ALTREP  {:?}", description.param::<ALTREP>());

        println!();
    }

    // The spec branches on the version too: a 1.0 calendar carries neither
    // PRODID nor UID, which iCalendar 2.0 requires, so what parses fine either
    // way is caught on the way out rather than on the way in.
    for raw in [versit, modern] {
        let cst = IcalCst::parse(raw).unwrap();
        let cal = cst.decode();
        let version = cal.version;

        match cal.validate() {
            Ok(_) => println!("version {} validates", &*version),
            Err(problems) => {
                println!("version {} does not validate:", &*version);
                for problem in &problems {
                    println!("  {problem}");
                }
            }
        }
    }
}
