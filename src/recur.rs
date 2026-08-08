//! # Recurrence expansion
//!
//! The typed recurrence rule and its occurrence iterator (RFC 5545 3.3.10,
//! extended by RFC 7529).
//!
//! [`IcalRecur`](crate::value::recur::IcalRecur) keeps a rule as its raw text,
//! which is what byte-faithful round-tripping needs and all a store or a codec
//! ever wants. This module is the opt-in layer above it:
//! [`IcalRecurRule::parse`] decodes that text into typed parts, and
//! [`expand::IcalRecurExpand`] turns a rule plus a start into the dates it
//! actually denotes.
//!
//! A rule is only ever part of the answer. What a component actually happens on
//! is its whole *recurrence set*: `DTSTART` plus every `RRULE` and `RDATE`,
//! minus every `EXDATE` and `EXRULE`, with the `RECURRENCE-ID` overrides
//! applied. That is [`set::IcalRecurSet`], and it is what a client wants.
//!
//! ## Civil times, no time zones
//!
//! RFC 5545 defines expansion on the local wall-clock time of `DTSTART`: a
//! daily rule on a zoned start recurs at the same local time every day, and a
//! UTC start is simply the case where local is UTC. Expansion therefore never
//! needs a UTC offset, and this module never resolves one. Occurrences are
//! civil [`IcalRecurDateTime`]s, and turning one into an instant, along with
//! the time-zone database, the invalid local times of a spring-forward and the
//! ambiguous ones of a fall-back, belongs to the caller at that boundary.
//!
//! ## Liberal in, strict out
//!
//! Parsing follows the crate's Postel's law: an unrecognised rule part is
//! ignored rather than refused, since RFC 5545 requires exactly that, and a
//! malformed value inside a part the module does claim to understand is an
//! error. The typed rule is not the round-trip path (the syntax tree is), so
//! ignored parts are dropped rather than carried.

use alloc::vec::Vec;
use core::{fmt, num::ParseIntError, str::FromStr};

mod civil;
pub mod expand;
pub mod set;
pub mod validate;

pub use self::validate::{IcalRecurPart, IcalRecurRuleProblem};

/// A civil date and time, with no time zone and no offset.
///
/// The unit both ends of this module speak: the start an expansion runs from,
/// the bound `UNTIL` sets, and every occurrence yielded. Comparison is
/// lexicographic through the derived ordering, which is the chronological
/// order for civil values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct IcalRecurDateTime {
    /// The proleptic Gregorian year, negative before 1 BCE.
    pub year: i32,
    /// The month, 1 to 12.
    pub month: u8,
    /// The day of the month, 1 to 31.
    pub day: u8,
    /// The hour, 0 to 23.
    pub hour: u8,
    /// The minute, 0 to 59.
    pub minute: u8,
    /// The second, 0 to 60, leap seconds included.
    pub second: u8,
}

impl IcalRecurDateTime {
    /// The second this civil date-time names, 1970-01-01T00:00:00 being zero.
    ///
    /// Civil throughout: no offset is applied, so this counts seconds on the
    /// wall clock, not on any timeline. It is the arithmetic unit for a
    /// duration between two civil times.
    pub const fn seconds(&self) -> i64 {
        civil::days_from_civil(self.year, self.month, self.day) * 86_400
            + self.hour as i64 * 3600
            + self.minute as i64 * 60
            + self.second as i64
    }

    /// The civil date-time a second count names, the inverse of
    /// [`seconds`](Self::seconds).
    pub const fn from_seconds(seconds: i64) -> Self {
        let days = seconds.div_euclid(86_400);
        let rest = seconds.rem_euclid(86_400);
        let (year, month, day) = civil::civil_from_days(days);

        Self {
            year,
            month,
            day,
            hour: (rest / 3600) as u8,
            minute: (rest % 3600 / 60) as u8,
            second: (rest % 60) as u8,
        }
    }

