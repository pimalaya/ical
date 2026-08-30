//! A calendar no standard would bless, round-tripping byte for byte, and the
//! recovering parser for the ones a strict reading throws away whole.
//!
//! Parsing is maximally liberal: the folds, the blank lines, the bare `LF`
//! endings and the `QUOTED-PRINTABLE` soft breaks a real exporter emits are all
//! resolved into one logical line and recorded, so serialization lays them back
//! out exactly.
//!
//! Run with: `cargo run --example forgiving_parse`

use ical::{component::vevent::VEVENT, prop::description::DESCRIPTION, tree::cst::IcalCst};

fn main() {
    // Folded at a column, a bare LF ending, a blank line between properties, a
    // QUOTED-PRINTABLE value broken across two physical lines.
    let messy = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\n",
        "BEGIN:VEVENT\r\n",
        "UID:messy@example.com\r\n",
        "\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DESCRIPTION;ENCODING=QUOTED-PRINTABLE:caf=\r\n",
        "=C3=A9 at the corner\r\n",
        "SUMMARY:A calendar written by a real exporter, folded acro\r\n",
        " ss two physical lines\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    let mut cst = IcalCst::parse(messy).expect("liberal in");

    // Every byte, including what the tokeniser resolved away.
    assert_eq!(cst.to_string(), messy);
    println!("round-tripped byte for byte: {}", cst.to_string() == messy);

    // The soft break is gone from the logical value the reader sees. The
    // QUOTED-PRINTABLE octets are still there: resolving those is the content
    // decoders' job, and opt-in (see the content_encodings example).
    let event = cst.component_mut::<VEVENT>().expect("the event");
    let value = event.prop_mut::<DESCRIPTION>().expect("the description");
    println!("the soft-broken value reads: {:?}", value.text());

    // A calendar a strict read refuses: a property with no colon, and a
    // component that never ends. `parse` gives up on it; `parse_recovering`
    // keeps what it cannot structure as opaque bytes and reports what it
    // worked around.
    let broken = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "THIS LINE HAS NO COLON\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:unterminated@example.com\r\n",
        "END:VCALENDAR\r\n",
    );

    println!("\nstrict parse: {:?}", IcalCst::parse(broken).is_err());

    let recovery = IcalCst::parse_recovering(broken);
    println!(
        "recovered {} calendar(s), working around:",
        recovery.calendars.len()
    );
    for problem in &recovery.problems {
        println!("  {problem}");
    }

    // What it recovered still serializes back to the bytes it came from.
    let out: String = recovery.calendars.iter().map(IcalCst::to_string).collect();
    println!("\nand still round-trips: {}", out == broken);
}
