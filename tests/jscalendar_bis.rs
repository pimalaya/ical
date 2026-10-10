//! The JSCalendar 2.0 conversion (draft-ietf-calext-jscalendarbis), both ways.
//!
//! Written as 2.0, a calendar must come back from its Group line for line, and
//! the Group must hold none of the members 2.0 obsoletes or reserves. Read, a
//! Group or Event a JMAP server wrote in 2.0 must say what it says in
//! iCalendar, and one in RFC 8984 must read as it always did.

#![cfg(all(feature = "jscalendar", feature = "parser"))]

mod common;

use ical::{ical::Ical, jscalendar::IcalJscalendarVersion, tree::cst::IcalCst};
use serde_json::{Value, json};

/// A calendar's decoded model, from its wire bytes.
fn decode(ics: &str) -> Ical<'static> {
    IcalCst::parse(ics)
        .expect("a readable calendar")
        .decode()
        .into_owned()
}

/// The 2.0 Group a calendar converts to.
fn group(ics: &str) -> Value {
    decode(ics).to_jscalendar_as(IcalJscalendarVersion::V2_0)
}

/// The calendar a JSCalendar value reads as, on the wire.
fn read(value: &Value) -> String {
    let ical = Ical::from_jscalendar(value).expect("a readable JSCalendar object");
    IcalCst::from(ical).to_string()
}

/// The content lines of a calendar, unfolded, with their parameters sorted
/// and the conversion's own `JSID` keys left out, sorted: what a round trip
/// must preserve, since neither line nor parameter order survives JSCalendar.
fn lines(ics: &str) -> Vec<String> {
    let mut lines: Vec<String> = ics
        .replace("\r\n ", "")
        .split("\r\n")
        .filter(|line| !line.is_empty() && !line.starts_with("JSID:"))
        .map(|line| {
            let (head, value) = line.split_once(':').expect("a content line");
            let mut params: Vec<&str> = head.split(';').collect();
            let name = params.remove(0);
            params.retain(|param| !param.starts_with("JSID="));
            params.sort_unstable();

            match params.is_empty() {
                true => format!("{name}:{value}"),
                false => format!("{name};{}:{value}", params.join(";")),
            }
        })
        .collect();

    lines.sort_unstable();
    lines
}

/// The member names a value holds at any depth, so a retired one cannot hide
/// in a participant or an override.
fn members(value: &Value, names: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for (name, value) in object {
                // NOTE: The escape hatch is jCal, whose names are iCalendar's.
                if name != "iCalendar" {
                    names.push(name.clone());
                    members(value, names);
                }
            }
        }
        Value::Array(items) => items.iter().for_each(|item| members(item, names)),
        _ => {}
    }
}

/// What a 2.0 object must never set (bis draft A.2.1, A.2.2).
const RETIRED: [&str; 16] = [
    "recurrenceRules",
    "excludedRecurrenceRules",
    "timeZones",
    "replyTo",
    "sendTo",
    "requestStatus",
    "localizations",
    "progressUpdated",
    "language",
    "locationId",
    "invitedBy",
    "participationComment",
    "scheduleAgent",
    "scheduleForceSend",
    "scheduleStatus",
    "cid",
];

/// Assert a 2.0 Group sets nothing 2.0 retired, anywhere.
fn assert_current(group: &Value) {
    let mut names = Vec::new();
    members(group, &mut names);

    for retired in RETIRED {
        assert!(
            !names.iter().any(|name| name == retired),
            "a 2.0 Group sets `{retired}`: {group:#}"
        );
    }

    let roles = group.to_string();
    assert!(!roles.contains("\"attendee\":true"), "{group:#}");
}

