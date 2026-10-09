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
//!
//! Liberal in, strict out: the reading takes what calendars in the wild write
//! (`P1H`, `p1d`, `PT1H20S`), the writing follows the grammar, and whether a
//! value follows it is the [validator](crate::validator)'s question.

use alloc::{borrow::Cow, format, string::String};

/// A decoded duration value (ISO 8601 `P...`), kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IcalDuration<'a>(pub Cow<'a, str>);

impl IcalDuration<'_> {
    /// The duration in seconds, a leading `-` making it negative.
    ///
    /// Lenient, so an event a calendar in the wild ends with `P1H` still
    /// ends: past the `P`, any run of digits closes on a unit letter, in any
    /// case and order, a `T` is skipped and a fraction of second dropped, and
    /// RFC 8984's `P1W2D` reads too. An `M` is a minute wherever it sits, and
    /// a week counts as seven days. `None` only for text that names no length,
    /// a missing `P` or a year in it, which would need a calendar.
    pub fn seconds(&self) -> Option<i64> {
        let span = self.0.as_ref();

        let (sign, span) = match span.strip_prefix('-') {
            Some(span) => (-1, span),
            None => (1, span.strip_prefix('+').unwrap_or(span)),
        };

        let mut total: i64 = 0;
        let mut amount = String::new();
        let mut fraction = false;

        for character in span.strip_prefix(['P', 'p'])?.chars() {
            let unit = match character.to_ascii_uppercase() {
                digit if digit.is_ascii_digit() => {
                    if !fraction {
                        amount.push(digit);
                    }
                    continue;
                }
                // NOTE: The T only separates the date part from the time
                // part, and a fraction of second is below what this counts.
                'T' => continue,
                '.' => {
                    fraction = true;
                    continue;
                }
                'W' => 604_800,
                'D' => 86_400,
                'H' => 3_600,
                'M' => 60,
                'S' => 1,
                _ => return None,
            };

            total = total.checked_add(amount.parse::<i64>().ok()?.checked_mul(unit)?)?;
            amount.clear();
            fraction = false;
        }

        Some(sign * total)
    }

    /// Whether the text follows the RFC 5545 3.3.6 grammar exactly: a week
    /// stands alone, a `T` opens a time part that names a unit, the units come
    /// in order, upper case, and a minute sits between an hour and a second.
    pub(crate) fn conforms(&self) -> bool {
        let text = self.0.as_ref();
        let span = text
            .strip_prefix(['+', '-'])
            .unwrap_or(text)
            .strip_prefix('P');

        let Some(span) = span else {
            return false;
        };

        let (date, time) = match span.split_once('T') {
            Some((date, time)) => (date, Some(time)),
            None => (span, None),
        };

        let (weeks, date) = unit(date, b'W');
        let (days, date) = unit(date, b'D');
        let (hours, rest) = unit(time.unwrap_or(""), b'H');
        let (minutes, rest) = unit(rest, b'M');
        let (seconds, rest) = unit(rest, b'S');

        let timed = hours || minutes || seconds;

        date.is_empty()
            && rest.is_empty()
            && time.is_some() == timed
            && (weeks || days || timed)
            && !(hours && !minutes && seconds)
            && !(weeks && (days || timed))
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

        // NOTE: The grammar has no second straight after an hour, so the
        // minute between them is spelled even when there is none.
        let minutes_spelled = minutes > 0 || hours > 0 && seconds > 0;

        for (amount, unit, spelled) in [
            (hours, 'H', hours > 0),
            (minutes, 'M', minutes_spelled),
            (seconds, 'S', seconds > 0),
        ] {
            if spelled {
                duration.push_str(&format!("{amount}{unit}"));
            }
        }

        IcalDuration(Cow::Owned(duration))
    }
}

/// Whether `text` opens with a run of digits closed by `unit`, and what
/// follows it; `text` untouched when it does not.
fn unit(text: &str, unit: u8) -> (bool, &str) {
    let digits = text.bytes().take_while(u8::is_ascii_digit).count();

    match text.as_bytes().get(digits) {
        Some(&byte) if digits > 0 && byte == unit => (true, &text[digits + 1..]),
        _ => (false, text),
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
    fn reads_every_form_the_grammar_admits() {
        let forms = [
            ("P1W", 604_800),
            ("PT1H", 3_600),
            ("-PT15M", -900),
            ("P1DT2H", 93_600),
            ("P15DT5H0M20S", 1_314_020),
            ("+P1D", 86_400),
            ("PT0S", 0),
        ];

        for (form, seconds) in forms {
            let duration = IcalDuration::from(form);

            assert_eq!(duration.seconds(), Some(seconds), "{form}");
            assert!(duration.conforms(), "{form}");
        }
    }

    #[test]
    fn reads_what_calendars_in_the_wild_write_without_it_conforming() {
        // NOTE: An hour with no `T`, lower case, a second straight after an
        // hour, a week beside a day (RFC 8984 allows it), units out of order,
        // and a fraction of second.
        let forms = [
            ("P1H", 3_600),
            ("p1d", 86_400),
            ("-pt15m", -900),
            ("PT1H20S", 3_620),
            ("P1W2D", 777_600),
            ("PT1M1H", 3_660),
            ("PT1.5S", 1),
        ];

        for (form, seconds) in forms {
            let duration = IcalDuration::from(form);

            assert_eq!(duration.seconds(), Some(seconds), "{form}");
            assert!(!duration.conforms(), "{form}");
        }
    }

    #[test]
    fn reads_no_length_from_text_that_names_none() {
        for form in ["", "1D", "P1Y", "PXD", "P99999999999999999999W"] {
            assert_eq!(IcalDuration::from(form).seconds(), None, "{form}");
        }
    }

    #[test]
    fn refuses_to_conform_what_the_grammar_does_not_admit() {
        // NOTE: A `T` naming nothing, a bare `P`, a week beside a time, a unit
        // repeated, a sign after the `P` and trailing junk.
        for form in [
            "", "1D", "P", "PT", "P1DT", "P1WT1H", "P1D1D", "P-1D", "P1D ",
        ] {
            assert!(!IcalDuration::from(form).conforms(), "{form}");
        }
    }

    #[test]
    fn a_duration_written_from_seconds_reads_back_as_those_seconds() {
        for seconds in [
            0, 1, 59, 60, 3_600, 3_601, 86_400, 86_401, 1_314_020, -86_400, -90,
        ] {
            let written = IcalDuration::from_seconds(seconds);

            assert_eq!(written.seconds(), Some(seconds), "{}", written.0);
            assert!(written.conforms(), "{}", written.0);
        }
    }

    #[test]
    fn a_week_comes_back_spelled_in_days() {
        assert_eq!(&*IcalDuration::from_seconds(604_800).0, "P7D");
    }

    #[test]
    fn an_hour_and_a_second_come_back_with_the_minute_between() {
        assert_eq!(&*IcalDuration::from_seconds(3_601).0, "PT1H0M1S");
    }
}
