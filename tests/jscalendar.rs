#![cfg(feature = "jscalendar")]

//! The RFC 8984 JSCalendar conversion, both ways.
//!
//! Two questions are asked of every case. Does the conversion say what the RFC
//! says it should, and does a second pass through it change anything? The
//! second is the stronger of the two: a Group converted to a calendar and back
//! must equal the Group it started as, whatever the mapping did in between,
//! because that is what "the escape hatch caught everything" means.

mod common;

use ical::{ical::Ical, tree::cst::IcalCst};
use serde_json::{Value, json};

/// A calendar's decoded model, from its wire bytes.
fn decode(ics: &str) -> Ical<'static> {
    IcalCst::parse(ics)
        .expect("a readable calendar")
        .decode()
        .into_owned()
}

/// The Group a calendar converts to.
fn group(ics: &str) -> Value {
    decode(ics).to_jscalendar()
}

/// The Group a Group converts to, once round-tripped through iCalendar.
fn again(group: &Value) -> Value {
    Ical::from_jscalendar(group)
        .expect("a readable JSCalendar object")
        .to_jscalendar()
}

/// The member at a top-level entry of a Group.
fn entry(group: &Value, member: &str) -> Value {
    group["entries"][0][member].clone()
}

#[test]
fn converts_the_calendar_envelope_to_a_group() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//Example//EN\r\n\
         UID:41aa02b6\r\n\
         NAME:Team calendar\r\n\
         END:VCALENDAR\r\n",
    );

    assert_eq!(group["@type"], json!("Group"));
    assert_eq!(group["uid"], json!("41aa02b6"));
    assert_eq!(group["prodId"], json!("-//Example//EN"));
    assert_eq!(group["title"], json!("Team calendar"));
}

#[test]
fn converts_an_event_to_an_event_object() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:DE935D01\r\n\
         DTSTART;TZID=Europe/Berlin:20240101T140000\r\n\
         DTEND;TZID=Europe/Berlin:20240101T153000\r\n\
         SUMMARY:Sprint review\r\n\
         DESCRIPTION:Bring the demo\r\n\
         SEQUENCE:3\r\n\
         PRIORITY:5\r\n\
         CLASS:CONFIDENTIAL\r\n\
         STATUS:CONFIRMED\r\n\
         TRANSP:TRANSPARENT\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
    );

    assert_eq!(entry(&group, "@type"), json!("Event"));
    assert_eq!(entry(&group, "uid"), json!("DE935D01"));
    assert_eq!(entry(&group, "start"), json!("2024-01-01T14:00:00"));
    assert_eq!(entry(&group, "timeZone"), json!("Europe/Berlin"));
    assert_eq!(entry(&group, "duration"), json!("PT1H30M"));
    assert_eq!(entry(&group, "title"), json!("Sprint review"));
    assert_eq!(entry(&group, "description"), json!("Bring the demo"));
    assert_eq!(entry(&group, "sequence"), json!(3));
    assert_eq!(entry(&group, "priority"), json!(5));
    // NOTE: The one privacy word JSCalendar does not share with iCalendar.
    assert_eq!(entry(&group, "privacy"), json!("secret"));
    assert_eq!(entry(&group, "status"), json!("confirmed"));
    assert_eq!(entry(&group, "freeBusyStatus"), json!("free"));
}

#[test]
fn shows_a_date_only_event_without_a_time() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:1\r\n\
         DTSTART;VALUE=DATE:20240101\r\n\
         DTEND;VALUE=DATE:20240102\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
    );

    assert_eq!(entry(&group, "start"), json!("2024-01-01T00:00:00"));
    assert_eq!(entry(&group, "showWithoutTime"), json!(true));
    assert_eq!(entry(&group, "duration"), json!("P1D"));
    assert_eq!(again(&group), group);
}

#[test]
fn converts_a_todo_to_a_task_object() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VTODO\r\n\
         UID:2\r\n\
         DUE;TZID=America/New_York:20240301T170000\r\n\
         PERCENT-COMPLETE:40\r\n\
         STATUS:IN-PROCESS\r\n\
         END:VTODO\r\n\
         END:VCALENDAR\r\n",
    );

    assert_eq!(entry(&group, "@type"), json!("Task"));
    assert_eq!(entry(&group, "due"), json!("2024-03-01T17:00:00"));
    assert_eq!(entry(&group, "timeZone"), json!("America/New_York"));
    assert_eq!(entry(&group, "percentComplete"), json!(40));
    // NOTE: A to-do's STATUS is its progress, not its status (draft 2.3.39).
    assert_eq!(entry(&group, "progress"), json!("in-process"));
}