const SERIES: &str = "BEGIN:VCALENDAR\r\n\
    VERSION:2.0\r\n\
    PRODID:-//Example//EN\r\n\
    BEGIN:VTIMEZONE\r\n\
    TZID:Europe/Berlin\r\n\
    BEGIN:STANDARD\r\n\
    DTSTART:19701025T030000\r\n\
    RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10\r\n\
    TZOFFSETFROM:+0200\r\n\
    TZOFFSETTO:+0100\r\n\
    END:STANDARD\r\n\
    BEGIN:DAYLIGHT\r\n\
    DTSTART:19700329T020000\r\n\
    RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=3\r\n\
    TZOFFSETFROM:+0100\r\n\
    TZOFFSETTO:+0200\r\n\
    END:DAYLIGHT\r\n\
    END:VTIMEZONE\r\n\
    BEGIN:VEVENT\r\n\
    UID:series@example.com\r\n\
    DTSTAMP:20260101T080000Z\r\n\
    DTSTART;TZID=Europe/Berlin:20260105T100000\r\n\
    DTEND;TZID=Europe/Berlin:20260105T110000\r\n\
    SUMMARY:Weekly sync\r\n\
    LOCATION:Room 1\r\n\
    RRULE:FREQ=WEEKLY;COUNT=10\r\n\
    EXDATE;TZID=Europe/Berlin:20260119T100000\r\n\
    ORGANIZER;CN=Ada:mailto:ada@example.com\r\n\
    ATTENDEE;CN=Ada;PARTSTAT=ACCEPTED;ROLE=CHAIR:mailto:ada@example.com\r\n\
    ATTENDEE;CN=Bob;RSVP=TRUE;PARTSTAT=NEEDS-ACTION;ROLE=REQ-PARTICIPANT:mailto:bob@example.com\r\n\
    ATTENDEE;ROLE=OPT-PARTICIPANT;LANGUAGE=fr;SCHEDULE-AGENT=CLIENT:mailto:cy@example.com\r\n\
    BEGIN:VALARM\r\n\
    TRIGGER:-PT15M\r\n\
    ACTION:DISPLAY\r\n\
    DESCRIPTION:Reminder\r\n\
    END:VALARM\r\n\
    END:VEVENT\r\n\
    BEGIN:VEVENT\r\n\
    UID:series@example.com\r\n\
    DTSTAMP:20260101T080000Z\r\n\
    RECURRENCE-ID;TZID=Europe/Berlin:20260112T100000\r\n\
    DTSTART;TZID=Europe/Berlin:20260112T140000\r\n\
    DTEND;TZID=Europe/Berlin:20260112T150000\r\n\
    SUMMARY:Weekly sync (moved)\r\n\
    LOCATION:Room 1\r\n\
    ORGANIZER;CN=Ada:mailto:ada@example.com\r\n\
    ATTENDEE;CN=Ada;PARTSTAT=ACCEPTED;ROLE=CHAIR:mailto:ada@example.com\r\n\
    ATTENDEE;CN=Bob;RSVP=TRUE;PARTSTAT=DECLINED;ROLE=REQ-PARTICIPANT:mailto:bob@example.com\r\n\
    ATTENDEE;ROLE=OPT-PARTICIPANT;LANGUAGE=fr;SCHEDULE-AGENT=CLIENT:mailto:cy@example.com\r\n\
    BEGIN:VALARM\r\n\
    TRIGGER:-PT15M\r\n\
    ACTION:DISPLAY\r\n\
    DESCRIPTION:Reminder\r\n\
    END:VALARM\r\n\
    END:VEVENT\r\n\
    END:VCALENDAR\r\n";

#[test]
fn writes_a_series_in_the_two_point_zero_names() {
    let group = group(SERIES);
    let event = &group["entries"][0];

    assert_eq!(group["version"], json!("2.0"));
    assert_eq!(
        event.get("version"),
        None,
        "an entry never states a version"
    );
    assert_eq!(group["entries"].as_array().map(Vec::len), Some(1));

    assert_eq!(event["start"], json!("2026-01-05T10:00:00"));
    assert_eq!(event["timeZone"], json!("Europe/Berlin"));
    assert_eq!(event["duration"], json!("PT1H"));
    assert_eq!(
        event["recurrenceRule"],
        json!({"@type": "RecurrenceRule", "frequency": "weekly", "count": 10})
    );
    assert_eq!(
        event["organizerCalendarAddress"],
        json!("mailto:ada@example.com")
    );

    // NOTE: The organizer and its attendee are one Participant (conversion
    // draft 2.3.29), and REQ-PARTICIPANT is the 2.0 `required` role.
    let participants = &event["participants"];
    assert_eq!(participants.as_object().map(|p| p.len()), Some(3));
    assert_eq!(
        participants["1"],
        json!({
            "@type": "Participant",
            "calendarAddress": "mailto:ada@example.com",
            "name": "Ada",
            "participationStatus": "accepted",
            "roles": {"chair": true, "owner": true},
        })
    );
    assert_eq!(
        participants["2"],
        json!({
            "@type": "Participant",
            "calendarAddress": "mailto:bob@example.com",
            "name": "Bob",
            "expectReply": true,
            "participationStatus": "needs-action",
            "roles": {"required": true},
        })
    );

    // NOTE: `language` and `scheduleAgent` are retired, so their parameters
    // stay on the record of the line they came from.
    assert_eq!(
        participants["3"],
        json!({
            "@type": "Participant",
            "calendarAddress": "mailto:cy@example.com",
            "roles": {"optional": true},
        })
    );
    assert_eq!(
        event["iCalendar"]["convertedProperties"]["participants/3"]["parameters"],
        json!({"language": "fr", "schedule-agent": "CLIENT"})
    );

    assert_eq!(
        event["recurrenceOverrides"],
        json!({
            "2026-01-19T10:00:00": {"excluded": true},
            "2026-01-12T10:00:00": {
                "start": "2026-01-12T14:00:00",
                "title": "Weekly sync (moved)",
                "participants/2/participationStatus": "declined",
            },
        })
    );
    assert_eq!(event["alerts"]["1"]["trigger"]["offset"], json!("-PT15M"));
    assert_eq!(event["locations"]["1"]["name"], json!("Room 1"));

    assert_current(&group);
}

