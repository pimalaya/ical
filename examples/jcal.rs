//! Write a calendar as jCal JSON and read it back.
//!
//! jCal is the RFC 7265 spelling of this model in JSON, member for member: a
//! component is `[name, [properties], [components]]` and a property is
//! `[name, {params}, type, value...]`.
//!
//! The boundary is a raw `serde_json::Value`, never a serde implementation on a
//! calendar type: one model has two JSON spellings here, jCal and JSCalendar,
//! and serde keys one representation per type.
//!
//! Run with: `cargo run --example jcal --features jcal`

use ical::{ical::Ical, tree::cst::IcalCst};

fn main() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:review@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART;TZID=Europe/Paris:20260105T090000\r\n",
        "SUMMARY;LANGUAGE=en:Design review\r\n",
        "RRULE:FREQ=WEEKLY;COUNT=4\r\n",
        "ATTENDEE;PARTSTAT=ACCEPTED:mailto:ada@example.com\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );
    let cst = IcalCst::parse(raw).unwrap();
    let cal = cst.decode();

    // Out: names lowercased, the VALUE parameter moved into the type slot, and
    // dates, times and recurrence rules re-spelled in the JSON forms.
    let jcal = cal.to_jcal();
    println!("{}", serde_json::to_string_pretty(&jcal).unwrap());

    // Back: the same model, whatever it carried.
    let back = Ical::from_jcal(&jcal).expect("a vcalendar");
    println!("\nsame model after a round trip: {}", back == cal);

    // The syntax tree is where byte fidelity lives; jCal is a projection of the
    // decoded model, so what comes back is the canonical spelling of the same
    // calendar rather than the bytes it arrived as.
    print!("\n{}", IcalCst::from(back));
}
