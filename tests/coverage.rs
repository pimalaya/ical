//! A sweep over the closed vocabularies and the value codec.
//!
//! Most of this crate is a small amount of logic over four closed enums, and
//! most of the ways those enums can be wrong are invisible to a test that only
//! reads a calendar or two: one `Deref` arm spelling a name differently from
//! the `FromStr` arm that parses it, one value kind whose decoder disagrees
//! with its encoder. Nothing catches that except walking every variant.
//!
//! So each vocabulary is walked whole, and every value kind is driven through
//! a calendar and back. The point is not the coverage number; it is that a
//! typo in an arm nobody exercises has somewhere to fail.

#![cfg(feature = "parser")]

use std::{borrow::Cow, str::FromStr};

use ical::{
    component::{IcalComponentKind, IcalComponentName},
    ical::Ical,
    param::{IcalParam, IcalParamKind},
    prop::{IcalProp, IcalPropKind, IcalPropName},
    tree::cst::IcalCst,
    tree::error::IcalParseError,
    tree::param::{
        IcalParamLens, IcalParamNode, altrep::ALTREP, charset::CHARSET, cn::CN, cutype::CUTYPE,
        delegated_from::DELEGATED_FROM, delegated_to::DELEGATED_TO, derived::DERIVED, dir::DIR,
        display::DISPLAY, email::EMAIL, encoding::ENCODING, fbtype::FBTYPE, feature::FEATURE,
        fmttype::FMTTYPE, gap::GAP, label::LABEL, language::LANGUAGE, linkrel::LINKREL,
        member::MEMBER, order::ORDER, partstat::PARTSTAT, range::RANGE, related::RELATED,
        reltype::RELTYPE, role::ROLE, rsvp::RSVP, schedule_agent::SCHEDULE_AGENT,
        schedule_force_send::SCHEDULE_FORCE_SEND, schedule_status::SCHEDULE_STATUS, schema::SCHEMA,
        sent_by::SENT_BY, tzid::TZID, value::VALUE,
    },
    value::{
        IcalUnknownValue, IcalValue, IcalValueKind,
        binary::IcalBinary,
        boolean::IcalBoolean,
        cal_address::IcalCalAddress,
        datetime::{IcalDate, IcalDateTime, IcalDateTimeList, IcalTime},
        duration::IcalDuration,
        float::IcalFloat,
        geo::IcalGeo,
        integer::IcalInteger,
        period::IcalPeriod,
        recur::IcalRecur,
        request_status::IcalRequestStatus,
        text::{IcalText, IcalTextList},
        uri::IcalUri,
        utc_offset::IcalUtcOffset,
    },
    version::IcalVersion,
};

// --- the closed vocabularies ---

#[test]
fn every_property_name_parses_back_from_its_own_spelling() {
    for kind in IcalPropKind::ALL {
        let wire: &str = &kind;

        assert_eq!(IcalPropKind::from_str(wire).ok(), Some(kind), "{wire}");
        assert_eq!(
            IcalPropKind::from_str(&wire.to_lowercase()).ok(),
            Some(kind),
            "{wire} is meant to be case-insensitive"
        );

        // NOTE: A known name resolves to its kind rather than staying verbatim,
        // which is what lets a caller match on it.
        assert_eq!(IcalPropName::from(wire), IcalPropName::Kind(kind));
    }
}

#[test]
fn every_component_name_parses_back_from_its_own_spelling() {
    for kind in IcalComponentKind::ALL {
        let wire: &str = &kind;

        assert_eq!(IcalComponentKind::from_str(wire).ok(), Some(kind), "{wire}");
        assert_eq!(
            IcalComponentKind::from_str(&wire.to_lowercase()).ok(),
            Some(kind),
            "{wire} is meant to be case-insensitive"
        );
        assert_eq!(IcalComponentName::from(wire), IcalComponentName::Kind(kind));
    }
}

#[test]
fn every_parameter_name_parses_back_from_its_own_spelling() {
    for kind in IcalParamKind::ALL {
        let wire: &str = &kind;

        assert_eq!(IcalParamKind::from_str(wire).ok(), Some(kind), "{wire}");
        assert_eq!(
            IcalParamKind::from_str(&wire.to_lowercase()).ok(),
            Some(kind),
            "{wire} is meant to be case-insensitive"
        );
    }
}

