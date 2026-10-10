//! # JSCalendar
//!
//! The JSCalendar conversion, RFC 8984 and 2.0: the decoded calendar as a
//! JSCalendar `Group`, and back.
//!
//! [`Ical::to_jscalendar`] writes the decoded model as the JSON object a JMAP
//! calendar server exchanges; [`Ical::from_jscalendar`] reads one back,
//! borrowing the JSON tree's strings where it can.
//!
//! There is no JSCalendar model in this crate: a Group is a plain
//! [`serde_json::Value`], and iCalendar stays the one decoded model, exactly
//! as [`jcal`](crate::jcal) leaves it.
//!
//! ## A re-modelling, not a re-encoding
//!
//! jCal spells the same model in JSON. JSCalendar is a different model: a
//! `VCALENDAR` is a Group of Events and Tasks (RFC 8984 2.1, 2.2, 5.3), a
//! `DTEND` is a duration, an `ATTENDEE` line is a Participant object and a
//! `VALARM` is an Alert.
//!
//! An overriding `VEVENT` is not a component at all, but a patch inside the
//! series it overrides.
//!
//! The conversion rules are those of [the conversion draft].
//!
//! ## Two versions
//!
//! [`Ical::to_jscalendar`] writes RFC 8984, which the draft calls version
//! 1.0 and other producers write: `recurrenceRules`, a Participant's `sendTo`
//! and the object's `replyTo`.
//!
//! [`Ical::to_jscalendar_as`] can write [JSCalendar 2.0] instead, the model
//! draft-ietf-jmap-calendars builds on: one `recurrenceRule`, `calendarAddress`
//! and `organizerCalendarAddress`, the organizer and its attendee as one
//! Participant, and none of the members 2.0 obsoletes or reserves, which stay
//! in the escape hatch instead. See [`IcalJscalendarVersion`].
//!
//! [`Ical::from_jscalendar`] reads both. An object is read as 2.0 when it or
//! its Group states `version` 2.0, or when it carries a member only 2.0 has,
//! and as RFC 8984 otherwise, so RFC 8984 input reads as it always did.
//!
//! ## Nothing is dropped
//!
//! Both directions are lossless through an escape hatch, and only a
//! non-object root can fail the import.
//!
//! Exporting, a property or component with no JSCalendar counterpart is kept
//! whole in the object's `iCalendar` member, in jCal syntax, and a parameter
//! left over after a property converts is kept in that member's
//! `convertedProperties` record (draft 5.1.1).
//!
//! The same record names the property a member came from wherever more than
//! one could have, so `updated` knows whether it was a `DTSTAMP` or a
//! `LAST-MODIFIED`.
//!
//! Importing, the mirror hatch applies: a member with no iCalendar
//! counterpart becomes a `JSPROP` property holding its JSON, located by a
//! `JSPTR` parameter (draft 4.1.2, 4.2.2).
//!
//! A collection key that was not simply the element's position is carried on
//! a `JSID` parameter so it survives the next conversion.
//!
//! ## What normalises
//!
//! Three things do not survive a round trip unchanged, and none of them is
//! recoverable from the JSON alone.
//!
//! An `RRULE`'s `UNTIL` is stated in UTC whenever `DTSTART` is, but RFC 8984
//! states it in the object's own time zone. Shifting between the two needs
//! the time-zone database, which this crate does not carry, so the wall-clock
//! digits are carried across unshifted.
//!
//! That is exact for a floating or UTC object, and off by that zone's offset
//! for any other. The whole of [`tz`](crate::tz) is available to a caller
//! that wants to shift it from the calendar's own `VTIMEZONE`.
//!
//! A `DTEND` becomes a duration, so an event that ended in another time zone
//! than it started in comes back with the start's zone on both ends.
//!
//! 2.0 can name that end zone in `endTimeZone`, but a span between two zones
//! needs the time-zone database too, so it is not written. Read, it rides a
//! `JSPROP` and the `DTEND` stays in the start's zone.
//!
//! Ordering inside a component is lost, since a JSCalendar object is a set of
//! members rather than a list of lines. Byte fidelity is the syntax tree's
//! job; JSCalendar is a projection of the decoded model, one further removed
//! than jCal is.
//!
//! [the conversion draft]: https://datatracker.ietf.org/doc/draft-ietf-calext-jscalendar-icalendar/
//! [JSCalendar 2.0]: https://datatracker.ietf.org/doc/draft-ietf-calext-jscalendarbis/

mod export;
mod hatch;
mod import;
mod patch;

use core::{error, fmt, ops};

use alloc::string::{String, ToString};

use serde_json::Value;

use crate::ical::Ical;

/// What a JSCalendar value cannot be read as.
///
/// Only the shape of the document is refused; everything inside it is read
/// liberally, so this is a short list on purpose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalJscalendarError {
    /// The document is not a JSON object.
    NotAnObject,
    /// The document is a JSCalendar object of a type this crate has no
    /// calendar for: neither a `Group`, an `Event` nor a `Task`.
    NotAGroup(String),
}

impl fmt::Display for IcalJscalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAnObject => f.write_str("JSCalendar value is not an object"),
            Self::NotAGroup(kind) => write!(
                f,
                "JSCalendar object is a `{kind}`, not a `Group`, an `Event` or a `Task`"
            ),
        }
    }
}

impl error::Error for IcalJscalendarError {}

