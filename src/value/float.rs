//! # Float value
//!
//! The decoded float value kind.
//!
//! Backs the float-valued properties and parameters (RFC 5545 3.3.7): a signed
//! real number such as `1000000.0000001` or `-3.14`. The value is kept as its
//! raw text so the original lexical form round-trips; use [`IcalFloat::get`] to
//! parse it into an [`f64`]. Pure data, no escaping; the owning property's wire
//! name lives on [`crate::prop::IcalProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded float value, kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalFloat<'a>(pub Cow<'a, str>);

impl IcalFloat<'_> {
    /// Parse the raw text into an [`f64`]; `None` if it is not a valid float.
    pub fn get(&self) -> Option<f64> {
        self.0.parse().ok()
    }
}

impl<'a> From<&'a str> for IcalFloat<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for IcalFloat<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for IcalFloat<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::value::float::IcalFloat;

    #[test]
    fn get_parses_signed_float() {
        assert_eq!(IcalFloat::from("-12.5").get(), Some(-12.5));
        assert_eq!(IcalFloat::from("abc").get(), None);
    }
}