#[test]
fn every_value_kind_parses_back_from_its_own_spelling() {
    for kind in IcalValueKind::ALL {
        let wire: &str = &kind;

        assert_eq!(IcalValueKind::from_str(wire).ok(), Some(kind), "{wire}");
        assert_eq!(
            IcalValueKind::from_str(&wire.to_lowercase()).ok(),
            Some(kind),
            "{wire} is meant to be case-insensitive"
        );
    }
}

#[test]
fn every_vocabulary_spelling_is_distinct() {
    let names: Vec<&str> = IcalPropKind::ALL.iter().map(|kind| &**kind).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();

    // NOTE: Two kinds sharing a wire name would make `FromStr` pick one and
    // silently lose the other, and only a comparison like this notices.
    assert_eq!(sorted.len(), names.len(), "two properties share a name");

    let params: Vec<&str> = IcalParamKind::ALL.iter().map(|kind| &**kind).collect();
    let mut sorted = params.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), params.len(), "two parameters share a name");
}

#[test]
fn every_version_parses_back_from_its_own_spelling() {
    for version in [IcalVersion::V1_0, IcalVersion::V2_0] {
        let wire: &str = &version;
        assert_eq!(IcalVersion::from_str(wire).ok(), Some(version), "{wire}");
    }
}

// --- the payload enums ---

#[test]
fn every_value_variant_reports_its_kind() {
    let text = || std::borrow::Cow::Borrowed("x");

    let values = [
        (
            IcalValue::Binary(IcalBinary::Base64(text())),
            IcalValueKind::Binary,
        ),
        (
            IcalValue::Boolean(IcalBoolean(text())),
            IcalValueKind::Boolean,
        ),
        (
            IcalValue::CalAddress(IcalCalAddress(text())),
            IcalValueKind::CalAddress,
        ),
        (IcalValue::Date(IcalDate(text())), IcalValueKind::Date),
        (
            IcalValue::DateTime(IcalDateTime(text())),
            IcalValueKind::DateTime,
        ),
        (
            IcalValue::DateTimeList(IcalDateTimeList(vec![text()])),
            IcalValueKind::DateTimeList,
        ),
        (
            IcalValue::Duration(IcalDuration(text())),
            IcalValueKind::Duration,
        ),
        (IcalValue::Float(IcalFloat(text())), IcalValueKind::Float),
        (
            IcalValue::Geo(IcalGeo {
                latitude: text(),
                longitude: text(),
            }),
            IcalValueKind::Geo,
        ),
        (
            IcalValue::Integer(IcalInteger(text())),
            IcalValueKind::Integer,
        ),
        (IcalValue::Period(IcalPeriod(text())), IcalValueKind::Period),
        (IcalValue::Recur(IcalRecur(text())), IcalValueKind::Recur),
        (
            IcalValue::RequestStatus(IcalRequestStatus {
                code: text(),
                description: text(),
                extra: text(),
            }),
            IcalValueKind::RequestStatus,
        ),
        (IcalValue::Text(IcalText(text())), IcalValueKind::Text),
        (
            IcalValue::TextList(IcalTextList(vec![text()])),
            IcalValueKind::TextList,
        ),
        (IcalValue::Time(IcalTime(text())), IcalValueKind::Time),
        (IcalValue::Uri(IcalUri(text())), IcalValueKind::Uri),
        (
            IcalValue::UtcOffset(IcalUtcOffset(text())),
            IcalValueKind::UtcOffset,
        ),
    ];

    assert_eq!(
        values.len(),
        IcalValueKind::ALL.len(),
        "a kind has no sample"
    );

    for (value, kind) in values {
        assert_eq!(value.kind(), Some(kind));

        // NOTE: Owning a value must not change what it is.
        assert_eq!(value.clone().into_owned().kind(), Some(kind));
        assert_eq!(value.clone().into_owned(), value);
    }

    // NOTE: The one arm outside the closed set, which is the point of it.
    assert_eq!(IcalValue::Unknown(IcalUnknownValue::default()).kind(), None);
}

