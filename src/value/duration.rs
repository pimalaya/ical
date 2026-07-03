//! # Duration value
//!
//! The decoded duration value kind.
//!
//! Backs `DURATION` and the duration form of other properties (RFC 5545
//! 3.3.6): an ISO 8601 duration such as `P15DT5H0M20S` or `-P1D`, always
//! prefixed by `P` (with an optional leading sign). The value is kept as its
//! raw text; the crate does not parse it into day/hour/minute/second
//! components. Pure data, no escaping; the owning property's wire name lives on
//! [`crate::prop::IcalProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded duration value (ISO 8601 `P...`), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalDuration<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for IcalDuration<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalDuration<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalDuration<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
