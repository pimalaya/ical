//! # Recurrence-rule value
//!
//! The decoded recurrence-rule value kind.
//!
//! Backs `RRULE` and the rule form of `EXRULE` (RFC 5545 3.3.10), including the
//! `RSCALE` and `SKIP` extensions (RFC 7529): a semicolon-separated list of
//! `part=value` rule components such as `FREQ=YEARLY;BYMONTH=1`. The value is
//! kept as its raw text; structured decoding into typed rule parts is deferred
//! to a future addition. Pure data, no escaping; the owning property's wire
//! name lives on [`crate::prop::IcalProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded recurrence-rule value, kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalRecur<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for IcalRecur<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalRecur<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalRecur<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
