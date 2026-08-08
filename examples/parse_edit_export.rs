//! Parse a calendar, edit one field, and write it back.
//!
//! This shows the byte-faithful editing: only the field you touch changes,
//! every other byte of the original calendar is preserved exactly.
//!
//! Run with: `cargo run --example parse_edit_export`

use ical::tree::{component::vevent::VEVENT, cst::IcalCst, prop::summary::SUMMARY};

fn main() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example Corp//Calendar//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:42@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260102T120000Z\r\n",
        "SUMMARY:Lunch\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    let mut cal = IcalCst::parse(raw).unwrap();

    // Walk into the nested VEVENT and rename its SUMMARY in place.
    cal.component_mut::<VEVENT>()
        .unwrap()
        .prop_mut::<SUMMARY>()
        .unwrap()
        .set_text("Dinner");

    // Every untouched byte (the UID, DTSTART, PRODID, the CRLF endings) is
    // preserved; only SUMMARY changed.
    print!("{cal}");
}
