//! # UTC-offset value
//!
//! The decoded UTC-offset value kind.
//!
//! Backs `TZOFFSETFROM` and `TZOFFSETTO`: a signed `+/-HHMM[SS]` offset from
//! UTC (RFC 5545 3.3.14; e.g. `-0500`), kept as its raw text so it goes back
//! on the wire exactly as it arrived.
//!
//! [`IcalUtcOffset::seconds`] reads it as a number for a caller that needs to
//! apply it, and for [`crate::tz`], which resolves a civil time against the
//! `VTIMEZONE` a calendar carries.

use core::ops::Range;

use alloc::{borrow::Cow, string::String};

/// A decoded UTC-offset value (signed `hhmm`), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalUtcOffset<'a>(pub Cow<'a, str>);

impl IcalUtcOffset<'_> {
    /// The offset in seconds east of UTC, so `-0500` reads as `-18000`.
    ///
    /// `None` for anything that is not the RFC 5545 3.3.14 `+/-hhmm[ss]`
    /// form, parsing being liberal enough elsewhere to let one through.
    pub fn seconds(&self) -> Option<i32> {
        let text = self.0.as_ref();

        let (sign, digits) = match text.as_bytes().first()? {
            b'+' => (1, &text[1..]),
            b'-' => (-1, &text[1..]),
            _ => (1, text),
        };

        if !matches!(digits.len(), 4 | 6) || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }

        let part = |range: Range<usize>| digits[range].parse::<i32>().ok();

        let hours = part(0..2)?;
        let minutes = part(2..4)?;
        let seconds = if digits.len() == 6 { part(4..6)? } else { 0 };

        Some(sign * (hours * 3600 + minutes * 60 + seconds))
    }
}

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

#[cfg(test)]
mod tests {
    use crate::value::utc_offset::IcalUtcOffset;

    #[test]
    fn reads_every_offset_spelling() {
        assert_eq!(IcalUtcOffset::from("-0500").seconds(), Some(-18_000));
        assert_eq!(IcalUtcOffset::from("+0100").seconds(), Some(3_600));
        assert_eq!(IcalUtcOffset::from("+053045").seconds(), Some(19_845));
        assert_eq!(IcalUtcOffset::from("0000").seconds(), Some(0));
    }

    #[test]
    fn refuses_what_is_not_an_offset() {
        assert_eq!(IcalUtcOffset::from("").seconds(), None);
        assert_eq!(IcalUtcOffset::from("+5").seconds(), None);
        assert_eq!(IcalUtcOffset::from("+05:00").seconds(), None);
        assert_eq!(IcalUtcOffset::from("+0h00").seconds(), None);
    }
}
