//! # Date and time values
//!
//! The decoded date/time value kinds: a calendar date, a date-time, and a
//! time.
//!
//! [`IcalDate`] (RFC 5545 3.3.4, `YYYYMMDD`) backs date-valued properties,
//! [`IcalDateTime`] (RFC 5545 3.3.5, `YYYYMMDDTHHMMSS` with an optional `Z`
//! suffix or a companion `TZID` parameter) backs `DTSTART` / `DTEND` /
//! `DTSTAMP` and friends, and [`IcalTime`] (RFC 5545 3.3.12, `HHMMSS` with an
//! optional `Z` suffix) backs bare time values. All three are kept as their
//! raw text rather than decoded into calendar fields: the grammars carry
//! timezone-dependent and reduced-precision forms whose meaning depends on
//! companion parameters, so decoding here would risk a lossy round-trip.
//! Callers that need calendar semantics parse the string themselves. Pure
//! data, no escaping; the owning property's wire name lives on
//! [`crate::prop::IcalProp::name`].

use alloc::{borrow::Cow, string::String};

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
