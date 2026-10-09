//! The two validators, and the proof they mint.
//!
//! Validation is the "strict out" half of Postel's law here: parsing accepts
//! anything, and this is the only place that says no. Both validators report
//! *every* problem rather than the first, because a caller fixing a calendar
//! wants the list, and both hand back an [`IcalValid`] on success, which is a
//! marker nothing else in the crate can construct.
//!
//! Every variant the two error enums have is reached at least once. A variant
//! no input can produce is a rule that does not exist.

#![cfg(feature = "parser")]

use ical::{
    component::IcalComponentKind,
    ical::Ical,
    prop::IcalPropKind,
    recur::{
        IcalRecurFreq, IcalRecurRule, validate::IcalRecurPart, validate::IcalRecurRuleProblem,
    },
    tree::cst::IcalCst,
    validator::{IcalValid, IcalValidateError},
    value::IcalValueKind,
    version::IcalVersion,
};

/// The decoded calendar behind some wire bytes.
fn decode(ics: &str) -> Ical<'static> {
    IcalCst::parse(ics)
        .expect("a readable calendar")
        .decode()
        .into_owned()
}

/// What validating a calendar reports.
fn problems(ics: &str) -> Vec<IcalValidateError> {
    decode(ics).validate().err().unwrap_or_default()
}

/// A conformant calendar holding one event, with the given extra lines.
fn calendar(extra: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//Example//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:validate@example.com\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         {extra}\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n"
    )
}

// --- the proof marker ---

#[test]
fn a_conformant_calendar_earns_a_proof_that_derefs_to_itself() {
    let ics = calendar("");
    let decoded = decode(&ics);
    let expected = decoded.clone();

    let valid: IcalValid<Ical<'static>> = decoded.validate().expect("a conformant calendar");

    // NOTE: The proof is a wrapper, not a copy: what went in is what comes out,
    // reachable through `Deref` and recoverable with `into_inner`.
    assert_eq!(valid.version, IcalVersion::V2_0);
    assert_eq!(valid.components.len(), 1);
    assert_eq!(valid.into_inner(), expected);
}

#[test]
fn a_validated_rule_earns_the_same_proof() {
    let rule = IcalRecurRule::parse("FREQ=WEEKLY;BYDAY=MO,WE;COUNT=10").expect("a readable rule");
    let valid = rule.clone().validate().expect("a conformant rule");

    assert_eq!(valid.freq, IcalRecurFreq::Weekly);
    assert_eq!(valid.into_inner(), rule);
}

#[test]
fn a_proof_can_be_handed_back_to_the_syntax_tree() {
    let ics = calendar("");
    let valid = decode(&ics).validate().expect("a conformant calendar");

    // NOTE: The conversion exists so a caller can prove a calendar before
    // writing it, which is the only reason to mint a proof at all.
    let cst = IcalCst::from(valid);

    assert!(
        String::from_utf8(cst.to_bytes())
            .unwrap()
            .contains("UID:validate@example.com")
    );
}

// --- every way a calendar fails ---

#[test]
fn reports_a_property_the_version_does_not_define() {
    let ics = "BEGIN:VCALENDAR\r\n\
         VERSION:1.0\r\n\
         PRODID:-//Example//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:old@example.com\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         COLOR:turquoise\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    // NOTE: COLOR is RFC 7986, so it has no business in a vCalendar 1.0 file.
    assert!(problems(ics).iter().any(|problem| matches!(
        problem,
        IcalValidateError::PropVersion { prop, version }
            if prop == "COLOR" && *version == IcalVersion::V1_0
    )));
}

#[test]
fn reports_a_required_property_that_is_absent() {
    let ics = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//Example//EN\r\n\
         BEGIN:VEVENT\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    assert!(problems(ics).iter().any(|problem| matches!(
        problem,
        IcalValidateError::MissingProp { component, prop }
            if component == "VEVENT" && *prop == IcalPropKind::Uid
    )));
}

#[test]
fn reports_a_calendar_with_no_prodid() {
    let ics = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n";

    assert!(problems(ics).iter().any(|problem| matches!(
        problem,
        IcalValidateError::MissingProp { component, prop }
            if component == "VCALENDAR" && *prop == IcalPropKind::ProdId
    )));
}

