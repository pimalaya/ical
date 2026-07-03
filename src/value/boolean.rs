//! # Boolean value
//!
//! The decoded boolean value kind.
//!
//! Backs the boolean-valued properties and parameters (RFC 5545 3.3.2): the
//! case-insensitive tokens `TRUE` and `FALSE`. The value is kept as its raw
//! text so the original casing round-trips; use [`IcalBoolean::is_true`] to
//! read it as a `bool`. Pure data, no escaping; the owning property's wire name
//! lives on [`crate::prop::IcalProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded boolean value (`TRUE` / `FALSE`), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalBoolean<'a>(pub Cow<'a, str>);

impl IcalBoolean<'_> {
    /// Whether the value is the (case-insensitive) token `TRUE`.
    pub fn is_true(&self) -> bool {
        self.0.eq_ignore_ascii_case("TRUE")
    }
}

impl<'a> From<&'a str> for IcalBoolean<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalBoolean<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalBoolean<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::value::boolean::IcalBoolean;

    #[test]
    fn is_true_reads_both_cases() {
        assert!(IcalBoolean::from("TRUE").is_true());
        assert!(IcalBoolean::from("true").is_true());
        assert!(!IcalBoolean::from("FALSE").is_true());
        assert!(!IcalBoolean::from("false").is_true());
    }
}