#[test]
fn round_trips_a_series_through_two_point_zero() {
    let group = group(SERIES);
    let back = read(&group);

    assert_eq!(lines(&back), lines(SERIES));

    // NOTE: A second pass changes nothing the first did not.
    assert_eq!(
        decode(&back).to_jscalendar_as(IcalJscalendarVersion::V2_0),
        group
    );
}

#[test]
fn reads_an_event_as_stalwart_writes_it() {
    // NOTE: The shape calcard, Stalwart's conversion, gives a scheduled series:
    // UUID keys, the organizer's Participant holding its attendance too, no
    // `@type` where 2.0 makes it optional, and no `version`.
    let event = json!({
        "@type": "Event",
        "uid": "7b0e5a52-3d3a-4a8a-9b0c-1f0e2c0d9c11",
        "updated": "2026-01-01T08:00:00Z",
        "title": "Planning",
        "start": "2026-02-02T09:30:00",
        "timeZone": "Europe/Paris",
        "duration": "PT45M",
        "recurrenceRule": {"frequency": "monthly", "byMonthDay": [2]},
        "recurrenceOverrides": {
            "2026-04-02T09:30:00": {"excluded": true},
            "2026-03-02T09:30:00": {"start": "2026-03-02T11:00:00"},
        },
        "organizerCalendarAddress": "mailto:ada@example.com",
        "participants": {
            "a19f19be-c464-5dcb-9851-9b656fcda246": {
                "calendarAddress": "mailto:ada@example.com",
                "name": "Ada",
                "participationStatus": "accepted",
                "roles": {"owner": true, "chair": true, "required": true},
            },
            "32e5f08c-6141-535c-8462-f8ff78ba0cb3": {
                "@type": "Participant",
                "calendarAddress": "mailto:bob@example.com",
                "expectReply": true,
                "participationStatus": "needs-action",
                "roles": {"required": true},
            },
        },
        "alerts": {
            "k1": {"trigger": {"offset": "-PT10M"}, "action": "display"},
        },
        "locations": {"l1": {"name": "Room 2"}},
        "virtualLocations": {"v1": {"uri": "https://meet.example.com/x", "name": "Call"}},
    });

    let back = read(&event);

    for line in [
        "SUMMARY:Planning",
        "DTSTART;TZID=Europe/Paris:20260202T093000",
        "DURATION:PT45M",
        "RRULE:FREQ=MONTHLY;BYMONTHDAY=2",
        "EXDATE;TZID=Europe/Paris:20260402T093000",
        "RECURRENCE-ID;TZID=Europe/Paris:20260302T093000",
        "DTSTART;TZID=Europe/Paris:20260302T110000",
        "ORGANIZER;CN=Ada:mailto:ada@example.com",
        "ATTENDEE;CN=Ada;PARTSTAT=ACCEPTED;ROLE=CHAIR:mailto:ada@example.com",
        "ATTENDEE;PARTSTAT=NEEDS-ACTION;ROLE=REQ-PARTICIPANT;RSVP=TRUE:mailto:bob@example.com",
        "TRIGGER:-PT10M",
        "LOCATION:Room 2",
        "CONFERENCE;LABEL=Call:https://meet.example.com/x",
    ] {
        assert!(
            lines(&back).contains(&line.to_owned()),
            "missing `{line}` in:\n{back}"
        );
    }

    // NOTE: The organizer reads as one ORGANIZER per component, never as a
    // second ATTENDEE, and nothing 2.0 names rides the JSPROP hatch.
    assert_eq!(back.matches("BEGIN:VEVENT").count(), 2);
    assert_eq!(back.matches("ORGANIZER").count(), 2);
    assert!(!back.contains("JSPROP"), "{back}");
}

