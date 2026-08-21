//! # Calendar-address value
//!
//! The decoded calendar-address value kind.
//!
//! Backs `ORGANIZER`, `ATTENDEE` and `CALENDAR-ADDRESS` (RFC 5545 3.3.3): a URI
//! identifying a calendar user, most commonly a `mailto:` address. The reference
//! is kept verbatim as a string; the crate does not parse or validate it.

use alloc::{borrow::Cow, string::String};

/// A decoded calendar-address value (a URI, usually `mailto:`), kept verbatim.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalCalAddress<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for IcalCalAddress<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalCalAddress<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalCalAddress<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