#[test]
fn converts_a_recurrence_rule_to_a_rule_object() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:3\r\n\
         DTSTART:20240101T010000Z\r\n\
         RRULE:FREQ=YEARLY;INTERVAL=2;BYMONTH=1;BYDAY=-1SU;BYHOUR=8,9\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
    );

    assert_eq!(
        entry(&group, "recurrenceRules"),
        json!([{
            "@type": "RecurrenceRule",
            "frequency": "yearly",
            "interval": 2,
            "byDay": [{"@type": "NDay", "day": "su", "nthOfPeriod": -1}],
            "byHour": [8, 9],
            "byMonth": ["1"],
        }])
    );

    assert_eq!(again(&group), group);
}

#[test]
fn folds_a_recurrence_override_into_the_series_it_overrides() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:4\r\n\
         DTSTART;TZID=Europe/Berlin:20240101T140000\r\n\
         SUMMARY:Standup\r\n\
         RRULE:FREQ=DAILY\r\n\
         END:VEVENT\r\n\
         BEGIN:VEVENT\r\n\
         UID:4\r\n\
         RECURRENCE-ID;TZID=Europe/Berlin:20240202T140000\r\n\
         DTSTART;TZID=Europe/Berlin:20240202T160000\r\n\
         SUMMARY:Standup\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
    );

    // NOTE: One entry, not two: the override is a patch inside its series.
    assert_eq!(group["entries"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        entry(&group, "recurrenceOverrides"),
        json!({"2024-02-02T14:00:00": {"start": "2024-02-02T16:00:00"}})
    );

    assert_eq!(again(&group), group);
}

#[test]
fn keeps_a_standalone_instance_as_an_entry_of_its_own() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:5\r\n\
         RECURRENCE-ID:20240202T140000Z\r\n\
         DTSTART:20240202T160000Z\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
    );

    assert_eq!(group["entries"].as_array().map(Vec::len), Some(1));
    assert_eq!(entry(&group, "recurrenceId"), json!("2024-02-02T14:00:00"));
    assert_eq!(entry(&group, "recurrenceIdTimeZone"), json!("Etc/UTC"));
}

#[test]
fn converts_added_and_excluded_dates_to_overrides() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:6\r\n\
         DTSTART:20240101T090000Z\r\n\
         RRULE:FREQ=WEEKLY\r\n\
         RDATE:20240110T090000Z\r\n\
         EXDATE:20240108T090000Z\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
    );

    assert_eq!(
        entry(&group, "recurrenceOverrides"),
        json!({
            "2024-01-08T09:00:00": {"excluded": true},
            "2024-01-10T09:00:00": {},
        })
    );
}

#[test]
fn converts_an_attendee_to_a_participant() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:7\r\n\
         ORGANIZER:mailto:chair@example.com\r\n\
         ATTENDEE;RSVP=TRUE;PARTSTAT=TENTATIVE;ROLE=OPT-PARTICIPANT;CN=Henry Cabot:mailto:hcabot@example.com\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
    );

    assert_eq!(
        entry(&group, "replyTo"),
        json!({"imip": "mailto:chair@example.com"})
    );

    let participants = entry(&group, "participants");
    let attendee = &participants["2"];

    assert_eq!(attendee["name"], json!("Henry Cabot"));
    assert_eq!(attendee["participationStatus"], json!("tentative"));
    assert_eq!(attendee["expectReply"], json!(true));
    assert_eq!(
        attendee["roles"],
        json!({"attendee": true, "optional": true})
    );
    assert_eq!(
        attendee["sendTo"],
        json!({"imip": "mailto:hcabot@example.com"})
    );

    // NOTE: The organizer is a participant too, and owns the object.
    assert_eq!(participants["1"]["roles"], json!({"owner": true}));
}