#[test]
fn reads_an_organizer_participant_as_no_attendee() {
    let event = json!({
        "@type": "Event",
        "uid": "o1",
        "start": "2026-02-02T09:30:00",
        "organizerCalendarAddress": "mailto:ada@example.com",
        "participants": {
            "x": {"calendarAddress": "mailto:ada@example.com", "name": "Ada", "roles": {"owner": true}},
            "y": {"calendarAddress": "mailto:bob@example.com", "roles": {"owner": true}},
        },
    });

    let back = lines(&read(&event));

    // NOTE: An owner that is not the organizer is an ATTENDEE with the OWNER
    // role (conversion draft Figure 22).
    assert!(back.contains(&"ORGANIZER;CN=Ada:mailto:ada@example.com".to_owned()));
    assert!(back.contains(&"ATTENDEE;ROLE=OWNER:mailto:bob@example.com".to_owned()));
    assert_eq!(
        back.iter()
            .filter(|line| line.starts_with("ATTENDEE"))
            .count(),
        1
    );
}

#[test]
fn keeps_an_attendee_organizing_its_own_event() {
    // NOTE: Three shapes of one person on two lines: lines that agree join,
    // lines that do not stay apart, and a bare ATTENDEE beside its ORGANIZER
    // keeps its line through the record the export leaves.
    for pair in [
        "ORGANIZER:mailto:ada@example.com\r\n\
         ATTENDEE:mailto:ada@example.com\r\n",
        "ORGANIZER;CN=Ada:mailto:ada@example.com\r\n\
         ATTENDEE;PARTSTAT=ACCEPTED:mailto:ada@example.com\r\n",
        "ORGANIZER:mailto:ada@example.com\r\n",
    ] {
        let ics = format!(
            "BEGIN:VCALENDAR\r\n\
             VERSION:2.0\r\n\
             BEGIN:VEVENT\r\n\
             UID:own\r\n\
             DTSTART:20260105T100000Z\r\n\
             {pair}\
             END:VEVENT\r\n\
             END:VCALENDAR\r\n"
        );

        let group = group(&ics);
        assert_current(&group);
        assert_eq!(lines(&read(&group)), lines(&ics), "{group:#}");
    }
}

#[test]
fn writes_a_task_attendee_progress_on_its_participant() {
    let ics = "BEGIN:VCALENDAR\r\n\
               VERSION:2.0\r\n\
               BEGIN:VTODO\r\n\
               UID:task\r\n\
               DTSTART:20260105T100000Z\r\n\
               STATUS:IN-PROCESS\r\n\
               COMPLETED:20260106T100000Z\r\n\
               ATTENDEE;PARTSTAT=COMPLETED:mailto:bob@example.com\r\n\
               REQUEST-STATUS:2.0;Success\r\n\
               END:VTODO\r\n\
               END:VCALENDAR\r\n";

    let group = group(ics);
    let task = &group["entries"][0];

    // NOTE: Conversion draft Table 13: the participant accepted and completed
    // its part, while the task as a whole is still in process.
    assert_eq!(task["progress"], json!("in-process"));
    assert_eq!(
        task["participants"]["1"]["participationStatus"],
        json!("accepted")
    );
    assert_eq!(task["participants"]["1"]["progress"], json!("completed"));

    assert_current(&group);
    assert_eq!(lines(&read(&group)), lines(ics));
}