    /// Builds a date at midnight.
    pub const fn date(year: i32, month: u8, day: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour: 0,
            minute: 0,
            second: 0,
        }
    }

    /// Parses the `DATE` and `DATE-TIME` forms of RFC 5545 3.3.4 and 3.3.5.
    ///
    /// Accepts `YYYYMMDD`, `YYYYMMDDTHHMMSS` and the `Z`-suffixed UTC form,
    /// the three spellings `UNTIL` admits. The suffix is consumed and not
    /// recorded: this type is civil, and a UTC `UNTIL` against a zoned start
    /// is the caller's to convert (see [`IcalRecurRule::until`]).
    pub fn parse(value: &str) -> Result<Self, IcalRecurRuleError> {
        let bytes = value.as_bytes();
        let naive = match bytes.len() {
            8 => value,
            15 => value,
            16 if bytes[15] == b'Z' || bytes[15] == b'z' => &value[..15],
            _ => return Err(IcalRecurRuleError::DateTime),
        };
        if naive.len() == 15 && !matches!(naive.as_bytes()[8], b'T' | b't') {
            return Err(IcalRecurRuleError::DateTime);
        }

        let num = |range: core::ops::Range<usize>| -> Result<u32, IcalRecurRuleError> {
            naive
                .get(range)
                .ok_or(IcalRecurRuleError::DateTime)?
                .parse()
                .map_err(|_| IcalRecurRuleError::DateTime)
        };

        let mut parsed = Self::date(num(0..4)? as i32, num(4..6)? as u8, num(6..8)? as u8);
        if naive.len() == 15 {
            parsed.hour = num(9..11)? as u8;
            parsed.minute = num(11..13)? as u8;
            parsed.second = num(13..15)? as u8;
        }

        let valid_date = (1..=12).contains(&parsed.month)
            && parsed.day >= 1
            && parsed.day <= civil::days_in_month(parsed.year, parsed.month);
        let valid_time = parsed.hour < 24 && parsed.minute < 60 && parsed.second < 61;
        if !valid_date || !valid_time {
            return Err(IcalRecurRuleError::DateTime);
        }

        Ok(parsed)
    }
}

/// The frequency a rule repeats at (`FREQ`).
///
/// The one required rule part, and the axis every `BY` part is read against:
/// the same part expands the candidate set at one frequency and narrows it at
/// another (RFC 5545 3.3.10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IcalRecurFreq {
    /// Every second.
    Secondly,
    /// Every minute.
    Minutely,
    /// Every hour.
    Hourly,
    /// Every day.
    Daily,
    /// Every week, starting on the rule's `WKST`.
    Weekly,
    /// Every month.
    Monthly,
    /// Every year.
    Yearly,
}

impl FromStr for IcalRecurFreq {
    type Err = IcalRecurRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("SECONDLY") {
            Ok(Self::Secondly)
        } else if s.eq_ignore_ascii_case("MINUTELY") {
            Ok(Self::Minutely)
        } else if s.eq_ignore_ascii_case("HOURLY") {
            Ok(Self::Hourly)
        } else if s.eq_ignore_ascii_case("DAILY") {
            Ok(Self::Daily)
        } else if s.eq_ignore_ascii_case("WEEKLY") {
            Ok(Self::Weekly)
        } else if s.eq_ignore_ascii_case("MONTHLY") {
            Ok(Self::Monthly)
        } else if s.eq_ignore_ascii_case("YEARLY") {
            Ok(Self::Yearly)
        } else {
            Err(IcalRecurRuleError::Freq)
        }
    }
}

/// A day of the week, as `BYDAY` and `WKST` spell it.
///
/// Ordered from Sunday so the discriminant matches the day number the civil
/// arithmetic produces, which is what makes the weekday of a date a cast.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IcalRecurWeekday {
    /// `SU`.
    Sunday = 0,
    /// `MO`.
    Monday = 1,
    /// `TU`.
    Tuesday = 2,
    /// `WE`.
    Wednesday = 3,
    /// `TH`.
    Thursday = 4,
    /// `FR`.
    Friday = 5,
    /// `SA`.
    Saturday = 6,
}

impl FromStr for IcalRecurWeekday {
    type Err = IcalRecurRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("SU") {
            Ok(Self::Sunday)
        } else if s.eq_ignore_ascii_case("MO") {
            Ok(Self::Monday)
        } else if s.eq_ignore_ascii_case("TU") {
            Ok(Self::Tuesday)
        } else if s.eq_ignore_ascii_case("WE") {
            Ok(Self::Wednesday)
        } else if s.eq_ignore_ascii_case("TH") {
            Ok(Self::Thursday)
        } else if s.eq_ignore_ascii_case("FR") {
            Ok(Self::Friday)
        } else if s.eq_ignore_ascii_case("SA") {
            Ok(Self::Saturday)
        } else {
            Err(IcalRecurRuleError::Weekday)
        }
    }
}

