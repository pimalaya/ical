//! # Duration value
//!
//! The decoded duration value kind.
//!
//! Backs `DURATION` and the duration form of other properties (RFC 5545
//! 3.3.6): an ISO 8601 duration such as `P15DT5H0M20S` or `-P1D`, always
//! prefixed by `P` (with an optional leading sign). The value is kept as its
//! raw text, so it goes back on the wire exactly as it arrived.
//!
//! [`IcalDuration::seconds`] reads it as a number and
//! [`IcalDuration::from_seconds`] writes one back, which is all the arithmetic
//! the grammar admits: it carries no month and no year, so no calendar is
//! needed to say how long one is.

use alloc::{borrow::Cow, format, string::String};

/// A decoded duration value (ISO 8601 `P...`), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalDuration<'a>(pub Cow<'a, str>);

impl IcalDuration<'_> {
    /// The duration in seconds, a leading `-` making it negative.
    ///
    /// `None` for anything that is not the RFC 5545 3.3.6 `P...` form, parsing
    /// being liberal enough elsewhere to let one through. A week counts as
    /// seven days; a month and a year are not part of the grammar, so nothing
    /// here needs a calendar to answer.
    pub fn seconds(&self) -> Option<i64> {
        let span = self.0.as_ref();

        let (sign, span) = match span.strip_prefix('-') {
            Some(span) => (-1, span),
            None => (1, span.strip_prefix('+').unwrap_or(span)),
        };

        let mut total: i64 = 0;
        let mut amount = String::new();

        for character in span.strip_prefix('P')?.chars() {
            if character.is_ascii_digit() {
                amount.push(character);
                continue;
            }

            // NOTE: The T only separates the date part from the time part;
            // every other letter closes the number before it.
            if character == 'T' {
                continue;
            }

            let unit = match character {
                'W' => 604_800,
                'D' => 86_400,
                'H' => 3_600,
                'M' => 60,
                'S' => 1,
                _ => return None,
            };

            total += amount.parse::<i64>().ok()? * unit;
            amount.clear();
        }

        Some(sign * total)
    }

    /// A number of seconds as a duration, the inverse of
    /// [`seconds`](Self::seconds).
    ///
    /// Days are the largest unit written: a week is spelled in days, since
    /// `P7D` and `P1W` are the same length and only one of them survives a
    /// round trip through a number.
    pub fn from_seconds(seconds: i64) -> IcalDuration<'static> {
        let sign = match seconds < 0 {
            true => "-",
            false => "",
        };

        let seconds = seconds.unsigned_abs();
        let (days, rest) = (seconds / 86_400, seconds % 86_400);
        let (hours, rest) = (rest / 3_600, rest % 3_600);
        let (minutes, seconds) = (rest / 60, rest % 60);

        let mut duration = String::from(sign);
        duration.push('P');

        if days > 0 {
            duration.push_str(&format!("{days}D"));
        }

        if hours == 0 && minutes == 0 && seconds == 0 {
            // NOTE: A whole number of days needs no time part, but a
            // zero-length span still has to spell something.
            if days == 0 {
                duration.push_str("T0S");
            }

            return IcalDuration(Cow::Owned(duration));
        }

        duration.push('T');

        for (amount, unit) in [(hours, 'H'), (minutes, 'M'), (seconds, 'S')] {
            if amount > 0 {
                duration.push_str(&format!("{amount}{unit}"));
            }
        }

        IcalDuration(Cow::Owned(duration))
    }
}

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

#[cfg(test)]
mod tests {
    use crate::value::duration::IcalDuration;

    #[test]
    fn reads_every_unit_the_grammar_admits() {
        assert_eq!(IcalDuration::from("P1W").seconds(), Some(604_800));
        assert_eq!(
            IcalDuration::from("P15DT5H0M20S").seconds(),
            Some(1_314_020)
        );
        assert_eq!(IcalDuration::from("-P1D").seconds(), Some(-86_400));
        assert_eq!(IcalDuration::from("PT0S").seconds(), Some(0));
    }

    #[test]
    fn refuses_what_is_not_a_duration() {
        assert_eq!(IcalDuration::from("").seconds(), None);
        assert_eq!(IcalDuration::from("1D").seconds(), None);
        assert_eq!(IcalDuration::from("P1Y").seconds(), None);
    }

    #[test]
    fn a_duration_written_from_seconds_reads_back_as_those_seconds() {
        for seconds in [0, 1, 59, 60, 3_600, 86_400, 1_314_020, -86_400, -90] {
            let written = IcalDuration::from_seconds(seconds);

            assert_eq!(written.seconds(), Some(seconds), "{}", written.0);
        }
    }

    #[test]
    fn a_week_comes_back_spelled_in_days() {
        assert_eq!(&*IcalDuration::from_seconds(604_800).0, "P7D");
    }
}