#[test]
fn converts_an_alarm_to_an_alert() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:8\r\n\
         DTSTART:20240101T090000Z\r\n\
         BEGIN:VALARM\r\n\
         UID:04DC2968\r\n\
         TRIGGER;RELATED=END:-PT30M\r\n\
         ACTION:DISPLAY\r\n\
         DESCRIPTION:Breakfast meeting\r\n\
         END:VALARM\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
    );

    let alert = &entry(&group, "alerts")["04DC2968"];

    assert_eq!(alert["action"], json!("display"));
    assert_eq!(
        alert["trigger"],
        json!({"@type": "OffsetTrigger", "offset": "-PT30M", "relativeTo": "end"})
    );

    // NOTE: An alarm's UID and DESCRIPTION are both unmappable, and the second
    // is mandatory in iCalendar, so the hatch is where they have to be (draft
    // 2.2.2, Figure 10).
    assert_eq!(
        alert["iCalendar"]["properties"],
        json!([
            ["uid", {}, "text", "04DC2968"],
            ["description", {}, "text", "Breakfast meeting"],
        ])
    );

    assert_eq!(again(&group), group);
}

#[test]
fn keeps_an_unmapped_property_in_the_escape_hatch() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         UID:9\r\n\
         SUMMARY;X-FOO=bar:test\r\n\
         X-BAR:bam\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
    );

    let hatch = entry(&group, "iCalendar");

    assert_eq!(hatch["name"], json!("vevent"));
    assert_eq!(
        hatch["properties"],
        json!([["x-bar", {}, "unknown", "bam"]])
    );

    // NOTE: A parameter the mapping cannot place is recorded against the member
    // its property became, which is what makes the round trip exact.
    assert_eq!(
        hatch["convertedProperties"]["title"],
        json!({"@type": "ICalProperty", "name": "summary", "parameters": {"x-foo": "bar"}})
    );

    assert_eq!(again(&group), group);
}

#[test]
fn keeps_an_unmapped_component_in_the_escape_hatch() {
    let group = group(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VTIMEZONE\r\n\
         TZID:Europe/Berlin\r\n\
         END:VTIMEZONE\r\n\
         BEGIN:VEVENT\r\n\
         UID:10\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
    );

    assert_eq!(
        group["iCalendar"]["components"],
        json!([["vtimezone", [["tzid", {}, "text", "Europe/Berlin"]], []]])
    );

    assert_eq!(again(&group), group);
}

#[test]
fn tells_apart_the_two_properties_that_share_the_updated_member() {
    let dtstamp = group(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:11\r\n\
         DTSTAMP:20240101T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
    );
    let modified = group(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:11\r\n\
         LAST-MODIFIED:20240101T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
    );

    assert_eq!(entry(&dtstamp, "updated"), entry(&modified, "updated"));

    // NOTE: DTSTAMP is what a reader assumes, so only LAST-MODIFIED is
    // recorded.
    assert_eq!(
        entry(&dtstamp, "iCalendar")["convertedProperties"],
        Value::Null
    );
    assert_eq!(
        entry(&modified, "iCalendar")["convertedProperties"]["updated"]["name"],
        json!("last-modified")
    );

    assert_eq!(again(&dtstamp), dtstamp);
    assert_eq!(again(&modified), modified);
}

#[test]
fn carries_a_member_no_property_holds_through_jsprop() {
    let group = json!({
        "@type": "Group",
        "entries": [{
            "@type": "Event",
            "uid": "12",
            "title": "Lunch",
            "useDefaultAlerts": true,
            "example.com:foo": {"bar": 1234},
        }],
    });

    let ical = Ical::from_jscalendar(&group).expect("a readable Group");
    let event = &ical.components[0];

    let pointers: Vec<&str> = event
        .props
        .iter()
        .filter(|prop| prop.name.eq_ignore_ascii_case("JSPROP"))
        .filter_map(|prop| match prop.params.first() {
            Some(ical::param::IcalParam::Unknown { values, .. }) => {
                values.first().map(|value| value.as_ref())
            }
            _ => None,
        })
        .collect();

    // NOTE: A JSON object is a set of members, and serde_json keeps it sorted,
    // so the pointers come out in that order rather than the source order.
    assert_eq!(pointers, ["example.com:foo", "useDefaultAlerts"]);
    assert_eq!(ical.to_jscalendar(), group);
}

#[test]
fn reads_a_lone_event_as_the_calendar_holding_it() {
    let event = json!({
        "@type": "Event",
        "uid": "13",
        "title": "Lunch",
        "start": "2024-01-01T12:00:00",
        "duration": "PT1H",
    });

    let ical = Ical::from_jscalendar(&event).expect("a readable Event");

    assert_eq!(ical.components.len(), 1);
    assert_eq!(&*ical.components[0].name, "VEVENT");
    assert_eq!(ical.to_jscalendar()["entries"][0], event);
}