/// One `BYDAY` entry: a weekday, optionally ordinal.
///
/// The ordinal counts occurrences of that weekday inside the period the
/// frequency defines, forward when positive and backward when negative, so
/// `-1SU` is the last Sunday of a monthly or yearly period. It is meaningless
/// at any other frequency and RFC 5545 forbids it there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IcalRecurWeekdayNum {
    /// The occurrence within the period, `None` when unqualified.
    pub ordinal: Option<i16>,
    /// The weekday itself.
    pub weekday: IcalRecurWeekday,
}

impl FromStr for IcalRecurWeekdayNum {
    type Err = IcalRecurRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE: The weekday is the last two characters, which is not the last two
        // bytes: a rule is text off the wire, so splitting on a byte offset
        // would cut a multi-byte character in half and panic.
        let split = s.char_indices().rev().nth(1).map_or(0, |(index, _)| index);
        let (ordinal, weekday) = s.split_at(split);
        let weekday = weekday.parse()?;
        let ordinal = match ordinal {
            "" => None,
            ordinal => Some(ordinal.parse().map_err(IcalRecurRuleError::Ordinal)?),
        };
        Ok(Self { ordinal, weekday })
    }
}

/// How RFC 7529 resolves a date its calendar scale cannot express.
///
/// Only ever consulted for a non-Gregorian `RSCALE`, or for a leap day a
/// Gregorian year lacks. Parsed so a rule carrying it round-trips through the
/// typed form; expansion honours [`Self::Omit`], the default, and treats the
/// other two as omission until a non-Gregorian scale is supported.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IcalRecurSkip {
    /// Drop the unrepresentable date. The default.
    #[default]
    Omit,
    /// Move it back to the closest valid date.
    Backward,
    /// Move it forward to the closest valid date.
    Forward,
}

impl FromStr for IcalRecurSkip {
    type Err = IcalRecurRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("OMIT") {
            Ok(Self::Omit)
        } else if s.eq_ignore_ascii_case("BACKWARD") {
            Ok(Self::Backward)
        } else if s.eq_ignore_ascii_case("FORWARD") {
            Ok(Self::Forward)
        } else {
            Err(IcalRecurRuleError::Skip)
        }
    }
}

/// A decoded recurrence rule.
///
/// Every part RFC 5545 3.3.10 defines, plus the RFC 7529 extensions, in the
/// shape expansion consumes. Absent parts stay empty or `None` rather than
/// being defaulted from a start date: the defaulting rules depend on the
/// frequency and belong to [`expand::IcalRecurExpand`], not to the decoded
/// value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcalRecurRule {
    /// The frequency, the only required part.
    pub freq: IcalRecurFreq,
    /// The last date the rule may yield, inclusive.
    ///
    /// NOTE: RFC 5545 requires this to be UTC whenever `DTSTART` is zoned,
    /// while expansion is civil throughout. A caller whose start carries a
    /// `TZID` therefore converts this bound into that zone before expanding;
    /// left unconverted it is off by the zone's offset at the boundary.
    pub until: Option<IcalRecurDateTime>,
    /// How many occurrences the rule yields in total, the start included.
    pub count: Option<u32>,
    /// How many frequency periods separate two occurrences, at least one.
    pub interval: u32,
    /// `BYSECOND`, seconds of the minute, 0 to 60.
    pub by_second: Vec<u8>,
    /// `BYMINUTE`, minutes of the hour, 0 to 59.
    pub by_minute: Vec<u8>,
    /// `BYHOUR`, hours of the day, 0 to 23.
    pub by_hour: Vec<u8>,
    /// `BYDAY`, weekdays, each optionally ordinal.
    pub by_day: Vec<IcalRecurWeekdayNum>,
    /// `BYMONTHDAY`, days of the month, 1 to 31 or -31 to -1.
    pub by_month_day: Vec<i8>,
    /// `BYYEARDAY`, days of the year, 1 to 366 or -366 to -1.
    pub by_year_day: Vec<i16>,
    /// `BYWEEKNO`, weeks of the year, 1 to 53 or -53 to -1.
    pub by_week_no: Vec<i8>,
    /// `BYMONTH`, months of the year, 1 to 12.
    pub by_month: Vec<u8>,
    /// `BYSETPOS`, positions within one period's candidate set.
    ///
    /// Applied last, once the other parts have produced and ordered the
    /// candidates of a period, counting from the start when positive and from
    /// the end when negative.
    pub by_set_pos: Vec<i16>,
    /// `WKST`, the weekday a week starts on. Monday unless stated.
    pub week_start: IcalRecurWeekday,
    /// `RSCALE`, the RFC 7529 calendar scale, uppercased.
    ///
    /// Anything other than `GREGORIAN` is decoded and then refused by
    /// expansion, which implements the Gregorian scale alone.
    pub scale: Option<alloc::string::String>,
    /// `SKIP`, the RFC 7529 resolution of an unrepresentable date.
    pub skip: IcalRecurSkip,
}