#[test]
fn every_parameter_variant_reports_its_kind() {
    let one = || std::borrow::Cow::Borrowed("x");
    let many = || vec![std::borrow::Cow::Borrowed("x")];

    let params = [
        IcalParam::AltRep(one()),
        IcalParam::Cn(one()),
        IcalParam::CuType(one()),
        IcalParam::DelegatedFrom(many()),
        IcalParam::DelegatedTo(many()),
        IcalParam::Dir(one()),
        IcalParam::Encoding(one()),
        IcalParam::FmtType(one()),
        IcalParam::FbType(one()),
        IcalParam::Language(one()),
        IcalParam::Member(many()),
        IcalParam::PartStat(one()),
        IcalParam::Range(one()),
        IcalParam::Related(one()),
        IcalParam::RelType(one()),
        IcalParam::Role(one()),
        IcalParam::Rsvp(one()),
        IcalParam::SentBy(one()),
        IcalParam::TzId(one()),
        IcalParam::Value(one()),
        IcalParam::Display(one()),
        IcalParam::Email(one()),
        IcalParam::Feature(many()),
        IcalParam::Label(one()),
        IcalParam::Order(one()),
        IcalParam::Schema(one()),
        IcalParam::Derived(one()),
        IcalParam::ScheduleAgent(one()),
        IcalParam::ScheduleForceSend(one()),
        IcalParam::ScheduleStatus(one()),
        IcalParam::LinkRel(one()),
        IcalParam::Gap(one()),
        IcalParam::Charset(one()),
    ];

    assert_eq!(
        params.len(),
        IcalParamKind::ALL.len(),
        "a kind has no sample"
    );

    for (param, kind) in params.iter().zip(IcalParamKind::ALL) {
        assert_eq!(param.kind(), Some(kind));
        assert_eq!(param.clone().into_owned(), *param);
    }

    assert_eq!(
        IcalParam::Unknown {
            name: one(),
            values: many()
        }
        .kind(),
        None
    );
}

// --- the codec, one property per value kind ---

/// A calendar exercising every value kind the model has, each on a property
/// that genuinely takes it.
const MAXIMAL: &str = "BEGIN:VCALENDAR\r\n\
     VERSION:2.0\r\n\
     PRODID:-//Example//EN\r\n\
     BEGIN:VEVENT\r\n\
     UID:maximal@example.com\r\n\
     DTSTAMP:20260101T000000Z\r\n\
     DTSTART;VALUE=DATE:20260105\r\n\
     DTEND:20260105T120000Z\r\n\
     DURATION:PT1H\r\n\
     SUMMARY:Everything\r\n\
     CATEGORIES:one,two\r\n\
     PRIORITY:5\r\n\
     GEO:37.386013;-122.082932\r\n\
     ATTACH;ENCODING=BASE64;VALUE=BINARY:Zm9v\r\n\
     URL:https://example.com/\r\n\
     ORGANIZER:mailto:chair@example.com\r\n\
     RRULE:FREQ=DAILY;COUNT=3\r\n\
     RDATE:20260106T090000Z,20260107T090000Z\r\n\
     REQUEST-STATUS:2.0;Success\r\n\
     X-FLOAT;VALUE=FLOAT:1.5\r\n\
     X-BOOL;VALUE=BOOLEAN:TRUE\r\n\
     X-TIME;VALUE=TIME:120000\r\n\
     X-PERIOD;VALUE=PERIOD:20260105T090000Z/PT1H\r\n\
     X-OFFSET;VALUE=UTC-OFFSET:+0100\r\n\
     X-ANYTHING:whatever\r\n\
     END:VEVENT\r\n\
     END:VCALENDAR\r\n";

#[test]
fn the_maximal_calendar_decodes_every_value_kind() {
    let cst = IcalCst::parse(MAXIMAL).expect("a readable calendar");
    let decoded = cst.decode();

    let mut seen: Vec<IcalValueKind> = decoded.components[0]
        .props
        .iter()
        .filter_map(|prop| prop.value.kind())
        .collect();

    seen.sort_unstable_by(|a, b| (**a).cmp(b));
    seen.dedup();

    let mut expected: Vec<IcalValueKind> = IcalValueKind::ALL.to_vec();
    expected.sort_unstable_by(|a, b| (**a).cmp(b));

    assert_eq!(seen, expected, "a value kind is never reached");

    // NOTE: The one property no kind covers, which must still survive.
    assert!(
        decoded.components[0]
            .props
            .iter()
            .any(|prop| matches!(prop.value, IcalValue::Unknown(_)))
    );
}

#[test]
fn the_maximal_calendar_survives_the_model_round_trip() {
    let cst = IcalCst::parse(MAXIMAL).expect("a readable calendar");
    let decoded = cst.decode();

    // NOTE: Encoding the model and decoding it again must land on the same
    // model: every value kind's encoder has to agree with its decoder.
    let encoded = IcalCst::from(decoded.clone());
    let again = encoded.decode();

    assert_eq!(again.version, decoded.version);
    assert_eq!(again.components.len(), decoded.components.len());
    assert_eq!(again.components[0].props, decoded.components[0].props);
}

