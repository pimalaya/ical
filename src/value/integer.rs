//! # Integer value
//!
//! The decoded integer value kind.
//!
//! Backs the integer-valued properties and parameters (RFC 5545 3.3.8): a signed
//! decimal integer such as `1234` or `-9`. The value is kept as its raw text so
//! the original lexical form round-trips; [`IcalInteger::get`] parses it into an
//! [`i64`].

use alloc::{borrow::Cow, string::String};

/// A decoded integer value (signed), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalInteger<'a>(pub Cow<'a, str>);

impl IcalInteger<'_> {
    /// Parse the raw text into an [`i64`]; `None` if it is not a valid integer.
    pub fn get(&self) -> Option<i64> {
        self.0.parse().ok()
    }
}

impl<'a> From<&'a str> for IcalInteger<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalInteger<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalInteger<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::value::integer::IcalInteger;

    #[test]
    fn get_parses_signed_integer() {
        assert_eq!(IcalInteger::from("-9").get(), Some(-9));
        assert_eq!(IcalInteger::from("abc").get(), None);
    }
}
