//! Convert a calendar to a JSCalendar `Group` and back, escape hatches
//! included.
//!
//! JSCalendar is not another spelling of iCalendar, it is a different model
//! (RFC 8984): a `VCALENDAR` is a Group of Events and Tasks, a `DTEND` is a
//! duration, an `ATTENDEE` line is a Participant object, a `VALARM` is an
//! Alert, and an overriding `VEVENT` is a patch inside the series it overrides.
//!
//! Both directions are lossless through an escape hatch of their own: what the
//! mapping cannot express is kept whole in the object's `iCalendar` member, in
//! jCal syntax, and a JSCalendar member with no iCalendar counterpart comes
//! back as a `JSPROP` property located by a `JSPTR` parameter.
//!
//! Run with: `cargo run --example jscalendar --features jscalendar`

use ical::{ical::Ical, tree::cst::IcalCst};

fn main() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:offsite@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART;TZID=Europe/Paris:20260105T090000\r\n",
        "DTEND;TZID=Europe/Paris:20260105T170000\r\n",
        "SUMMARY:Team offsite\r\n",
        "LOCATION:The old mill\r\n",
        "ORGANIZER;CN=Ada:mailto:ada@example.com\r\n",
        "ATTENDEE;CN=Grace;PARTSTAT=ACCEPTED:mailto:grace@example.com\r\n",
        "X-CUSTOM-FLAG:only iCalendar knows this one\r\n",
        "BEGIN:VALARM\r\n",
        "ACTION:DISPLAY\r\n",
        "DESCRIPTION:Leave now\r\n",
        "TRIGGER:-PT30M\r\n",
        "END:VALARM\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );
    let cst = IcalCst::parse(raw).unwrap();
    let cal = cst.decode();

    // Out: a Group of one Event. `DTEND` became a duration, the two calendar
    // addresses became Participant objects, and the alarm became an Alert.
    let group = cal.to_jscalendar();
    println!("{}", serde_json::to_string_pretty(&group).unwrap());

    // Back: what JSCalendar has no member for came through the `iCalendar`
    // escape hatch, so the extension property is still here.
    let back = Ical::from_jscalendar(&group).expect("a Group");
    print!("\n{}", IcalCst::from(back.clone()));

    // The stronger law: converting the calendar that came back gives the Group
    // it came from, which is what "the escape hatch caught everything" means.
    println!(
        "\nsecond pass changes nothing: {}",
        back.to_jscalendar() == group
    );
}