#[test]
fn reports_a_value_of_a_kind_the_property_does_not_take() {
    let ics = calendar("SUMMARY;VALUE=INTEGER:42\r\n");

    assert!(problems(&ics).iter().any(|problem| matches!(
        problem,
        IcalValidateError::ValueKind { prop, kind }
            if *prop == IcalPropKind::Summary && *kind == IcalValueKind::Integer
    )));
}

#[test]
fn reports_a_parameter_the_property_does_not_take() {
    let ics = calendar("SUMMARY;PARTSTAT=ACCEPTED:Kickoff\r\n");

    assert!(problems(&ics).iter().any(|problem| matches!(
        problem,
        IcalValidateError::ParamNotAllowed { prop, param }
            if *prop == IcalPropKind::Summary && *param == ical::param::IcalParamKind::PartStat
    )));
}

#[test]
fn lets_an_extension_parameter_pass() {
    let ics = calendar("SUMMARY;X-VENDOR-FLAG=1:Kickoff\r\n");

    // NOTE: A parameter outside the vocabulary is somebody else's extension,
    // and refusing it would make the validator the arbiter of a namespace it
    // does not own.
    assert!(problems(&ics).is_empty(), "{:?}", problems(&ics));
}

#[test]
fn reports_a_property_that_appears_too_often() {
    let ics = calendar("SUMMARY:One\r\nSUMMARY:Two\r\n");

    assert!(problems(&ics).iter().any(|problem| matches!(
        problem,
        IcalValidateError::TooMany { component, prop, count }
            if component == "VEVENT" && *prop == IcalPropKind::Summary && *count == 2
    )));
}

#[test]
fn lets_a_repeatable_property_repeat() {
    let ics = calendar("ATTENDEE:mailto:a@example.com\r\nATTENDEE:mailto:b@example.com\r\n");

    assert!(problems(&ics).is_empty(), "{:?}", problems(&ics));
}

#[test]
fn accepts_every_value_and_parameter_the_rfcs_give_a_property() {
    // NOTE: One line per form, each refused once by a contract narrower than
    // its RFC: whole days, zoned times, an organizer with a name and a proxy,
    // an inline attachment, and the RFC 7986 and 9073 parameters.
    let lines = [
        "DTSTART;VALUE=DATE:20260101",
        "DTSTART;TZID=Europe/Paris:20260101T090000",
        "DTEND;VALUE=DATE:20260102",
        "DTEND;TZID=Europe/Paris:20260101T100000",
        "RECURRENCE-ID;VALUE=DATE:20260101",
        "RECURRENCE-ID;TZID=Europe/Paris;RANGE=THISANDFUTURE:20260101T090000",
        "EXDATE;TZID=Europe/Paris:20260108T090000,20260115T090000",
        "RDATE;TZID=Europe/Paris:20260120T090000",
        "RDATE;VALUE=DATE:20260121",
        "ORGANIZER;CN=Ada;DIR=\"ldap://example.com/cn=Ada\";SENT-BY=\"mailto:bob@example.com\"\
         ;LANGUAGE=en;EMAIL=ada@example.com;SCHEDULE-AGENT=CLIENT;SCHEDULE-FORCE-SEND=REQUEST\
         ;SCHEDULE-STATUS=2.0:mailto:ada@example.com",
        "ATTACH;FMTTYPE=text/plain;ENCODING=BASE64;VALUE=BINARY:SGVsbG8=",
        "IMAGE;VALUE=URI;FMTTYPE=image/png;DISPLAY=BADGE;ALTREP=\"https://example.com/a\"\
         :https://example.com/a.png",
        "IMAGE;ENCODING=BASE64;VALUE=BINARY:iVBORw0KGgo=",
        "CONFERENCE;VALUE=URI;FEATURE=AUDIO,VIDEO;LABEL=Room;LANGUAGE=en:https://example.com/r",
        "RELATED-TO;VALUE=URI;RELTYPE=PARENT:https://example.com/parent",
        "LINK;VALUE=URI;LINKREL=describedby;LABEL=Notes;LANGUAGE=en:https://example.com/n",
        "STRUCTURED-DATA;VALUE=URI;FMTTYPE=application/ld+json;SCHEMA=\"https://schema.org/Event\"\
         :https://example.com/e.json",
        "STRUCTURED-DATA;ENCODING=BASE64;VALUE=BINARY;FMTTYPE=application/json:e30=",
        "STYLED-DESCRIPTION;VALUE=URI;FMTTYPE=text/html;DERIVED=TRUE:https://example.com/d.html",
    ];

    for line in lines {
        let ics = calendar(&format!("{line}\r\n"));
        assert!(problems(&ics).is_empty(), "{line}: {:?}", problems(&ics));
    }
}