#[test]
fn keeps_what_two_point_zero_retired_in_the_escape_hatch() {
    let ics = "BEGIN:VCALENDAR\r\n\
               VERSION:2.0\r\n\
               BEGIN:VEVENT\r\n\
               UID:retired\r\n\
               DTSTART:20260105T100000Z\r\n\
               RRULE:FREQ=DAILY;COUNT=3\r\n\
               RRULE:FREQ=WEEKLY;COUNT=2\r\n\
               EXRULE:FREQ=DAILY;INTERVAL=2\r\n\
               IMAGE;VALUE=URI;DISPLAY=BADGE:https://example.com/i.png\r\n\
               BEGIN:VLOCATION\r\n\
               UID:loc\r\n\
               NAME:Hall\r\n\
               DESCRIPTION:Second floor\r\n\
               END:VLOCATION\r\n\
               END:VEVENT\r\n\
               END:VCALENDAR\r\n";

    let group = group(ics);
    let event = &group["entries"][0];

    // NOTE: One rule is 2.0's `recurrenceRule`; the second and the excluded
    // one have no member (bis draft A.2.1).
    assert_eq!(
        event["recurrenceRule"],
        json!({"@type": "RecurrenceRule", "frequency": "daily", "count": 3})
    );
    assert_eq!(
        event["iCalendar"]["properties"],
        json!([
            ["rrule", {}, "recur", {"freq": "weekly", "count": 2}],
            ["exrule", {}, "recur", {"freq": "daily", "interval": 2}],
        ])
    );

    // NOTE: A Link's display is a set in 2.0 (A.2.3), a Location's
    // description reserved (A.2.2.4).
    assert_eq!(event["links"]["1"]["display"], json!({"badge": true}));
    assert_eq!(event["locations"]["loc"].get("description"), None);

    assert_current(&group);
    assert_eq!(lines(&read(&group)), lines(ics));

    // NOTE: RFC 8984 still holds all of them as members.
    let rfc = decode(ics).to_jscalendar();
    assert_eq!(
        rfc["entries"][0]["recurrenceRules"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert!(rfc["entries"][0]["excludedRecurrenceRules"].is_array());
    assert_eq!(rfc["entries"][0]["links"]["1"]["display"], json!("badge"));
}

#[test]
fn moves_the_method_onto_every_entry() {
    let ics = "BEGIN:VCALENDAR\r\n\
               VERSION:2.0\r\n\
               METHOD:REQUEST\r\n\
               BEGIN:VEVENT\r\n\
               UID:m1\r\n\
               DTSTART:20260105T100000Z\r\n\
               END:VEVENT\r\n\
               BEGIN:VEVENT\r\n\
               UID:m2\r\n\
               DTSTART:20260106T100000Z\r\n\
               END:VEVENT\r\n\
               END:VCALENDAR\r\n";

    let group = group(ics);

    // NOTE: A 2.0 Group has no method (bis draft 4.3); every entry carries
    // the calendar's (conversion draft 2.3.27).
    assert_eq!(group.get("method"), None);
    assert_eq!(group["entries"][0]["method"], json!("request"));
    assert_eq!(group["entries"][1]["method"], json!("request"));

    assert_eq!(lines(&read(&group)), lines(ics));

    // NOTE: A lone entry is the calendar it stands in, method included.
    let lone = read(&group["entries"][0]);
    assert!(lone.contains("METHOD:REQUEST"), "{lone}");
    assert_eq!(lone.matches("METHOD").count(), 1);
}

#[test]
fn reads_rfc_8984_as_before() {
    let event = json!({
        "@type": "Event",
        "uid": "r1",
        "start": "2026-02-02T09:30:00",
        "recurrenceRules": [{"@type": "RecurrenceRule", "frequency": "daily"}],
        "replyTo": {"imip": "mailto:ada@example.com"},
        "participants": {
            "1": {
                "@type": "Participant",
                "sendTo": {"imip": "mailto:ada@example.com"},
                "name": "Ada",
                "roles": {"owner": true},
            },
            "2": {
                "@type": "Participant",
                "sendTo": {"imip": "mailto:bob@example.com"},
                "roles": {"attendee": true},
                "language": "fr",
            },
        },
    });

    let back = lines(&read(&event));

    assert!(back.contains(&"RRULE:FREQ=DAILY".to_owned()));
    assert!(back.contains(&"ORGANIZER;CN=Ada:mailto:ada@example.com".to_owned()));
    assert!(back.contains(&"ATTENDEE;LANGUAGE=fr:mailto:bob@example.com".to_owned()));

    // NOTE: The same object reads back to itself, as RFC 8984.
    let ical = Ical::from_jscalendar(&event).expect("an Event");
    assert_eq!(ical.to_jscalendar()["entries"][0], event);
}

#[test]
fn converts_the_whole_corpus_to_a_stable_two_point_zero_group() {
    // NOTE: The same sweep as the RFC 8984 one, at 2.0, which also asserts
    // that no fixture makes a 2.0 Group set a retired member.
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

            let group = cst
                .decode()
                .into_owned()
                .to_jscalendar_as(IcalJscalendarVersion::V2_0);

            let again = Ical::from_jscalendar(&group)
                .expect("a readable JSCalendar object")
                .to_jscalendar_as(IcalJscalendarVersion::V2_0);

            assert_eq!(again, group, "not stable through JSCalendar 2.0: {name}");
            assert_current(&group);
            converted += 1;
        });
    }

    assert_eq!(converted, 186);
}
