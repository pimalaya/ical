//! Quoted-printable, inline base64 and a foreign character set, kept raw then
//! decoded on demand.
//!
//! A calendar carries bytes, and the encodings they arrive in are a caller's
//! question rather than the parser's. The value is kept exactly as written, and
//! each decoder is a separate opt-in cargo feature, so a build that never sees
//! `QUOTED-PRINTABLE` pays nothing for it.
//!
//! Run with `cargo run --example content_encodings` and the three content
//! features: `--features quoted-printable,base64,encoding`.

use ical::{
    component::vevent::VEVENT,
    prop::{IcalPropKind, IcalPropName, attach::ATTACH, description::DESCRIPTION},
    tree::cst::IcalCst,
    value::IcalValue,
};

fn main() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:encoded@example.com\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260105T090000Z\r\n",
        "DESCRIPTION;ENCODING=QUOTED-PRINTABLE;CHARSET=ISO-8859-1:caf=E9 at the corner\r\n",
        "ATTACH;ENCODING=BASE64;VALUE=BINARY:SGVsbG8sIGljYWwh\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );
    let mut cst = IcalCst::parse(raw).unwrap();
    let event = cst.component_mut::<VEVENT>().expect("the event");

    {
        let description = event.prop_mut::<DESCRIPTION>().expect("a description");

        // Raw: the octets exactly as the calendar spelled them.
        println!("as written:        {:?}", description.text());

        // Resolved: the QUOTED-PRINTABLE octets, still in Latin-1.
        println!("quoted-printable:  {:?}", description.quoted_printable());

        // Transcoded: through the CHARSET the line declares, into text.
        println!("charset-decoded:   {:?}", description.charset());
    }

    {
        let attach = event.prop_mut::<ATTACH>().expect("an attachment");
        println!("\nbase64 as written: {:?}", attach.text());
    }

    // The decoded model keeps inline base64 verbatim too, and hands the bytes
    // over only when asked.
    let cal = cst.decode();
    let event = cal.components[0]
        .props
        .iter()
        .find(|prop| matches!(prop.name, IcalPropName::Kind(IcalPropKind::Attach)));

    if let Some(IcalValue::Binary(binary)) = event.map(|prop| &prop.value)
        && let Some(Ok(bytes)) = binary.decode_base64()
    {
        println!("base64-decoded:    {:?}", String::from_utf8_lossy(&bytes));
    }

    // Nothing was rewritten: the calendar comes back exactly as it arrived,
    // encodings and all.
    println!("\nround-tripped byte for byte: {}", cst.to_string() == raw);
}
