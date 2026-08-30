//! Walk a file holding several calendars, and read an envelope-less record.
//!
//! `parse` reads one calendar and stops, which is what a CalDAV resource holds.
//! A published `.ics` file often holds several, and `parse_many` iterates them,
//! keeping the blank lines between them so the file reproduces byte for byte.
//!
//! Run with: `cargo run --example calendar_file`

use ical::{component::vevent::VEVENT, prop::summary::SUMMARY, tree::cst::IcalCst};

fn main() {
    let file = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:one@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "SUMMARY:Kickoff\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
        "\r\n",
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Other//EN\r\n",
        "BEGIN:VTODO\r\n",
        "UID:two@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "SUMMARY:Write the report\r\n",
        "END:VTODO\r\n",
        "END:VCALENDAR\r\n",
    );

    let mut calendars: Vec<IcalCst<'_>> =
        IcalCst::parse_many(file).collect::<Result<_, _>>().unwrap();
    println!("{} calendars in the file", calendars.len());

    for cal in &mut calendars {
        let cal = cal.decode();
        let component = cal.components.first().expect("a component");

        println!("  {} holding a {}", &*cal.version, &*component.name);
    }

    // Concatenating what `parse_many` yields reproduces the file, the blank
    // line between the two calendars included.
    let out: String = calendars.iter().map(IcalCst::to_string).collect();
    println!("round-tripped byte for byte: {}", out == file);

    // The first calendar's own event, through its lens.
    let event = calendars[0].component_mut::<VEVENT>().expect("the event");
    let summary = event.prop_mut::<SUMMARY>().expect("a summary");
    println!("\nthe first calendar's event: {:?}", summary.text());

    // A record with no BEGIN and END envelope at all, which is what a
    // directory-style feed hands out. Its properties sit at the root.
    let bare = "SUMMARY:Standalone\r\nDTSTART:20260105T090000Z\r\n";
    let record = IcalCst::parse(bare).expect("a bare record");

    println!("\nthe bare record holds {} line(s)", record.items.len());
    println!("and round-trips: {}", record.to_string() == bare);
}