impl IcalRecurRule {
    /// Decodes a rule from the raw text of a `RRULE` or `EXRULE` value.
    ///
    /// Unrecognised parts are ignored, as RFC 5545 requires; a malformed
    /// value inside a known part is an error. `FREQ` is mandatory, and
    /// `UNTIL` with `COUNT` is refused since the RFC declares them mutually
    /// exclusive.
    pub fn parse(value: &str) -> Result<Self, IcalRecurRuleError> {
        let mut freq = None;
        let mut rule = Self {
            freq: IcalRecurFreq::Daily,
            until: None,
            count: None,
            interval: 1,
            by_second: Vec::new(),
            by_minute: Vec::new(),
            by_hour: Vec::new(),
            by_day: Vec::new(),
            by_month_day: Vec::new(),
            by_year_day: Vec::new(),
            by_week_no: Vec::new(),
            by_month: Vec::new(),
            by_set_pos: Vec::new(),
            week_start: IcalRecurWeekday::Monday,
            scale: None,
            skip: IcalRecurSkip::Omit,
        };

        for part in value.split(';').filter(|part| !part.is_empty()) {
            let Some((name, raw)) = part.split_once('=') else {
                continue;
            };
            let name = name.trim();
            let raw = raw.trim();

            if name.eq_ignore_ascii_case("FREQ") {
                freq = Some(raw.parse()?);
            } else if name.eq_ignore_ascii_case("UNTIL") {
                rule.until = Some(IcalRecurDateTime::parse(raw)?);
            } else if name.eq_ignore_ascii_case("COUNT") {
                rule.count = Some(raw.parse().map_err(IcalRecurRuleError::Count)?);
            } else if name.eq_ignore_ascii_case("INTERVAL") {
                let interval = raw.parse().map_err(IcalRecurRuleError::Interval)?;
                if interval == 0 {
                    return Err(IcalRecurRuleError::IntervalZero);
                }
                rule.interval = interval;
            } else if name.eq_ignore_ascii_case("BYSECOND") {
                rule.by_second = numbers(raw, 0, 60)?;
            } else if name.eq_ignore_ascii_case("BYMINUTE") {
                rule.by_minute = numbers(raw, 0, 59)?;
            } else if name.eq_ignore_ascii_case("BYHOUR") {
                rule.by_hour = numbers(raw, 0, 23)?;
            } else if name.eq_ignore_ascii_case("BYDAY") {
                rule.by_day = weekday_nums(raw)?;
            } else if name.eq_ignore_ascii_case("BYMONTHDAY") {
                rule.by_month_day = signed(raw, 1, 31)?;
            } else if name.eq_ignore_ascii_case("BYYEARDAY") {
                rule.by_year_day = signed(raw, 1, 366)?;
            } else if name.eq_ignore_ascii_case("BYWEEKNO") {
                rule.by_week_no = signed(raw, 1, 53)?;
            } else if name.eq_ignore_ascii_case("BYMONTH") {
                rule.by_month = numbers(raw, 1, 12)?;
            } else if name.eq_ignore_ascii_case("BYSETPOS") {
                rule.by_set_pos = signed(raw, 1, 366)?;
            } else if name.eq_ignore_ascii_case("WKST") {
                rule.week_start = raw.parse()?;
            } else if name.eq_ignore_ascii_case("RSCALE") {
                rule.scale = Some(raw.to_ascii_uppercase());
            } else if name.eq_ignore_ascii_case("SKIP") {
                rule.skip = raw.parse()?;
            }
        }

        // NOTE: A rule carrying both UNTIL and COUNT breaks RFC 5545 3.3.10, but
        // it is still a rule, and refusing it here would be strictness applied
        // on the way in. `validate` is where that is reported, as
        // `UntilWithCount`; expansion honours whichever bound comes first.
        rule.freq = freq.ok_or(IcalRecurRuleError::FreqMissing)?;

        Ok(rule)
    }
}