#[test]
fn the_maximal_calendar_is_byte_faithful() {
    let cst = IcalCst::parse(MAXIMAL).expect("a readable calendar");

    assert_eq!(cst.to_bytes(), MAXIMAL.as_bytes());
}

#[test]
fn an_owned_calendar_outlives_the_bytes_it_was_read_from() {
    let owned: Ical<'static> = {
        let ics = MAXIMAL.to_owned();
        let cst = IcalCst::parse(&ics).expect("a readable calendar");
        cst.decode().into_owned()
    };

    assert_eq!(
        owned.components[0].props[0].name,
        IcalPropName::Kind(IcalPropKind::Uid)
    );
}

// --- errors say what they are ---

#[test]
fn every_parse_error_names_what_it_could_not_read() {
    let cases = [
        IcalPropKind::from_str("NOT-A-PROP")
            .unwrap_err()
            .to_string(),
        IcalComponentKind::from_str("VNOPE")
            .unwrap_err()
            .to_string(),
        IcalParamKind::from_str("NOT-A-PARAM")
            .unwrap_err()
            .to_string(),
        IcalValueKind::from_str("NOT-A-KIND")
            .unwrap_err()
            .to_string(),
        IcalVersion::from_str("9.9").unwrap_err().to_string(),
    ];

    let offenders = ["NOT-A-PROP", "VNOPE", "NOT-A-PARAM", "NOT-A-KIND", "9.9"];

    for (message, offender) in cases.iter().zip(offenders) {
        assert!(
            message.contains(offender),
            "`{message}` does not name `{offender}`"
        );
    }
}

#[test]
fn an_unknown_name_keeps_its_own_spelling() {
    let prop = IcalProp {
        name: IcalPropName::from("X-VENDOR"),
        params: Vec::new(),
        value: IcalValue::Text(IcalText(std::borrow::Cow::Borrowed("v"))),
    };

    assert!(matches!(prop.name, IcalPropName::Unknown(_)));
    assert_eq!(&*prop.name, "X-VENDOR");
    assert_eq!(&*prop.clone().into_owned().name, "X-VENDOR");
}

// --- the parameter lenses ---

/// Assert one scalar lens reads the value a wire parameter carries, and writes
/// back the name it is named after.
///
/// The wire name is spelled out here rather than taken from the lens' own
/// `KIND`, so a marker pointing at the wrong kind (the copy-paste hazard of
/// thirty-odd near-identical modules) has somewhere to fail.
macro_rules! assert_scalar_lens {
    ($lens:ty, $name:literal, $sample:literal) => {{
        let wire = concat!($name, "=", $sample);

        let node = IcalParamNode::parse(wire);
        assert_eq!(<$lens>::decode(&node), $sample, "{wire} does not decode");

        let encoded = <$lens>::encode(&Cow::Borrowed($sample));
        assert_eq!(encoded.to_string(), wire, "{wire} does not encode back");
        assert_eq!(
            <$lens>::decode(&encoded),
            $sample,
            "{wire} does not survive"
        );
    }};
}

/// The list counterpart: the values split on the commas outside quotes, and
/// join back into one parameter.
macro_rules! assert_list_lens {
    ($lens:ty, $name:literal, $sample:literal) => {{
        let wire = concat!($name, "=", $sample);
        let expected: Vec<Cow<'_, str>> = $sample.split(',').map(Cow::Borrowed).collect();

        let node = IcalParamNode::parse(wire);
        assert_eq!(<$lens>::decode(&node), expected, "{wire} does not decode");

        let encoded = <$lens>::encode(&expected);
        assert_eq!(encoded.to_string(), wire, "{wire} does not encode back");
        assert_eq!(
            <$lens>::decode(&encoded),
            expected,
            "{wire} does not survive"
        );
    }};
}

