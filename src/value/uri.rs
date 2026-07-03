//! # URI value
//!
//! The decoded URI value kind.
//!
//! Backs every RFC 5545 property whose value is a URI (RFC 5545 3.3.13):
//! `URL`, `TZURL`, `SOURCE`, `CONFERENCE`, `ATTACH` (when a URI), `IMAGE` (when
//! a URI), and `STYLED-DESCRIPTION`. The reference is kept verbatim as a
//! string; the crate does not parse or validate it. Pure data with no escaping
//! knowledge, like every other type in [`crate::value`]; the owning property's
//! wire name lives on [`crate::prop::IcalProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded URI value, kept verbatim.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalUri<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for IcalUri<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalUri<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalUri<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}
