//! # JSCalendar
//!
//! The RFC 8984 conversion: the decoded calendar as a JSCalendar `Group`, and
//! back.
//!
//! [`Ical::to_jscalendar`] writes the decoded model as the JSON object a JMAP
//! calendar server exchanges; [`Ical::from_jscalendar`] reads one back,
//! borrowing the JSON tree's strings where it can. There is no JSCalendar model
//! in this crate: a Group is a plain [`serde_json::Value`], and iCalendar stays
//! the one decoded model, exactly as [`jcal`](crate::jcal) leaves it.
//!
//! ## A re-modelling, not a re-encoding
//!
//! jCal spells the same model in JSON. JSCalendar is a different model: a
//! `VCALENDAR` is a Group of Events and Tasks (RFC 8984 2.1, 2.2, 5.3), a
//! `DTEND` is a duration, an `ATTENDEE` line is a Participant object, a `VALARM`
//! is an Alert, and an overriding `VEVENT` is not a component at all but a patch
//! inside the series it overrides. The conversion rules are those of
//! [draft-ietf-calext-jscalendar-icalendar](https://datatracker.ietf.org/doc/draft-ietf-calext-jscalendar-icalendar/),
//! read against the published RFC 8984 rather than its successor: where the
//! draft names a member only JSCalendar 2.0 has, this crate writes RFC 8984's
//! (`recurrenceRules` rather than `recurrenceRule`, `sendTo` and `replyTo`
//! rather than `calendarAddress`).
//!
//! ## Nothing is dropped
//!
//! Both directions are lossless through an escape hatch, and only a non-object
//! root can fail the import. Exporting, a property or component with no
//! JSCalendar counterpart is kept whole in the object's `iCalendar` member, in
//! jCal syntax, and a parameter left over after a property converts is kept in
//! that member's `convertedProperties` record (draft 5.1.1). The same record
//! names the property a member came from wherever more than one could have, so
//! `updated` knows whether it was a `DTSTAMP` or a `LAST-MODIFIED`.
//!
//! Importing, the mirror hatch applies: a member with no iCalendar counterpart
//! becomes a `JSPROP` property holding its JSON, located by a `JSPTR` parameter
//! (draft 4.1.2, 4.2.2), and a collection key that was not simply the element's
//! position is carried on a `JSID` parameter so it survives the next conversion.
//!
//! ## What normalises
//!
//! Three things do not survive a round trip unchanged, and none of them is
//! recoverable from the JSON alone.
//!
//! An `RRULE`'s `UNTIL` is stated in UTC whenever `DTSTART` is, but RFC 8984
//! states it in the object's own time zone. Shifting between the two needs the
//! time-zone database, which this crate does not carry, so the wall-clock digits
//! are carried across unshifted: exact for a floating or UTC object, and off by
//! that zone's offset for any other. The whole of [`timezone`](crate::timezone)
//! is available to a caller that wants to shift it from the calendar's own
//! `VTIMEZONE`.
//!
//! A `DTEND` becomes a duration, so an event that ended in another time zone
//! than it started in comes back with the start's zone on both ends.
//!
//! Ordering inside a component is lost, since a JSCalendar object is a set of
//! members rather than a list of lines. Byte fidelity is the syntax tree's job;
//! JSCalendar is a projection of the decoded model, one further removed than
//! jCal is.

use core::{error, fmt};

use alloc::string::{String, ToString};

use serde_json::Value;

use crate::ical::Ical;

mod export;
mod hatch;
mod import;
mod patch;

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

impl Ical<'_> {
    /// The calendar as an RFC 8984 JSCalendar `Group` value.
    ///
    /// Infallible: what the mapping cannot express is preserved in the
    /// `iCalendar` escape hatch rather than dropped.
    pub fn to_jscalendar(&self) -> Value {
        export::group(self)
    }
}

impl<'a> Ical<'a> {
    /// Read a calendar back from an RFC 8984 JSCalendar value.
    ///
    /// A `Group` is a whole calendar. A lone `Event` or `Task` is the calendar
    /// holding it, since that is what a JMAP calendar server hands out one
    /// object at a time.
    ///
    /// Liberal: only a root that is none of those errors; a member with no
    /// iCalendar counterpart is preserved as a `JSPROP` property.
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