#[test]
fn accepts_the_alarm_journal_and_free_busy_forms_the_rfc_gives() {
    let ics = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//Example//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:alarm@example.com\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         BEGIN:VALARM\r\n\
         ACTION:DISPLAY\r\n\
         TRIGGER;RELATED=END:-PT15M\r\n\
         END:VALARM\r\n\
         BEGIN:VALARM\r\n\
         ACTION:DISPLAY\r\n\
         TRIGGER;VALUE=DATE-TIME:20260101T080000Z\r\n\
         END:VALARM\r\n\
         END:VEVENT\r\n\
         BEGIN:VJOURNAL\r\n\
         UID:journal@example.com\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         DESCRIPTION:Morning\r\n\
         DESCRIPTION:Evening\r\n\
         END:VJOURNAL\r\n\
         BEGIN:VFREEBUSY\r\n\
         UID:busy@example.com\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         FREEBUSY;FBTYPE=BUSY-TENTATIVE:20260101T090000Z/PT1H\r\n\
         END:VFREEBUSY\r\n\
         END:VCALENDAR\r\n";

    // NOTE: An absolute trigger and its RELATED, FBTYPE on a busy period, and
    // the several descriptions RFC 5545 3.6.3 lets a journal carry.
    assert!(problems(ics).is_empty(), "{:?}", problems(ics));
}

#[test]
fn reports_a_duration_outside_the_grammar_it_still_reads() {
    let ics = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//Example//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:duration@example.com\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         DURATION:P1H\r\n\
         RELATED-TO;GAP=PT1H20S:parent@example.com\r\n\
         BEGIN:VALARM\r\n\
         ACTION:DISPLAY\r\n\
         TRIGGER:-pt15m\r\n\
         END:VALARM\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    let found: Vec<_> = problems(ics)
        .into_iter()
        .filter_map(|problem| match problem {
            IcalValidateError::Duration { prop, duration } => Some((prop, duration)),
            _ => None,
        })
        .collect();

    // NOTE: An hour with no `T`, a second straight after an hour in a GAP
    // parameter, and lower case: each reads as a length, none conforms.
    assert_eq!(
        found,
        [
            (IcalPropKind::Duration, "P1H".to_string()),
            (IcalPropKind::RelatedTo, "PT1H20S".to_string()),
            (IcalPropKind::Trigger, "-pt15m".to_string()),
        ]
    );
}

#[test]
fn passes_a_duration_the_grammar_admits() {
    let ics = calendar("DURATION:P1DT2H\r\nRELATED-TO;GAP=-PT15M:parent@example.com\r\n");

    assert!(problems(&ics).is_empty(), "{:?}", problems(&ics));
}

#[test]
fn still_reports_a_parameter_a_date_property_does_not_take() {
    let ics = calendar("DTSTART;PARTSTAT=ACCEPTED:20260101T090000Z\r\n");

    assert!(problems(&ics).iter().any(|problem| matches!(
        problem,
        IcalValidateError::ParamNotAllowed { prop, param }
            if *prop == IcalPropKind::DtStart && *param == ical::param::IcalParamKind::PartStat
    )));
}

#[test]
fn reports_a_component_nested_where_it_may_not_be() {
    let ics = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID:-//Example//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:nested@example.com\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         BEGIN:VTIMEZONE\r\n\
         TZID:Europe/Berlin\r\n\
         END:VTIMEZONE\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    assert!(problems(ics).iter().any(|problem| matches!(
        problem,
        IcalValidateError::Nesting { parent, child }
            if parent == "VEVENT" && *child == IcalComponentKind::VTimezone
    )));
}

#[test]
fn reports_a_recurrence_rule_the_rule_validator_refuses() {
    let ics = calendar("DTSTART:20260105T090000Z\r\nRRULE:FREQ=DAILY;BYWEEKNO=3\r\n");

    assert!(problems(&ics).iter().any(|problem| matches!(
        problem,
        IcalValidateError::Rule { prop, problem: IcalRecurRuleProblem::PartFreq { part, .. } }
            if *prop == IcalPropKind::RRule && *part == IcalRecurPart::ByWeekNo
    )));
}

