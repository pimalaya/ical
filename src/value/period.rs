//! # Period value
//!
//! The decoded period-of-time value kind.
//!
//! Backs `FREEBUSY` and the period form of `RDATE` (RFC 5545 3.3.9): a start
//! date-time and either an end date-time (`start/end`) or a duration
//! (`start/duration`), the two components separated by a solidus. The value is
//! kept as its raw text; the crate does not split or decode the two
//! components. Pure data, no escaping; the owning property's wire name lives on
//! [`crate::prop::IcalProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded period value (`start/end` or `start/duration`), kept as its raw
/// text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalPeriod<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for IcalPeriod<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalPeriod<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalPeriod<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