/// Parses a comma-separated list of unsigned numbers, bounds inclusive.
fn numbers<T>(raw: &str, min: i32, max: i32) -> Result<Vec<T>, IcalRecurRuleError>
where
    T: TryFrom<i32>,
{
    let mut parsed = Vec::new();
    for item in raw.split(',').filter(|item| !item.is_empty()) {
        let value: i32 = item.trim().parse().map_err(IcalRecurRuleError::Number)?;
        if value < min || value > max {
            return Err(IcalRecurRuleError::Range);
        }
        parsed.push(T::try_from(value).map_err(|_| IcalRecurRuleError::Range)?);
    }
    Ok(parsed)
}

/// Parses a comma-separated list of signed numbers, zero excluded.
///
/// The bounds apply to the magnitude, since every signed `BY` part admits the
/// same range forward and backward.
fn signed<T>(raw: &str, min: i32, max: i32) -> Result<Vec<T>, IcalRecurRuleError>
where
    T: TryFrom<i32>,
{
    let mut parsed = Vec::new();
    for item in raw.split(',').filter(|item| !item.is_empty()) {
        let value: i32 = item.trim().parse().map_err(IcalRecurRuleError::Number)?;
        let magnitude = value.unsigned_abs() as i32;
        if value == 0 || magnitude < min || magnitude > max {
            return Err(IcalRecurRuleError::Range);
        }
        parsed.push(T::try_from(value).map_err(|_| IcalRecurRuleError::Range)?);
    }
    Ok(parsed)
}

/// Parses a comma-separated `BYDAY` list.
fn weekday_nums(raw: &str) -> Result<Vec<IcalRecurWeekdayNum>, IcalRecurRuleError> {
    let mut parsed = Vec::new();
    for item in raw.split(',').filter(|item| !item.is_empty()) {
        parsed.push(item.trim().parse()?);
    }
    Ok(parsed)
}

/// Everything that can go wrong decoding a rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcalRecurRuleError {
    /// `FREQ` names no frequency this module knows.
    Freq,
    /// The rule carries no `FREQ`, which RFC 5545 requires.
    FreqMissing,
    /// A weekday is not one of the seven two-letter codes.
    Weekday,
    /// `SKIP` names no RFC 7529 resolution.
    Skip,
    /// `UNTIL` is not a `DATE` or `DATE-TIME`, or names no real instant.
    DateTime,
    /// `COUNT` is not a number.
    Count(ParseIntError),
    /// `INTERVAL` is not a number.
    Interval(ParseIntError),
    /// `INTERVAL` is zero, which would never advance.
    IntervalZero,
    /// A `BYDAY` ordinal is not a number.
    Ordinal(ParseIntError),
    /// A `BY` part holds something that is not a number.
    Number(ParseIntError),
    /// A `BY` part holds a number outside the range RFC 5545 allows.
    Range,
}