#[test]
fn every_scalar_parameter_lens_round_trips_under_its_own_name() {
    assert_scalar_lens!(ALTREP, "ALTREP", "cid:part1.msg@example.com");
    assert_scalar_lens!(CHARSET, "CHARSET", "UTF-8");
    assert_scalar_lens!(CN, "CN", "Jane");
    assert_scalar_lens!(CUTYPE, "CUTYPE", "GROUP");
    assert_scalar_lens!(DERIVED, "DERIVED", "TRUE");
    assert_scalar_lens!(DIR, "DIR", "ldap://example.com");
    assert_scalar_lens!(DISPLAY, "DISPLAY", "BADGE");
    assert_scalar_lens!(EMAIL, "EMAIL", "jane@example.com");
    assert_scalar_lens!(ENCODING, "ENCODING", "BASE64");
    assert_scalar_lens!(FBTYPE, "FBTYPE", "BUSY-TENTATIVE");
    assert_scalar_lens!(FMTTYPE, "FMTTYPE", "text/plain");
    assert_scalar_lens!(GAP, "GAP", "PT5M");
    assert_scalar_lens!(LABEL, "LABEL", "Room-12");
    assert_scalar_lens!(LANGUAGE, "LANGUAGE", "en-GB");
    assert_scalar_lens!(LINKREL, "LINKREL", "describedby");
    assert_scalar_lens!(ORDER, "ORDER", "1");
    assert_scalar_lens!(PARTSTAT, "PARTSTAT", "NEEDS-ACTION");
    assert_scalar_lens!(RANGE, "RANGE", "THISANDFUTURE");
    assert_scalar_lens!(RELATED, "RELATED", "START");
    assert_scalar_lens!(RELTYPE, "RELTYPE", "PARENT");
    assert_scalar_lens!(ROLE, "ROLE", "REQ-PARTICIPANT");
    assert_scalar_lens!(RSVP, "RSVP", "TRUE");
    assert_scalar_lens!(SCHEDULE_AGENT, "SCHEDULE-AGENT", "SERVER");
    assert_scalar_lens!(SCHEDULE_FORCE_SEND, "SCHEDULE-FORCE-SEND", "REQUEST");
    assert_scalar_lens!(SCHEDULE_STATUS, "SCHEDULE-STATUS", "2.0");
    assert_scalar_lens!(SCHEMA, "SCHEMA", "https://schema.org/Event");
    assert_scalar_lens!(SENT_BY, "SENT-BY", "mailto:sec@example.com");
    assert_scalar_lens!(TZID, "TZID", "Europe/Paris");
    assert_scalar_lens!(VALUE, "VALUE", "DATE-TIME");
}

#[test]
fn every_list_parameter_lens_round_trips_under_its_own_name() {
    assert_list_lens!(DELEGATED_FROM, "DELEGATED-FROM", "mailto:a@example.com");
    assert_list_lens!(
        DELEGATED_TO,
        "DELEGATED-TO",
        "mailto:a@example.com,mailto:b@example.com"
    );
    assert_list_lens!(FEATURE, "FEATURE", "AUDIO,VIDEO");
    assert_list_lens!(MEMBER, "MEMBER", "mailto:team@example.com");
}

/// Each structural failure is reported as its own variant, naming the text it
/// choked on.
///
/// The classification is what a caller branches on, and the message is what a
/// user reads; a parse that returned the right shape of error with the wrong
/// variant, or a message that named nothing, would otherwise go unnoticed.
#[test]
fn every_structural_failure_is_classified_and_quoted() {
    // NOTE: A last line left without its CRLF is accepted, so what is left for
    // MissingCrlf is input that holds no content line at all.
    let missing_crlf = IcalCst::parse("\r\n\r\n").unwrap_err();
    assert!(matches!(missing_crlf, IcalParseError::MissingCrlf(_)));
    assert!(missing_crlf.to_string().contains("line separator"));

    let missing_colon =
        IcalCst::parse("BEGIN:VCALENDAR\r\nSUMMARY\r\nEND:VCALENDAR\r\n").unwrap_err();
    assert!(matches!(
        missing_colon,
        IcalParseError::MissingPropertyColon(_)
    ));
    assert!(missing_colon.to_string().contains("SUMMARY"));

    let non_utf8 =
        IcalCst::parse(b"BEGIN:VCALENDAR\r\nSUMM\xffARY:x\r\nEND:VCALENDAR\r\n").unwrap_err();
    assert!(matches!(non_utf8, IcalParseError::NonUtf8Header(_)));

    // NOTE: A single envelope-less record is read as a bare calendar rather
    // than refused, so ExpectedBegin is what a second one, following a proper
    // calendar, is met with.
    let expected_begin = IcalCst::parse_many("BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\nSUMMARY:x\r\n")
        .nth(1)
        .expect("a second entry")
        .unwrap_err();
    assert!(matches!(expected_begin, IcalParseError::ExpectedBegin(_)));
    assert!(expected_begin.to_string().contains("SUMMARY"));

    let missing_end = IcalCst::parse("BEGIN:VCALENDAR\r\nSUMMARY:x\r\n").unwrap_err();
    assert!(matches!(missing_end, IcalParseError::MissingEnd(_)));
    assert!(missing_end.to_string().contains("VCALENDAR"));
}