/// The JSCalendar version a conversion writes (bis draft 1.9).
///
/// Reading needs none: [`Ical::from_jscalendar`] reads both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IcalJscalendarVersion {
    /// JSCalendar 1.0, RFC 8984.
    #[default]
    V1_0,
    /// JSCalendar 2.0, draft-ietf-calext-jscalendarbis-22, which
    /// draft-ietf-jmap-calendars builds on.
    ///
    /// The Group states its `version`. One `RRULE` is the `recurrenceRule`;
    /// a further `RRULE` and an `EXRULE` stay in the escape hatch. `ORGANIZER`
    /// is the `organizerCalendarAddress` and an owner Participant, the same
    /// one as the `ATTENDEE` of that address when the two agree on its name
    /// and email. A Participant's address is its `calendarAddress`, its roles
    /// have no default and `REQ-PARTICIPANT` is `required`, and a task
    /// attendee's progress is its own. `METHOD` is every entry's `method`.
    ///
    /// What 2.0 obsoletes or reserves is never written and stays in the
    /// escape hatch: a participant's `LANGUAGE` and `SCHEDULE-*` parameters,
    /// `REQUEST-STATUS`, `COMPLETED` and a `VLOCATION`'s `DESCRIPTION`.
    V2_0,
}

impl ops::Deref for IcalJscalendarVersion {
    type Target = str;

    /// The `version` value this version is spelled as.
    fn deref(&self) -> &Self::Target {
        match self {
            Self::V1_0 => "1.0",
            Self::V2_0 => "2.0",
        }
    }
}

impl Ical<'_> {
    /// The calendar as an RFC 8984 JSCalendar `Group` value.
    ///
    /// Infallible: what the mapping cannot express is preserved in the
    /// `iCalendar` escape hatch rather than dropped.
    pub fn to_jscalendar(&self) -> Value {
        export::group(self, IcalJscalendarVersion::V1_0)
    }

    /// The calendar as a JSCalendar `Group` value of the given version.
    ///
    /// Infallible, as [`to_jscalendar`](Self::to_jscalendar) is, which this
    /// is at [`V1_0`](IcalJscalendarVersion::V1_0).
    pub fn to_jscalendar_as(&self, version: IcalJscalendarVersion) -> Value {
        export::group(self, version)
    }
}

impl<'a> Ical<'a> {
    /// Read a calendar back from a JSCalendar value, RFC 8984 or 2.0.
    ///
    /// A `Group` is a whole calendar, and a lone `Event` or `Task` is the
    /// calendar holding it, since that is what a JMAP calendar server hands
    /// out one object at a time. Only a root that is neither errors; a member
    /// with no iCalendar counterpart is preserved as a `JSPROP` property.
    ///
    /// An object is 2.0 when it or its Group states a `version` other than
    /// 1.0, or when it carries `recurrenceRule`, `organizerCalendarAddress`,
    /// `endTimeZone`, `mainLocationId` or a Participant's `calendarAddress`.
    /// Either way the RFC 8984 names are read too.
    pub fn from_jscalendar(jscalendar: &'a Value) -> Result<Self, IcalJscalendarError> {
        let object = jscalendar
            .as_object()
            .ok_or(IcalJscalendarError::NotAnObject)?;

        match object.get("@type").and_then(Value::as_str) {
            None | Some("Group") => Ok(import::ical(object)),
            Some("Event" | "Task") => Ok(import::of_entry(jscalendar)),
            Some(kind) => Err(IcalJscalendarError::NotAGroup(kind.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, vec};

    use crate::{
        component::{IcalComponent, IcalComponentKind},
        ical::Ical,
        jscalendar::IcalJscalendarError,
        prop::{IcalProp, IcalPropKind},
        value::{IcalValue, datetime::IcalDateTime, text::IcalText},
        version::IcalVersion,
    };

    /// A hand-built calendar, so the conversion is exercised with no parser.
    fn calendar() -> Ical<'static> {
        Ical {
            version: IcalVersion::V2_0,
            props: vec![],
            components: vec![IcalComponent {
                name: IcalComponentKind::VEvent.into(),
                props: vec![
                    IcalProp {
                        name: IcalPropKind::Uid.into(),
                        params: vec![],
                        value: IcalValue::Text(IcalText(Cow::Borrowed("42@example.com"))),
                    },
                    IcalProp {
                        name: IcalPropKind::DtStart.into(),
                        params: vec![],
                        value: IcalValue::DateTime(IcalDateTime(Cow::Borrowed("20260102T120000Z"))),
                    },
                    IcalProp {
                        name: IcalPropKind::Summary.into(),
                        params: vec![],
                        value: IcalValue::Text(IcalText(Cow::Borrowed("Lunch"))),
                    },
                ],
                components: vec![],
            }],
        }
    }

    #[test]
    fn a_group_survives_a_conversion_with_no_parser() {
        let group = calendar().to_jscalendar();
        let back = Ical::from_jscalendar(&group).expect("a Group");

        assert_eq!(back.to_jscalendar(), group);
    }

    #[test]
    fn refuses_an_object_that_is_no_calendar_of_ours() {
        let value = serde_json::json!({ "@type": "Alert" });

        assert_eq!(
            Ical::from_jscalendar(&value),
            Err(IcalJscalendarError::NotAGroup("Alert".into()))
        );
    }
}