#[test]
fn reports_every_problem_rather_than_the_first() {
    let ics = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         SUMMARY:One\r\n\
         SUMMARY:Two\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    // NOTE: A missing PRODID, a missing UID, a missing DTSTAMP and a repeated
    // SUMMARY, all at once, because a caller fixing a calendar wants the list.
    assert!(problems(ics).len() >= 4, "{:?}", problems(ics));
}

#[test]
fn every_calendar_problem_says_what_is_wrong() {
    let broken = "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         BEGIN:VEVENT\r\n\
         SUMMARY;VALUE=INTEGER;PARTSTAT=ACCEPTED:1\r\n\
         SUMMARY:Two\r\n\
         RRULE:FREQ=DAILY;BYWEEKNO=3\r\n\
         DURATION:P1H\r\n\
         BEGIN:VTIMEZONE\r\n\
         TZID:Europe/Berlin\r\n\
         END:VTIMEZONE\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n";

    for problem in problems(broken) {
        assert!(
            !problem.to_string().is_empty(),
            "{problem:?} has nothing to say"
        );
    }
}

// --- every way a rule fails ---

/// What validating a rule reports.
fn rule_problems(rule: &str) -> Vec<IcalRecurRuleProblem> {
    IcalRecurRule::parse(rule)
        .expect("a readable rule")
        .problems()
}

#[test]
fn reports_a_part_the_frequency_forbids() {
    let cases = [
        (
            "FREQ=DAILY;BYWEEKNO=3",
            IcalRecurPart::ByWeekNo,
            IcalRecurFreq::Daily,
        ),
        (
            "FREQ=WEEKLY;BYMONTHDAY=1",
            IcalRecurPart::ByMonthDay,
            IcalRecurFreq::Weekly,
        ),
        (
            "FREQ=MONTHLY;BYYEARDAY=200",
            IcalRecurPart::ByYearDay,
            IcalRecurFreq::Monthly,
        ),
    ];

    for (rule, part, freq) in cases {
        assert!(
            rule_problems(rule).contains(&IcalRecurRuleProblem::PartFreq { part, freq }),
            "{rule}"
        );
    }
}

#[test]
fn reports_a_byday_ordinal_where_it_means_nothing() {
    // NOTE: "the second Monday" only means something inside a month or a year.
    assert!(
        rule_problems("FREQ=WEEKLY;BYDAY=2MO").contains(&IcalRecurRuleProblem::OrdinalFreq {
            freq: IcalRecurFreq::Weekly
        })
    );
}

#[test]
fn reports_a_byday_ordinal_that_would_mean_two_things_at_once() {
    assert!(
        rule_problems("FREQ=YEARLY;BYWEEKNO=3;BYDAY=2MO")
            .contains(&IcalRecurRuleProblem::OrdinalWithWeekNo)
    );
}

#[test]
fn reports_bysetpos_with_nothing_to_pick_from() {
    assert!(rule_problems("FREQ=MONTHLY;BYSETPOS=-1").contains(&IcalRecurRuleProblem::SetPosAlone));
}

#[test]
fn reports_a_rule_bounded_twice() {
    assert!(
        rule_problems("FREQ=DAILY;COUNT=10;UNTIL=20260101T000000Z")
            .contains(&IcalRecurRuleProblem::UntilWithCount)
    );
}

#[test]
fn accepts_the_rules_the_rfc_writes_out() {
    let rules = [
        "FREQ=DAILY;COUNT=10",
        "FREQ=YEARLY;BYMONTH=1;BYDAY=SU,MO,TU,WE,TH,FR,SA",
        "FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1",
        "FREQ=YEARLY;BYWEEKNO=20;BYDAY=MO",
        "FREQ=SECONDLY;BYSECOND=0,30",
    ];

    for rule in rules {
        assert!(
            rule_problems(rule).is_empty(),
            "{rule}: {:?}",
            rule_problems(rule)
        );
    }
}

