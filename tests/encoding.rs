//! Content encodings, driven through the public API.
//!
//! Two things are checked, and the order matters. First, that the core
//! transforms nothing: a `QUOTED-PRINTABLE` value, a `BASE64` payload and a
//! value in a foreign charset all reach the caller as the bytes the wire
//! carried, with the parameters that say how to read them still attached.
//! Second, that each opt-in helper decodes what it says it decodes.
//!
//! The first half is the contract the `no_std` core owes a caller who compiled
//! none of the features in: nothing is silently mangled, and nothing needed to
//! un-mangle it later has been dropped.

#![cfg(feature = "parser")]

use std::borrow::Cow;

use ical::{
    component::vevent::VEVENT,
    param::IcalParam,
    prop::description::DESCRIPTION,
    tree::{
        codec::{Codec, mode::Escaper},
        cst::{IcalCst, IcalItem},
        line::IcalLine,
    },
    value::{IcalValue, binary::IcalBinary, uri::IcalUri},
};

/// A calendar holding one event with the given property line.
fn calendar(prop: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:encoding@example.com\r\n\
         {prop}\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n"
    )
}

/// The same, as raw bytes, for the values that are not UTF-8 to begin with.
fn raw_calendar(prop: &[u8]) -> Vec<u8> {
    let mut raw =
        b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:encoding@example.com\r\n".to_vec();
    raw.extend_from_slice(prop);
    raw.extend_from_slice(b"\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n");
    raw
}

/// The decoded parameters of the event's second property (the one under test).
fn params(ics: &str) -> Vec<IcalParam<'static>> {
    let cst = IcalCst::parse(ics).expect("a readable calendar");

    cst.decode().components[0].props[1]
        .params
        .iter()
        .map(|param| param.clone().into_owned())
        .collect()
}

#[test]
fn keeps_the_charset_parameter_on_the_decoded_model() {
    let ics = calendar("DESCRIPTION;CHARSET=ISO-8859-1:cafe");

    assert!(params(&ics).contains(&IcalParam::Charset(Cow::Borrowed("ISO-8859-1"))));
}

#[test]
fn keeps_a_value_in_a_foreign_charset_as_its_own_bytes() {
    // NOTE: 0xE9 is é in ISO-8859-1 and not valid UTF-8 at all, so a core that
    // transformed anything here would have to have lost or replaced it.
    let raw = raw_calendar(b"DESCRIPTION;CHARSET=ISO-8859-1:caf\xe9");
    let mut cst = IcalCst::parse(&raw).expect("a readable calendar");
    let event = cst.component_mut::<VEVENT>().expect("the event");
    let value = event.prop_mut::<DESCRIPTION>().expect("the description");

    assert_eq!(value.bytes().as_ref(), b"caf\xe9");
    assert_eq!(cst.to_bytes(), raw);
}

#[test]
fn keeps_quoted_printable_octets_raw_and_says_so() {
    let ics = calendar("DESCRIPTION;ENCODING=QUOTED-PRINTABLE:caf=C3=A9");
    let mut cst = IcalCst::parse(&ics).expect("a readable calendar");

    let event = cst.component_mut::<VEVENT>().expect("the event");
    let value = event.prop_mut::<DESCRIPTION>().expect("the description");

    // NOTE: The core resolves nothing: the `=C3=A9` reaches the caller as
    // written, and the parameter that says what it is comes with it.
    assert_eq!(value.bytes().as_ref(), b"caf=C3=A9");
    assert!(params(&ics).contains(&IcalParam::Encoding(Cow::Borrowed("QUOTED-PRINTABLE"))));
}

#[test]
fn keeps_a_base64_payload_verbatim() {
    let ics = calendar("ATTACH;ENCODING=BASE64;VALUE=BINARY:Zm9v");
    let cst = IcalCst::parse(&ics).expect("a readable calendar");
    let decoded = cst.decode();

    // NOTE: The decoded model holds the base64 text, not the bytes it stands
    // for: decoding is the caller's call, behind a feature.
    assert_eq!(
        decoded.components[0].props[1].value,
        IcalValue::Binary(IcalBinary::Base64(Cow::Borrowed("Zm9v")))
    );
}

#[test]
fn tells_an_inline_payload_from_a_uri_reference() {
    let uri = calendar("ATTACH:https://example.com/agenda.pdf");
    let cst = IcalCst::parse(&uri).expect("a readable calendar");
    let decoded = cst.decode();

    assert!(matches!(
        decoded.components[0].props[1].value,
        IcalValue::Uri(_)
    ));
}

#[cfg(feature = "quoted-printable")]
#[test]
fn resolves_quoted_printable_octets_on_request() {
    let ics = calendar("DESCRIPTION;ENCODING=QUOTED-PRINTABLE:caf=C3=A9");
    let mut cst = IcalCst::parse(&ics).expect("a readable calendar");

    let event = cst.component_mut::<VEVENT>().expect("the event");
    let value = event.prop_mut::<DESCRIPTION>().expect("the description");

    assert_eq!(value.quoted_printable(), "café".as_bytes());
}

#[cfg(feature = "quoted-printable")]
#[test]
fn leaves_a_value_alone_when_no_encoding_is_declared() {
    let ics = calendar("DESCRIPTION:caf=C3=A9");
    let mut cst = IcalCst::parse(&ics).expect("a readable calendar");

    let event = cst.component_mut::<VEVENT>().expect("the event");
    let value = event.prop_mut::<DESCRIPTION>().expect("the description");

    // NOTE: The octets only mean something because a parameter says they do.
    assert_eq!(value.quoted_printable(), b"caf=C3=A9");
}