#[test]
fn refuses_a_root_that_is_no_calendar_object() {
    use ical::jscalendar::IcalJscalendarError;

    assert_eq!(
        Ical::from_jscalendar(&json!("nope")),
        Err(IcalJscalendarError::NotAnObject)
    );
    assert_eq!(
        Ical::from_jscalendar(&json!({"@type": "Alert"})),
        Err(IcalJscalendarError::NotAGroup(String::from("Alert")))
    );
}

#[test]
fn round_trips_the_worked_examples_of_rfc_8984() {
    // NOTE: RFC 8984 6.1, 6.2, 6.4, 6.5 and 6.9, trimmed of the members this
    // conversion has no iCalendar word for, which ride the JSPROP hatch and
    // are covered on their own above.
    let examples = [
        json!({
            "@type": "Event",
            "uid": "a8df6573-0474-496d-8496-033ad45d7fea",
            "updated": "2020-01-02T18:23:04Z",
            "title": "Some event",
            "start": "2020-01-15T13:00:00",
            "timeZone": "America/New_York",
            "duration": "PT1H",
        }),
        json!({
            "@type": "Task",
            "uid": "2a358cee-6489-4f14-a57f-c104db4dc2f2",
            "updated": "2020-01-09T14:32:01Z",
            "title": "Do something",
        }),
        json!({
            "@type": "Event",
            "uid": "2a358cee-6489-4f14-a57f-c104db4dc343",
            "updated": "2020-01-09T14:32:01Z",
            "title": "April Fool's Day",
            "showWithoutTime": true,
            "start": "2020-04-01T00:00:00",
            "duration": "P1D",
            "recurrenceRules": [{"@type": "RecurrenceRule", "frequency": "yearly"}],
        }),
        json!({
            "@type": "Task",
            "uid": "2a358cee-6489-4f14-a57f-c104db4dc357",
            "updated": "2020-01-09T14:32:01Z",
            "title": "Buy groceries",
            "due": "2020-01-19T18:00:00",
            "timeZone": "Europe/Vienna",
            "estimatedDuration": "PT1H",
        }),
        json!({
            "@type": "Event",
            "uid": "2a358cee-6489-4f14-a57f-c104db4dc2f2",
            "updated": "2020-01-02T18:23:04Z",
            "title": "Delivery",
            "start": "2020-01-07T09:00:00",
            "timeZone": "Australia/Melbourne",
            "duration": "PT1H",
            "recurrenceRules": [{"@type": "RecurrenceRule", "frequency": "weekly"}],
            "recurrenceOverrides": {
                "2020-01-08T09:00:00": {"start": "2020-01-08T12:00:00"},
                "2020-01-15T09:00:00": {"excluded": true},
            },
        }),
    ];

    for example in examples {
        let ical = Ical::from_jscalendar(&example).expect("a readable JSCalendar object");
        assert_eq!(
            ical.to_jscalendar()["entries"][0],
            example,
            "\n  the example did not survive the round trip"
        );
    }
}

#[test]
fn converts_the_whole_corpus_to_a_stable_group() {
    // NOTE: The fixture counts the corpus sweep asserts; the ones a strict
    // parse refuses are skipped here, since a calendar that never decodes never
    // converts.
    let corpora = [
        ("rfc", 7),
        ("vcalendar", 1),
        ("libical", 40),
        ("ical4j", 104),
        ("icaljs", 46),
    ];
    let mut converted = 0;

    for (corpus, total) in corpora {
        common::each_fixture(corpus, total, |name, bytes| {
            let Ok(cst) = IcalCst::parse(bytes) else {
                return;
            };

            let ical = cst.decode().into_owned();
            let group = ical.to_jscalendar();

            // NOTE: The first conversion may normalise (a DTEND becomes a span,
            // an override folds into its series); a second one may not, or the
            // escape hatch did not catch what the first pass could not express.
            assert_eq!(
                again(&group),
                group,
                "not stable through JSCalendar: {name}"
            );
            converted += 1;
        });
    }

    // NOTE: The count is asserted so a fixture that stops parsing, and so stops
    // being converted, cannot pass unnoticed.
    assert_eq!(converted, 186);
}
