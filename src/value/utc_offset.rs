//! # UTC-offset value
//!
//! The decoded UTC-offset value kind.
//!
//! Backs `TZOFFSETFROM` and `TZOFFSETTO`: a signed `+/-HHMM[SS]` offset from UTC
//! (RFC 5545 3.3.14; e.g. `-0500`), kept as its raw text. The crate does not
//! decode it into hours, minutes and seconds.

use alloc::{borrow::Cow, string::String};

/// A decoded UTC-offset value (signed `hhmm`), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalUtcOffset<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for IcalUtcOffset<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalUtcOffset<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalUtcOffset<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