impl fmt::Display for IcalRecurRuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Freq => write!(f, "Unknown recurrence frequency"),
            Self::FreqMissing => write!(f, "Missing recurrence frequency"),
            Self::Weekday => write!(f, "Unknown recurrence weekday"),
            Self::Skip => write!(f, "Unknown recurrence skip"),
            Self::DateTime => write!(f, "Invalid recurrence date or date-time"),
            Self::Count(err) => write!(f, "Invalid recurrence count: {err}"),
            Self::Interval(err) => write!(f, "Invalid recurrence interval: {err}"),
            Self::IntervalZero => write!(f, "Recurrence interval cannot be zero"),
            Self::Ordinal(err) => write!(f, "Invalid recurrence weekday ordinal: {err}"),
            Self::Number(err) => write!(f, "Invalid recurrence number: {err}"),
            Self::Range => write!(f, "Recurrence number out of range"),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::recur::*;

    #[test]
    fn parses_every_part() {
        let rule = IcalRecurRule::parse(
            "FREQ=YEARLY;INTERVAL=2;BYMONTH=1,3;BYDAY=-1SU,MO;BYMONTHDAY=1,-1;\
             BYYEARDAY=100,-1;BYWEEKNO=1,-1;BYHOUR=9;BYMINUTE=30;BYSECOND=0;\
             BYSETPOS=-1;WKST=SU;UNTIL=20301231T235959Z",
        )
        .unwrap();

        assert_eq!(rule.freq, IcalRecurFreq::Yearly);
        assert_eq!(rule.interval, 2);
        assert_eq!(rule.by_month, vec![1, 3]);
        assert_eq!(rule.by_day[0].ordinal, Some(-1));
        assert_eq!(rule.by_day[0].weekday, IcalRecurWeekday::Sunday);
        assert_eq!(rule.by_day[1].ordinal, None);
        assert_eq!(rule.by_month_day, vec![1, -1]);
        assert_eq!(rule.by_year_day, vec![100, -1]);
        assert_eq!(rule.by_week_no, vec![1, -1]);
        assert_eq!(rule.by_set_pos, vec![-1]);
        assert_eq!(rule.week_start, IcalRecurWeekday::Sunday);
        assert_eq!(
            rule.until,
            Some(IcalRecurDateTime {
                year: 2030,
                month: 12,
                day: 31,
                hour: 23,
                minute: 59,
                second: 59,
            })
        );
    }

    #[test]
    fn defaults_interval_and_week_start() {
        let rule = IcalRecurRule::parse("FREQ=DAILY").unwrap();
        assert_eq!(rule.interval, 1);
        assert_eq!(rule.week_start, IcalRecurWeekday::Monday);
        assert_eq!(rule.skip, IcalRecurSkip::Omit);
    }

    #[test]
    fn ignores_unknown_parts() {
        let rule = IcalRecurRule::parse("FREQ=DAILY;X-VENDOR=1;NONSENSE=abc").unwrap();
        assert_eq!(rule.freq, IcalRecurFreq::Daily);
    }

    #[test]
    fn refuses_contradictions() {
        assert_eq!(
            IcalRecurRule::parse("INTERVAL=2"),
            Err(IcalRecurRuleError::FreqMissing)
        );
        assert_eq!(
            IcalRecurRule::parse("FREQ=DAILY;INTERVAL=0"),
            Err(IcalRecurRuleError::IntervalZero)
        );
        assert_eq!(
            IcalRecurRule::parse("FREQ=FORTNIGHTLY"),
            Err(IcalRecurRuleError::Freq)
        );
    }

    #[test]
    fn refuses_out_of_range_and_zero_ordinals() {
        assert_eq!(
            IcalRecurRule::parse("FREQ=MONTHLY;BYMONTHDAY=32"),
            Err(IcalRecurRuleError::Range)
        );
        assert_eq!(
            IcalRecurRule::parse("FREQ=MONTHLY;BYMONTHDAY=0"),
            Err(IcalRecurRuleError::Range)
        );
        assert_eq!(
            IcalRecurRule::parse("FREQ=YEARLY;BYMONTH=13"),
            Err(IcalRecurRuleError::Range)
        );
    }

    #[test]
    fn refuses_a_multi_byte_weekday_rather_than_splitting_it() {
        // NOTE: A rule is text off the wire, so a BYDAY whose last characters are
        // multi-byte must be refused, never split mid-character.
        for value in ["€", "𝄞", "SU€", "-1€", "1FR€"] {
            assert!(IcalRecurRule::parse(&alloc::format!("FREQ=DAILY;BYDAY={value}")).is_err());
        }
    }

    #[test]
    fn parses_the_three_until_spellings() {
        let date = IcalRecurRule::parse("FREQ=DAILY;UNTIL=20300102").unwrap();
        assert_eq!(date.until, Some(IcalRecurDateTime::date(2030, 1, 2)));

        let local = IcalRecurRule::parse("FREQ=DAILY;UNTIL=20300102T030405").unwrap();
        let utc = IcalRecurRule::parse("FREQ=DAILY;UNTIL=20300102T030405Z").unwrap();
        assert_eq!(local.until, utc.until);
        assert_eq!(local.until.unwrap().hour, 3);

        assert_eq!(
            IcalRecurRule::parse("FREQ=DAILY;UNTIL=20300230"),
            Err(IcalRecurRuleError::DateTime)
        );
    }
}
