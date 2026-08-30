//! # Date and time values
//!
//! The decoded date/time value kinds: a calendar date, a date-time, and a time.
//!
//! [`IcalDate`] (RFC 5545 3.3.4, `YYYYMMDD`) backs date-valued properties and
//! [`IcalTime`] (RFC 5545 3.3.12, `HHMMSS` with an optional `Z` suffix) backs
//! bare time values.
//!
//! [`IcalDateTime`] (RFC 5545 3.3.5, `YYYYMMDDTHHMMSS` with an optional `Z`
//! suffix or a companion `TZID` parameter) backs `DTSTART` / `DTEND` /
//! `DTSTAMP` and friends.
//!
//! All three are kept as their raw text rather than decoded into calendar
//! fields: the grammars carry timezone-dependent and reduced-precision forms
//! whose meaning depends on companion parameters, so decoding here would risk
//! a lossy round-trip.
//!
//! Callers that need calendar semantics parse the string themselves.

use alloc::{borrow::Cow, string::String, vec::Vec};

/// A decoded date value (`YYYYMMDD`), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalDate<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for IcalDate<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalDate<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalDate<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

/// A decoded comma-separated list of date-ish values, each kept as raw text.
///
/// Backs `RDATE` and `EXDATE`, which RFC 5545 3.8.5.1 and 3.8.5.2 define as
/// lists rather than single values. An item is a `DATE`, a `DATE-TIME` or (for
/// `RDATE`) a `PERIOD`, whichever the `VALUE` parameter declares, kept exactly
/// as written so a period keeps its `start/end` or `start/duration` form.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalDateTimeList<'a>(pub Vec<Cow<'a, str>>);

impl<'a> From<Vec<Cow<'a, str>>> for IcalDateTimeList<'a> {
    fn from(values: Vec<Cow<'a, str>>) -> Self {
        Self(values)
    }
}

/// A decoded date-time value (`YYYYMMDDTHHMMSS`, optional `Z` or `TZID`), kept
/// as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalDateTime<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for IcalDateTime<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalDateTime<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalDateTime<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

/// A decoded time value (`HHMMSS`, optional `Z`), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalTime<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for IcalTime<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalTime<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalTime<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