#[test]
fn every_rule_problem_says_what_is_wrong() {
    let broken =
        "FREQ=WEEKLY;BYWEEKNO=3;BYMONTHDAY=1;BYYEARDAY=4;BYDAY=2MO;COUNT=1;UNTIL=20260101T000000Z";
    let problems = rule_problems(broken);

    assert!(problems.len() >= 5, "{problems:?}");

    for problem in problems {
        assert!(
            !problem.to_string().is_empty(),
            "{problem:?} has nothing to say"
        );
    }
}

#[test]
fn names_every_rule_part_it_can_point_at() {
    let parts = [
        IcalRecurPart::BySecond,
        IcalRecurPart::ByMinute,
        IcalRecurPart::ByHour,
        IcalRecurPart::ByDay,
        IcalRecurPart::ByMonthDay,
        IcalRecurPart::ByYearDay,
        IcalRecurPart::ByWeekNo,
        IcalRecurPart::ByMonth,
        IcalRecurPart::BySetPos,
    ];

    for part in parts {
        let name = part.to_string();

        // NOTE: The name a problem prints has to be the name the rule spells,
        // or a caller cannot find the part it is being told about.
        assert!(name.starts_with("BY"), "{name}");
        assert_eq!(name, name.to_uppercase());
    }
}

// --- the version axis, over the whole vocabulary ---

/// Whether validating this property in this version reports it as out of place.
fn out_of_place(kind: IcalPropKind, version: IcalVersion) -> bool {
    let name: &str = &kind;
    let ics = format!(
        "BEGIN:VCALENDAR\r\n\
         VERSION:{}\r\n\
         PRODID:-//Example//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:axis@example.com\r\n\
         DTSTAMP:20260101T000000Z\r\n\
         {name}:x\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
        &*version,
    );

    problems(&ics).iter().any(|problem| {
        matches!(
            problem,
            IcalValidateError::PropVersion { prop, .. } if prop == name
        )
    })
}

#[test]
fn every_property_belongs_to_at_least_one_version() {
    for kind in IcalPropKind::ALL {
        let versions = [IcalVersion::V1_0, IcalVersion::V2_0];
        let home = versions.iter().filter(|v| !out_of_place(kind, **v)).count();

        // NOTE: A property no version defines is a property that can never be
        // written, which is not a thing the vocabulary should hold.
        assert!(home > 0, "{} belongs to no version", &*kind);
    }
}

#[test]
fn the_legacy_alarm_properties_stayed_in_vcalendar_1_0() {
    // NOTE: iCalendar 2.0 replaced all four with the VALARM component, and
    // RNUM and TZ went with them.
    let legacy = [
        IcalPropKind::AAlarm,
        IcalPropKind::DAlarm,
        IcalPropKind::MAlarm,
        IcalPropKind::PAlarm,
        IcalPropKind::RNum,
        IcalPropKind::Tz,
    ];

    for kind in legacy {
        assert!(!out_of_place(kind, IcalVersion::V1_0), "{}", &*kind);
        assert!(out_of_place(kind, IcalVersion::V2_0), "{}", &*kind);
    }
}

#[test]
fn the_extension_properties_did_not_exist_in_vcalendar_1_0() {
    // NOTE: One per extension RFC the crate covers, so a whole spec silently
    // leaking into 1.0 has somewhere to fail.
    let extensions = [
        IcalPropKind::Color,          // RFC 7986
        IcalPropKind::StructuredData, // RFC 9073
        IcalPropKind::Acknowledged,   // RFC 9074
        IcalPropKind::Concept,        // RFC 9253
        IcalPropKind::BusyType,       // RFC 7953
        IcalPropKind::DtStamp,        // RFC 5545 itself
    ];

    for kind in extensions {
        assert!(out_of_place(kind, IcalVersion::V1_0), "{}", &*kind);
        assert!(!out_of_place(kind, IcalVersion::V2_0), "{}", &*kind);
    }
}

#[test]
fn the_properties_both_versions_share_are_at_home_in_both() {
    let shared = [
        IcalPropKind::Summary,
        IcalPropKind::Description,
        IcalPropKind::DtStart,
        IcalPropKind::Uid,
        IcalPropKind::RRule,
        IcalPropKind::Attendee,
    ];

    for kind in shared {
        assert!(!out_of_place(kind, IcalVersion::V1_0), "{}", &*kind);
        assert!(!out_of_place(kind, IcalVersion::V2_0), "{}", &*kind);
    }
}