#[cfg(feature = "quoted-printable")]
#[test]
fn accepts_the_vcalendar_bare_encoding_token() {
    // NOTE: vCalendar 1.0 writes the encoding as a bare parameter token rather
    // than as ENCODING=, and real files still do.
    let ics = "BEGIN:VCALENDAR\r\n\
         VERSION:1.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:bare@example.com\r\n\
         DESCRIPTION;QUOTED-PRINTABLE:caf=C3=A9\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    let mut cst = IcalCst::parse(ics).expect("a readable calendar");
    let event = cst.component_mut::<VEVENT>().expect("the event");
    let value = event.prop_mut::<DESCRIPTION>().expect("the description");

    assert_eq!(value.quoted_printable(), "café".as_bytes());
}

#[cfg(feature = "base64")]
#[test]
fn decodes_an_inline_base64_payload_on_request() {
    let ics = calendar("ATTACH;ENCODING=BASE64;VALUE=BINARY:Zm9v");
    let cst = IcalCst::parse(&ics).expect("a readable calendar");
    let decoded = cst.decode();

    let IcalValue::Binary(binary) = &decoded.components[0].props[1].value else {
        panic!("the attachment did not decode as binary");
    };

    assert_eq!(
        binary.decode_base64().expect("inline data").unwrap(),
        b"foo"
    );
}

#[cfg(feature = "base64")]
#[test]
fn decodes_nothing_for_a_uri_reference() {
    let reference = IcalBinary::Uri(Cow::Borrowed("https://example.com/agenda.pdf"));

    // NOTE: A reference embeds no data, so there is nothing to hand back, which
    // is not the same as handing back an error.
    assert!(reference.decode_base64().is_none());
}

#[cfg(feature = "base64")]
#[test]
fn reports_a_malformed_base64_payload_rather_than_guessing() {
    let broken = IcalBinary::Base64(Cow::Borrowed("not base64!"));

    assert!(broken.decode_base64().expect("inline data").is_err());
}

#[cfg(feature = "encoding")]
#[test]
fn transcodes_a_foreign_charset_to_text() {
    let raw = raw_calendar(b"DESCRIPTION;CHARSET=ISO-8859-1:caf\xe9");
    let mut cst = IcalCst::parse(&raw).expect("a readable calendar");
    let event = cst.component_mut::<VEVENT>().expect("the event");
    let value = event.prop_mut::<DESCRIPTION>().expect("the description");

    assert_eq!(value.charset(), "café");
}

#[cfg(feature = "encoding")]
#[test]
fn reads_a_value_as_utf8_when_no_charset_is_declared() {
    let ics = calendar("DESCRIPTION:café");
    let mut cst = IcalCst::parse(&ics).expect("a readable calendar");

    let event = cst.component_mut::<VEVENT>().expect("the event");
    let value = event.prop_mut::<DESCRIPTION>().expect("the description");

    assert_eq!(value.charset(), "café");
}

#[cfg(feature = "encoding")]
#[test]
fn falls_back_to_utf8_for_a_charset_label_nobody_knows() {
    let ics = calendar("DESCRIPTION;CHARSET=X-NOT-A-CHARSET:café");
    let mut cst = IcalCst::parse(&ics).expect("a readable calendar");

    let event = cst.component_mut::<VEVENT>().expect("the event");
    let value = event.prop_mut::<DESCRIPTION>().expect("the description");

    assert_eq!(value.charset(), "café");
}

#[cfg(all(feature = "encoding", feature = "quoted-printable"))]
#[test]
fn resolves_octets_before_transcoding_them() {
    // NOTE: The two encodings stack, and in one order only: `=E9` is one octet
    // of ISO-8859-1, so the quoted-printable layer has to come off first.
    let ics = calendar("DESCRIPTION;CHARSET=ISO-8859-1;ENCODING=QUOTED-PRINTABLE:caf=E9");
    let mut cst = IcalCst::parse(&ics).expect("a readable calendar");

    let event = cst.component_mut::<VEVENT>().expect("the event");
    let value = event.prop_mut::<DESCRIPTION>().expect("the description");

    assert_eq!(value.charset(), "café");
}

/// A URI carries its own `;` and `,`: RFC 5545 section 3.3.13 gives the value
/// no structure and no escaping, so reading one `;`-component alone would
/// decode a data URI to its media type and throw the payload away.
#[test]
fn a_uri_keeps_everything_past_its_first_semicolon() {
    let source = calendar("ATTACH:data:text/plain;base64,QUFB");
    let cst = IcalCst::parse(&source).expect("a calendar");

    let attach = lines(&cst)
        .find(|line| line.name.get().eq_ignore_ascii_case("ATTACH"))
        .map(|line| IcalUri::decode(&line.value))
        .expect("an attachment");

    assert_eq!(attach.0, "data:text/plain;base64,QUFB");
    assert_eq!(cst.to_string(), source, "and the calendar round trips");
}

/// The encode side is the same promise read backwards: a URI written out is
/// the reference it holds, not an escaped rendering of it.
#[test]
fn a_uri_is_written_back_without_escaping() {
    let uri = IcalUri::from("data:text/plain;base64,QUFB");
    let node = uri.encode(Escaper::Modern);

    assert_eq!(IcalUri::decode(&node).0, "data:text/plain;base64,QUFB");
}

/// Every property line of a calendar, its nested components included.
fn lines<'a>(cst: &'a IcalCst<'a>) -> impl Iterator<Item = &'a IcalLine<'a>> {
    cst.items.iter().flat_map(|item| match item {
        IcalItem::Prop(line) => vec![line],
        IcalItem::Component(inner) => lines(inner).collect(),
        IcalItem::Opaque(_) => Vec::new(),
    })
}
