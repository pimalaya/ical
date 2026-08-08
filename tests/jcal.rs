//! The RFC 7265 jCal codec, over the examples the RFC prints and the shapes it
//! defines.
//!
//! Two things are pinned. The JSON a calendar writes, against the RFC's own
//! example, so the format is the one the RFC describes and not a paraphrase of
//! it. And the round-trip: a calendar written to jCal and read back decodes to
//! the same model, whatever it carries.

#![cfg(all(feature = "jcal", feature = "parser"))]

use ical::{ical::Ical, tree::cst::IcalCst};
use serde_json::{Value, json};

/// A calendar's jCal, and the jCal of the model read back from it.
///
/// The second is the round-trip assertion. jCal normalises two orders it has no
/// way to keep (parameters, because a JSON object is unordered, and rule parts,
/// for the same reason), so a wire comparison would be asserting the format's
/// limits rather than the codec's fidelity. A fixpoint asserts what actually
/// matters: nothing is lost on the way through.
fn round_trip(raw: &str) -> (Value, Value) {
    let cst = IcalCst::parse(raw).expect("parse");
    let model = cst.decode();

    let jcal = model.to_jcal();
    let back = Ical::from_jcal(&jcal).expect("read the jCal back");

    (jcal.clone(), back.to_jcal())
}

const SIMPLE: &str = concat!(
    "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
    "BEGIN:VEVENT\r\n",
    "UID:1\r\n",
    "DTSTAMP:20260101T000000Z\r\n",
    "DTSTART;TZID=America/New_York:20260105T090000\r\n",
    "SUMMARY:Lunch\r\n",
    "END:VEVENT\r\nEND:VCALENDAR\r\n",
);

#[test]
fn writes_the_shape_the_rfc_defines() {
    let (jcal, _) = round_trip(SIMPLE);

    assert_eq!(
        jcal,
        json!([
            "vcalendar",
            [
                ["version", {}, "text", "2.0"],
                ["prodid", {}, "text", "-//Example//EN"],
            ],
            [[
                "vevent",
                [
                    ["uid", {}, "text", "1"],
                    ["dtstamp", {}, "date-time", "2026-01-01T00:00:00Z"],
                    [
                        "dtstart",
                        { "tzid": "America/New_York" },
                        "date-time",
                        "2026-01-05T09:00:00"
                    ],
                    ["summary", {}, "text", "Lunch"],
                ],
                []
            ]]
        ])
    );
}

#[test]
fn round_trips_a_simple_calendar() {
    let (jcal, back) = round_trip(SIMPLE);
    assert_eq!(back, jcal);
}

#[test]
fn writes_every_value_kind_in_its_json_spelling() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "GEO:48.85;2.35\r\n",
        "PRIORITY:5\r\n",
        "CATEGORIES:one,two\r\n",
        "RDATE;VALUE=PERIOD:20260106T100000/20260106T120000\r\n",
        "RRULE:FREQ=WEEKLY;COUNT=3;BYDAY=MO,WE\r\n",
        "REQUEST-STATUS:2.0;Success\r\n",
        "END:VEVENT\r\nEND:VCALENDAR\r\n",
    );

    let (jcal, _) = round_trip(raw);
    let props = &jcal[2][0][1];

    assert_eq!(props[2], json!(["geo", {}, "geo", [48.85, 2.35]]));
    assert_eq!(props[3], json!(["priority", {}, "integer", 5]));
    assert_eq!(props[4], json!(["categories", {}, "text", "one", "two"]));
    assert_eq!(
        props[5],
        json!([
            "rdate",
            {},
            "period",
            "2026-01-06T10:00:00/2026-01-06T12:00:00"
        ])
    );
    assert_eq!(
        props[6],
        json!([
            "rrule",
            {},
            "recur",
            { "freq": "weekly", "count": 3, "byday": ["mo", "we"] }
        ])
    );
    assert_eq!(
        props[7],
        json!(["request-status", {}, "request-status", ["2.0", "Success"]])
    );
}

#[test]
fn round_trips_every_value_kind() {
    let raw = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
        "BEGIN:VTIMEZONE\r\nTZID:Europe/Paris\r\n",
        "BEGIN:STANDARD\r\nDTSTART:19701025T030000\r\n",
        "TZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nEND:STANDARD\r\n",
        "END:VTIMEZONE\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:1\r\nDTSTAMP:20260101T000000Z\r\n",
        "GEO:48.85;2.35\r\n",
        "PRIORITY:5\r\n",
        "CATEGORIES:one,two\r\n",
        "RDATE;VALUE=PERIOD:20260106T100000/20260106T120000\r\n",
        "EXDATE:20260107T090000,20260108T090000\r\n",
        "RRULE:FREQ=WEEKLY;COUNT=3;BYDAY=MO,WE\r\n",
        "REQUEST-STATUS:2.0;Success\r\n",
        "ATTACH;VALUE=BINARY;ENCODING=BASE64:aGk=\r\n",
        "X-VENDOR;X-PARAM=1:whatever\r\n",
        "END:VEVENT\r\nEND:VCALENDAR\r\n",
    );

    let (jcal, back) = round_trip(raw);
    assert_eq!(back, jcal);
}

#[test]
fn round_trips_a_calendar_of_every_rfc_fixture() {
    for name in [
        "rfc5545_simple_event",
        "rfc5545_recurring",
        "rfc5545_event_with_alarm",
        "rfc5545_journal",
        "rfc5545_timezone",
        "rfc5545_todo",
        "rfc7953_availability",
    ] {
        let raw =
            std::fs::read_to_string(format!("tests/corpus/rfc/{name}.ics")).expect("read fixture");
        let (jcal, back) = round_trip(&raw);
        assert_eq!(back, jcal, "{name} does not round-trip");
    }
}

#[test]
fn reads_a_jcal_it_did_not_write() {
    // Hand-written, with an unknown property, an unknown parameter and a type
    // slot naming nothing: all three survive.
    let jcal = json!([
        "vcalendar",
        [["prodid", {}, "text", "-//x//EN"]],
        [[
            "vevent",
            [
                ["uid", {}, "text", "1"],
                ["x-thing", { "x-param": "v" }, "made-up", "value"],
            ],
            []
        ]]
    ]);

    let model = Ical::from_jcal(&jcal).expect("read");
    let wire = model.encode().to_string();

    assert!(
        wire.contains("X-THING;X-PARAM=v;VALUE=MADE-UP:value\r\n"),
        "{wire}"
    );
}

#[test]
fn refuses_what_is_not_a_calendar() {
    assert!(Ical::from_jcal(&json!("nope")).is_err());
    assert!(Ical::from_jcal(&json!(["vcard", [], []])).is_err());
}
