//! # Occurrence expansion
//!
//! [`IcalRecurExpand`], the iterator turning a rule and a start into the civil
//! date-times the rule denotes (RFC 5545 3.3.10).
//!
//! ## What one step does
//!
//! Expansion walks the periods the frequency defines, `INTERVAL` of them at a
//! time, beginning with the period `DTSTART` falls in. Each period becomes an
//! ordered candidate set: the `BY` parts either widen that set or narrow it,
//! which of the two depending on the frequency, and `BYSETPOS` then picks
//! positions out of the finished set. What survives is yielded in order before
//! the next period is looked at, so nothing is ever buffered beyond one
//! period and an endless rule is walked lazily.
//!
//! `DTSTART` bounds the walk but is not forced into it: candidates before it
//! are dropped, and a start the rule does not itself generate is simply never
//! yielded. RFC 5545 3.8.5.3 leaves that case undefined, and dropping matches
//! what the widely used implementations settled on.
//!
//! ## Unsatisfiable rules end
//!
//! `FREQ=YEARLY;BYMONTH=2;BYMONTHDAY=30` names a date no year has, and a rule
//! like it would otherwise search forever for a next occurrence. Expansion
//! stops at year 9999, the last year an RFC 5545 date can spell, and periods
//! that cannot hold a candidate are skipped whole rather than stepped through,
//! so reaching that end stays cheap even at `FREQ=SECONDLY`. A period the date
//! gates admit but `BYSETPOS` then empties cannot be skipped that way, so a
//! second bound, a budget of barren periods per occurrence, keeps a rule such
//! as `FREQ=SECONDLY;BYSETPOS=2` from walking to year 9999 a second at a time.
//! Neither bound truncates a rule that does yield: an occurrence is an
//! occurrence however far apart from the last one it falls.

use alloc::{vec, vec::Vec};

use crate::recur::{IcalRecurDateTime, IcalRecurFreq, IcalRecurRule, IcalRecurSkip, civil};

/// The last year expansion looks at, the widest an RFC 5545 date spells.
const MAX_YEAR: i32 = 9999;

/// How many barren periods one occurrence may cost before expansion gives up.
///
/// The year cap alone bounds a rule whose periods are days or longer, but not
/// one whose periods are seconds: `FREQ=SECONDLY;BYSETPOS=2` names a second
/// that no one-element candidate set can hold, and walking to year 9999 a
/// second at a time is a lifetime of work for an answer that is always none.
/// One budget covers both ways a period comes up barren, the empty candidate
/// set and the failing date gate, so the work behind any one `next` is bounded
/// whatever shape the rule takes. No satisfiable rule comes near it: the gates
/// are skipped over rather than stepped through (see
/// [`IcalRecurExpand::seek`]), and this many skips is a millennium of them.
const EMPTY_PERIODS: u32 = 1_000_000;

/// Seconds in a day.
const DAY: i64 = 86_400;

/// The period a `BYDAY` ordinal counts occurrences inside.
///
/// RFC 5545 3.3.10 hangs this on the frequency alone, narrowed to the month
/// when `BYMONTH` picks the months of a yearly period (see [`ordinals`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IcalRecurOrdinals {
    /// The ordinal is meaningless here and the entry counts as unqualified.
    Ignored,
    /// The ordinal counts inside the month, so `1FR` is its first Friday.
    Month,
    /// The ordinal counts inside the year, so `20MO` is its 20th Monday.
    Year,
}

/// The occurrences of a recurrence rule, in chronological order.
///
/// A lazy iterator: it holds one period's candidates at a time and computes
/// the next period only when asked, so a rule with neither `UNTIL` nor
/// `COUNT` is endless yet costs nothing to hold.
///
/// Items are civil, exactly as the module explains: a zoned start recurs at
/// the same wall-clock time, and turning an occurrence into an instant is the
/// caller's step. Bounding the walk to a window is `skip_while` for its start
/// and `take_while` for its end.
///
/// ```
/// use ical::recur::expand::IcalRecurExpand;
/// use ical::recur::{IcalRecurDateTime, IcalRecurRule};
///
/// let rule = IcalRecurRule::parse("FREQ=MONTHLY;BYDAY=-1FR").unwrap();
/// let start = IcalRecurDateTime::date(2026, 1, 30);
/// let dates: Vec<_> = IcalRecurExpand::new(rule, start).take(3).collect();
///
/// assert_eq!(dates, [
///     IcalRecurDateTime::date(2026, 1, 30),
///     IcalRecurDateTime::date(2026, 2, 27),
///     IcalRecurDateTime::date(2026, 3, 27),
/// ]);
/// ```
#[derive(Clone, Debug)]
pub struct IcalRecurExpand {
    rule: IcalRecurRule,
    start: IcalRecurDateTime,
    start_weekday: u8,
    cursor: IcalRecurDateTime,
    ordinals: IcalRecurOrdinals,
    hours: Vec<u8>,
    minutes: Vec<u8>,
    seconds: Vec<u8>,
    buffer: Vec<IcalRecurDateTime>,
    index: usize,
    emitted: u32,
    done: bool,
}

impl IcalRecurExpand {
    /// Starts an expansion of a rule from a start date-time.
    ///
    /// The start plays the part `DTSTART` plays in RFC 5545: it opens the
    /// walk, and it supplies the fields no `BY` part pins down, such as the
    /// time of day of a daily rule or the day of the month of a yearly one.
    ///
    /// An `RSCALE` other than `GREGORIAN` yields nothing: expanding another
    /// calendar system needs the CLDR arithmetic RFC 7529 5 points at, and
    /// yielding Gregorian dates under a Hebrew rule would be a wrong answer
    /// rather than a missing one. `RSCALE=GREGORIAN` is expanded, `SKIP` and
    /// all (RFC 7529 4.1).
    pub fn new(rule: IcalRecurRule, start: IcalRecurDateTime) -> Self {
        let alien = rule
            .scale
            .as_deref()
            .is_some_and(|scale| scale != "GREGORIAN");
        Self {
            start_weekday: weekday_of(day_of(start)),
            cursor: period_start(&rule, start),
            ordinals: ordinals(&rule),
            hours: expanded(&rule.by_hour, start.hour),
            minutes: expanded(&rule.by_minute, start.minute),
            seconds: expanded(&rule.by_second, start.second),
            buffer: Vec::new(),
            index: 0,
            emitted: 0,
            done: alien,
            rule,
            start,
        }
    }

    /// Loads the next non-empty period into the buffer.
    ///
    /// Returns whether one was found, a `false` meaning the rule is spent
    /// (bounded out, or given up on).
    fn fill(&mut self) -> bool {
        self.buffer.clear();
        self.index = 0;
        let mut budget = EMPTY_PERIODS;
        while self.buffer.is_empty() {
            if !self.seek(&mut budget) || budget == 0 {
                return false;
            }
            self.collect();
            self.step();
            budget -= 1;
        }
        true
    }

    /// Moves the cursor onto the next period that could hold a candidate.
    ///
    /// Beyond the bounds this checks, the work here is the sub-daily skip: at
    /// `FREQ=SECONDLY` a `BYMONTH=1` rules out eleven months a candidate at a
    /// time, so a failing gate jumps the cursor to the next day, hour, minute
    /// or second the gate could pass at, keeping it on the `INTERVAL` lattice.
    /// A daily or coarser frequency has no such gap to close, one period being
    /// a whole day at the least.
    ///
    /// Every skip spends one unit of the caller's budget, since a gate that
    /// keeps failing walks the calendar just as a run of empty periods does.
    fn seek(&mut self, budget: &mut u32) -> bool {
        loop {
            if self.cursor.year > MAX_YEAR
                || *budget == 0
                || self.rule.until.is_some_and(|until| self.cursor > until)
            {
                return false;
            }
            *budget -= 1;

            let Some(step) = self.step_seconds() else {
                return true;
            };

            let day = day_of(self.cursor);
            let seconds = seconds_of(self.cursor);
            let limits_minute = matches!(
                self.rule.freq,
                IcalRecurFreq::Minutely | IcalRecurFreq::Secondly
            );
            let limits_second = self.rule.freq == IcalRecurFreq::Secondly;

            let target = if !self.day_matches(day) {
                (day + 1) * DAY
            } else if !self.rule.by_hour.is_empty()
                && !self.rule.by_hour.contains(&self.cursor.hour)
            {
                (seconds.div_euclid(3600) + 1) * 3600
            } else if limits_minute
                && !self.rule.by_minute.is_empty()
                && !self.rule.by_minute.contains(&self.cursor.minute)
            {
                (seconds.div_euclid(60) + 1) * 60
            } else if limits_second
                && !self.rule.by_second.is_empty()
                && !self.rule.by_second.contains(&self.cursor.second)
            {
                seconds + 1
            } else {
                return true;
            };

            self.cursor = instant_of(align(seconds, target, step));
        }
    }

    /// Builds the candidates of the period the cursor sits on.
    ///
    /// The buffer comes out sorted and free of duplicates without being made
    /// so: days are walked forward, and the three time lists were sorted and
    /// deduplicated once when the expansion was built.
    fn collect(&mut self) {
        match self.rule.freq {
            IcalRecurFreq::Secondly => self.buffer.push(self.cursor),
            IcalRecurFreq::Minutely => {
                for second in &self.seconds {
                    self.buffer.push(IcalRecurDateTime {
                        second: *second,
                        ..self.cursor
                    });
                }
            }
            IcalRecurFreq::Hourly => {
                for minute in &self.minutes {
                    for second in &self.seconds {
                        self.buffer.push(IcalRecurDateTime {
                            minute: *minute,
                            second: *second,
                            ..self.cursor
                        });
                    }
                }
            }
            _ => {
                let (first, last) = self.period_days();
                let mut days: Vec<i64> = (first..=last)
                    .filter(|day| self.day_matches(*day))
                    .collect();

                days.extend(self.skipped_days());

                for day in days {
                    let date = date_of(day);
                    for hour in &self.hours {
                        for minute in &self.minutes {
                            for second in &self.seconds {
                                self.buffer.push(IcalRecurDateTime {
                                    hour: *hour,
                                    minute: *minute,
                                    second: *second,
                                    ..date
                                });
                            }
                        }
                    }
                }

                // NOTE: A resolved day can land outside the period and beside a
                // day the scan already found, so the buffer has to be put back
                // in order and its duplicates removed (RFC 5545 3.8.5.3).
                if self.rule.skip != IcalRecurSkip::Omit {
                    self.buffer.sort_unstable();
                    self.buffer.dedup();
                }
            }
        }

        self.apply_set_pos();
    }

    /// Keeps only the candidates `BYSETPOS` names, order preserved.
    fn apply_set_pos(&mut self) {
        if self.rule.by_set_pos.is_empty() {
            return;
        }

        let positions = &self.rule.by_set_pos;
        let total = self.buffer.len() as i32;
        let mut index = 0;
        self.buffer.retain(|_| {
            index += 1;
            positions
                .iter()
                .any(|pos| i32::from(*pos) == index || i32::from(*pos) == index - total - 1)
        });
    }

    /// Moves the cursor on by one `INTERVAL` of the frequency.
    fn step(&mut self) {
        self.cursor = match self.rule.freq {
            IcalRecurFreq::Secondly | IcalRecurFreq::Minutely | IcalRecurFreq::Hourly => {
                let step = self.step_seconds().unwrap_or(1);
                instant_of(seconds_of(self.cursor) + step)
            }
            IcalRecurFreq::Daily => date_of(day_of(self.cursor) + i64::from(self.rule.interval)),
            IcalRecurFreq::Weekly => {
                date_of(day_of(self.cursor) + i64::from(self.rule.interval) * 7)
            }
            IcalRecurFreq::Monthly => {
                let month = i64::from(self.cursor.year) * 12 + i64::from(self.cursor.month) - 1
                    + i64::from(self.rule.interval);
                let year = month.div_euclid(12).min(i64::from(MAX_YEAR) + 1);
                IcalRecurDateTime::date(year as i32, month.rem_euclid(12) as u8 + 1, 1)
            }
            IcalRecurFreq::Yearly => {
                let year = i64::from(self.cursor.year) + i64::from(self.rule.interval);
                IcalRecurDateTime::date(year.min(i64::from(MAX_YEAR) + 1) as i32, 1, 1)
            }
        };
    }

    /// The length of one period in seconds, for sub-daily frequencies only.
    fn step_seconds(&self) -> Option<i64> {
        let unit = match self.rule.freq {
            IcalRecurFreq::Secondly => 1,
            IcalRecurFreq::Minutely => 60,
            IcalRecurFreq::Hourly => 3600,
            _ => return None,
        };
        Some(i64::from(self.rule.interval) * unit)
    }

    /// The first and last day of the period the cursor sits on, inclusive.
    ///
    /// A yearly period carrying `BYWEEKNO` reaches a week either side of the
    /// year, since the first and last week of a year both straddle it; the
    /// week-year check in [`Self::day_matches`] is what keeps the strays out.
    fn period_days(&self) -> (i64, i64) {
        let first = day_of(self.cursor);
        match self.rule.freq {
            IcalRecurFreq::Weekly => (first, first + 6),
            IcalRecurFreq::Monthly => {
                let month = civil::days_in_month(self.cursor.year, self.cursor.month);
                (first, first + i64::from(month) - 1)
            }
            IcalRecurFreq::Yearly => {
                let last = first + i64::from(civil::days_in_year(self.cursor.year)) - 1;
                match self.rule.by_week_no.is_empty() {
                    true => (first, last),
                    false => (first - 7, last + 7),
                }
            }
            _ => (first, first),
        }
    }

    /// Whether `BYWEEKNO` is in force: RFC 5545 3.3.10 allows it at `YEARLY`
    /// alone.
    fn week_no_applies(&self) -> bool {
        !self.rule.by_week_no.is_empty() && self.rule.freq == IcalRecurFreq::Yearly
    }

    /// Whether `BYYEARDAY` is in force: RFC 5545 3.3.10 forbids it at `DAILY`,
    /// `WEEKLY` and `MONTHLY`.
    fn year_day_applies(&self) -> bool {
        !self.rule.by_year_day.is_empty()
            && !matches!(
                self.rule.freq,
                IcalRecurFreq::Daily | IcalRecurFreq::Weekly | IcalRecurFreq::Monthly
            )
    }

    /// Whether `BYMONTHDAY` is in force: RFC 5545 3.3.10 forbids it at
    /// `WEEKLY`.
    fn month_day_applies(&self) -> bool {
        !self.rule.by_month_day.is_empty() && self.rule.freq != IcalRecurFreq::Weekly
    }

    /// The days an invalid day of the month resolves to under `SKIP` (RFC 7529
    /// 4.1).
    ///
    /// Only a day the rule actually intends is resolved: each positive
    /// `BYMONTHDAY`, or the day of the month the start supplies when no part
    /// selects a day. A negative `BYMONTHDAY` counts back from the end of the
    /// month and so always names a day that exists, and in the Gregorian scale
    /// a month number always does too, which leaves the day of the month as
    /// the only thing that can be missing.
    ///
    /// A resolved day is not put back through the day-selecting parts. RFC
    /// 7529 4.1 resolves the day *after* `BYMONTHDAY` has chosen it, and the
    /// whole point is to land on a date the rule did not choose.
    fn skipped_days(&self) -> Vec<i64> {
        if self.rule.skip == IcalRecurSkip::Omit {
            return Vec::new();
        }

        let rule = &self.rule;
        let selects_a_day = !rule.by_day.is_empty()
            || self.month_day_applies()
            || self.year_day_applies()
            || self.week_no_applies();

        let intended: Vec<u8> = match self.month_day_applies() {
            true => rule
                .by_month_day
                .iter()
                .filter(|day| **day > 0)
                .map(|day| *day as u8)
                .collect(),
            // NOTE: With no part selecting a day, the start supplies it, which
            // is the case the RFC's own leap-day example is written for.
            false if !selects_a_day => vec![self.start.day],
            false => return Vec::new(),
        };

        let months: Vec<u8> = match rule.freq {
            IcalRecurFreq::Monthly => vec![self.cursor.month],
            IcalRecurFreq::Yearly if !rule.by_month.is_empty() => rule.by_month.clone(),
            IcalRecurFreq::Yearly if self.month_day_applies() => (1..=12).collect(),
            IcalRecurFreq::Yearly => vec![self.start.month],
            _ => return Vec::new(),
        };

        let year = self.cursor.year;
        let mut days = Vec::new();

        for month in months {
            let length = civil::days_in_month(year, month);

            for _ in intended.iter().filter(|date| **date > length) {
                let last = day_of(IcalRecurDateTime::date(year, month, length));

                days.push(match rule.skip {
                    IcalRecurSkip::Backward => last,
                    // NOTE: Forward from the last day of a month is the first of
                    // the next one, which is where the RFC's Feb 29 lands.
                    _ => last + 1,
                });
            }
        }

        days
    }

    /// Whether the day-selecting parts admit a day, given as a day number.
    fn day_matches(&self, day: i64) -> bool {
        let (year, month, date) = civil::civil_from_days(day);
        let rule = &self.rule;

        if !rule.by_month.is_empty() && !rule.by_month.contains(&month) {
            return false;
        }

        if self.week_no_applies() {
            let start = rule.week_start as u8;
            let (week_year, week) = civil::week_number(year, month, date, start);
            if week_year != self.cursor.year {
                return false;
            }
            let total = i16::from(civil::weeks_in_year(week_year, start));
            let week = i16::from(week);
            let matched = rule
                .by_week_no
                .iter()
                .any(|no| i16::from(*no) == week || i16::from(*no) == week - total - 1);
            if !matched {
                return false;
            }
        }

        if self.year_day_applies() {
            let ordinal = civil::day_of_year(year, month, date) as i16;
            let total = civil::days_in_year(year) as i16;
            let matched = rule
                .by_year_day
                .iter()
                .any(|d| *d == ordinal || *d == ordinal - total - 1);
            if !matched {
                return false;
            }
        }

        if self.month_day_applies() {
            let ordinal = i16::from(date);
            let total = i16::from(civil::days_in_month(year, month));
            let matched = rule
                .by_month_day
                .iter()
                .any(|d| i16::from(*d) == ordinal || i16::from(*d) == ordinal - total - 1);
            if !matched {
                return false;
            }
        }

        if !rule.by_day.is_empty() && !self.weekday_matches(day, year, month, date) {
            return false;
        }

        self.default_day_matches(day, month, date)
    }

    /// Whether a `BYDAY` entry admits a day, ordinals included.
    fn weekday_matches(&self, day: i64, year: i32, month: u8, date: u8) -> bool {
        let weekday = weekday_of(day);
        self.rule.by_day.iter().any(|entry| {
            if entry.weekday as u8 != weekday {
                return false;
            }

            let ordinal = entry
                .ordinal
                .filter(|_| self.ordinals != IcalRecurOrdinals::Ignored);
            let Some(ordinal) = ordinal else {
                return true;
            };

            let (position, total) = match self.ordinals {
                IcalRecurOrdinals::Month => (
                    i16::from(date),
                    i16::from(civil::days_in_month(year, month)),
                ),
                _ => (
                    civil::day_of_year(year, month, date) as i16,
                    civil::days_in_year(year) as i16,
                ),
            };
            let forward = (position - 1) / 7 + 1;
            let count = (total - position) / 7 + forward;
            ordinal == forward || ordinal == forward - count - 1
        })
    }

    /// Whether the fields the start supplies admit a day.
    ///
    /// Only consulted when no part selects a day, which is when RFC 5545 has
    /// the frequency read the missing field off `DTSTART`: a monthly rule
    /// keeps its day of the month, a yearly one its month too unless `BYMONTH`
    /// supplies that, and a weekly one its weekday.
    fn default_day_matches(&self, day: i64, month: u8, date: u8) -> bool {
        let rule = &self.rule;
        // NOTE: Only a part that is *in force* counts as selecting a day: one the
        // frequency forbids is ignored whole, so it can neither select a day
        // nor stop the start from supplying one.
        if !rule.by_day.is_empty()
            || self.month_day_applies()
            || self.year_day_applies()
            || self.week_no_applies()
        {
            return true;
        }

        match rule.freq {
            IcalRecurFreq::Weekly => weekday_of(day) == self.start_weekday,
            IcalRecurFreq::Monthly => date == self.start.day,
            IcalRecurFreq::Yearly => {
                date == self.start.day && (!rule.by_month.is_empty() || month == self.start.month)
            }
            _ => true,
        }
    }
}

impl Iterator for IcalRecurExpand {
    type Item = IcalRecurDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.done {
            while self.index < self.buffer.len() {
                let candidate = self.buffer[self.index];
                self.index += 1;

                if candidate < self.start {
                    continue;
                }
                if self.rule.until.is_some_and(|until| candidate > until)
                    || self.rule.count.is_some_and(|count| self.emitted >= count)
                {
                    self.done = true;
                    return None;
                }

                self.emitted += 1;
                return Some(candidate);
            }

            if !self.fill() {
                self.done = true;
            }
        }

        None
    }
}

/// The start of the period a date-time falls in, at a given frequency.
fn period_start(rule: &IcalRecurRule, start: IcalRecurDateTime) -> IcalRecurDateTime {
    match rule.freq {
        IcalRecurFreq::Secondly | IcalRecurFreq::Minutely | IcalRecurFreq::Hourly => start,
        IcalRecurFreq::Daily => IcalRecurDateTime::date(start.year, start.month, start.day),
        IcalRecurFreq::Weekly => {
            let day = day_of(start);
            let back = (weekday_of(day) + 7 - rule.week_start as u8) % 7;
            date_of(day - i64::from(back))
        }
        IcalRecurFreq::Monthly => IcalRecurDateTime::date(start.year, start.month, 1),
        IcalRecurFreq::Yearly => IcalRecurDateTime::date(start.year, 1, 1),
    }
}

/// The period a `BYDAY` ordinal counts inside.
///
/// RFC 5545 3.3.10 admits an ordinal at `MONTHLY` and `YEARLY` alone, and
/// scopes it to the period the frequency names, narrowed to the month when
/// `BYMONTH` picks the months of a yearly period. The other `BY` parts do not
/// change that scope: an ordinal keeps counting even where the table has
/// `BYDAY` limit rather than expand, which is what every other implementation
/// does (`BYMONTHDAY=15;BYDAY=2MO` is the 15th when it is also the second
/// Monday, not the 15th when it is any Monday).
fn ordinals(rule: &IcalRecurRule) -> IcalRecurOrdinals {
    match rule.freq {
        IcalRecurFreq::Monthly => IcalRecurOrdinals::Month,
        IcalRecurFreq::Yearly => match rule.by_month.is_empty() {
            true => IcalRecurOrdinals::Year,
            false => IcalRecurOrdinals::Month,
        },
        _ => IcalRecurOrdinals::Ignored,
    }
}

/// The values a time part expands to, or the one the start supplies.
fn expanded(part: &[u8], default: u8) -> Vec<u8> {
    if part.is_empty() {
        return vec![default];
    }
    let mut values = part.to_vec();
    values.sort_unstable();
    values.dedup();
    values
}

/// The day number of a date, 1970-01-01 being zero.
fn day_of(date: IcalRecurDateTime) -> i64 {
    civil::days_from_civil(date.year, date.month, date.day)
}

/// The date a day number names, at midnight.
fn date_of(day: i64) -> IcalRecurDateTime {
    let (year, month, date) = civil::civil_from_days(day);
    IcalRecurDateTime::date(year, month, date)
}

/// The weekday of a day number, Sunday being zero.
fn weekday_of(day: i64) -> u8 {
    (day + 4).rem_euclid(7) as u8
}

/// The second a date-time names, 1970-01-01T00:00:00 being zero.
fn seconds_of(date: IcalRecurDateTime) -> i64 {
    day_of(date) * DAY
        + i64::from(date.hour) * 3600
        + i64::from(date.minute) * 60
        + i64::from(date.second)
}

/// The date-time a second names.
fn instant_of(seconds: i64) -> IcalRecurDateTime {
    let rest = seconds.rem_euclid(DAY);
    IcalRecurDateTime {
        hour: (rest / 3600) as u8,
        minute: (rest / 60 % 60) as u8,
        second: (rest % 60) as u8,
        ..date_of(seconds.div_euclid(DAY))
    }
}

/// The first point of a lattice at or after a target.
///
/// The lattice is the one `from` sits on, stepping by `step`, which is what
/// keeps a skipped-over gap from knocking an expansion off its `INTERVAL`.
fn align(from: i64, target: i64, step: i64) -> i64 {
    from + (target - from + step - 1) / step * step
}

#[cfg(test)]
mod tests {
    use crate::recur::expand::*;

    fn expand(rule: &str, start: IcalRecurDateTime, take: usize) -> Vec<IcalRecurDateTime> {
        let rule = IcalRecurRule::parse(rule).unwrap();
        IcalRecurExpand::new(rule, start).take(take).collect()
    }

    fn dates(days: &[(i32, u8, u8)]) -> Vec<IcalRecurDateTime> {
        days.iter()
            .map(|(y, m, d)| IcalRecurDateTime::date(*y, *m, *d))
            .collect()
    }

    #[test]
    fn walks_days_weeks_months_and_years() {
        let start = IcalRecurDateTime::date(1997, 9, 2);
        assert_eq!(
            expand("FREQ=DAILY;INTERVAL=10", start, 5),
            dates(&[
                (1997, 9, 2),
                (1997, 9, 12),
                (1997, 9, 22),
                (1997, 10, 2),
                (1997, 10, 12)
            ])
        );
        assert_eq!(
            expand("FREQ=WEEKLY", start, 3),
            dates(&[(1997, 9, 2), (1997, 9, 9), (1997, 9, 16)])
        );
        assert_eq!(
            expand("FREQ=MONTHLY", start, 3),
            dates(&[(1997, 9, 2), (1997, 10, 2), (1997, 11, 2)])
        );
        assert_eq!(
            expand("FREQ=YEARLY", start, 3),
            dates(&[(1997, 9, 2), (1998, 9, 2), (1999, 9, 2)])
        );
    }

    #[test]
    fn reads_missing_fields_off_the_start() {
        let start = IcalRecurDateTime::date(1997, 3, 10);
        assert_eq!(
            expand("FREQ=YEARLY;BYMONTH=1,3", start, 4),
            dates(&[(1997, 3, 10), (1998, 1, 10), (1998, 3, 10), (1999, 1, 10)])
        );
    }

    #[test]
    fn skips_periods_the_start_day_is_missing_from() {
        let start = IcalRecurDateTime::date(2026, 1, 31);
        assert_eq!(
            expand("FREQ=MONTHLY", start, 3),
            dates(&[(2026, 1, 31), (2026, 3, 31), (2026, 5, 31)])
        );

        let leap = IcalRecurDateTime::date(2024, 2, 29);
        assert_eq!(
            expand("FREQ=YEARLY", leap, 2),
            dates(&[(2024, 2, 29), (2028, 2, 29)])
        );
    }

    #[test]
    fn counts_weekday_ordinals_inside_their_period() {
        let start = IcalRecurDateTime::date(1997, 9, 5);
        assert_eq!(
            expand("FREQ=MONTHLY;BYDAY=1FR", start, 3),
            dates(&[(1997, 9, 5), (1997, 10, 3), (1997, 11, 7)])
        );
        assert_eq!(
            expand("FREQ=MONTHLY;BYDAY=-1MO", start, 2),
            dates(&[(1997, 9, 29), (1997, 10, 27)])
        );

        let monday = IcalRecurDateTime::date(1997, 5, 19);
        assert_eq!(
            expand("FREQ=YEARLY;BYDAY=20MO", monday, 2),
            dates(&[(1997, 5, 19), (1998, 5, 18)])
        );
    }

    #[test]
    fn honours_the_week_start_of_a_weekly_interval() {
        let start = IcalRecurDateTime::date(1997, 8, 5);
        assert_eq!(
            expand("FREQ=WEEKLY;INTERVAL=2;BYDAY=TU,SU;WKST=MO", start, 4),
            dates(&[(1997, 8, 5), (1997, 8, 10), (1997, 8, 19), (1997, 8, 24)])
        );
        assert_eq!(
            expand("FREQ=WEEKLY;INTERVAL=2;BYDAY=TU,SU;WKST=SU", start, 4),
            dates(&[(1997, 8, 5), (1997, 8, 17), (1997, 8, 19), (1997, 8, 31)])
        );
    }

    #[test]
    fn picks_positions_out_of_a_finished_period() {
        let start = IcalRecurDateTime::date(1997, 9, 4);
        assert_eq!(
            expand("FREQ=MONTHLY;BYDAY=TU,WE,TH;BYSETPOS=3", start, 3),
            dates(&[(1997, 9, 4), (1997, 10, 7), (1997, 11, 6)])
        );
        assert_eq!(
            expand("FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-2", start, 2),
            dates(&[(1997, 9, 29), (1997, 10, 30)])
        );
    }

    #[test]
    fn crosses_the_time_parts_with_the_days() {
        let start = IcalRecurDateTime {
            year: 2026,
            month: 1,
            day: 1,
            hour: 9,
            minute: 0,
            second: 0,
        };
        let times = expand("FREQ=DAILY;BYHOUR=9,17;BYMINUTE=0,30", start, 5);
        let expected = [(9, 0), (9, 30), (17, 0), (17, 30)];
        assert_eq!(times.len(), 5);
        for (occurrence, (hour, minute)) in times.iter().zip(expected) {
            assert_eq!((occurrence.hour, occurrence.minute), (hour, minute));
            assert_eq!(occurrence.day, 1);
        }
        assert_eq!((times[4].day, times[4].hour), (2, 9));
    }

    #[test]
    fn skips_whole_days_a_sub_daily_rule_cannot_land_on() {
        let start = IcalRecurDateTime {
            year: 2026,
            month: 1,
            day: 1,
            hour: 9,
            minute: 0,
            second: 0,
        };
        let hours = expand("FREQ=HOURLY;INTERVAL=6;BYMONTH=1,3", start, 4);
        assert_eq!(hours.len(), 4);

        let crossing = expand("FREQ=HOURLY;INTERVAL=6;BYMONTH=1", start, 200);
        let last = crossing.last().unwrap();
        assert_eq!((last.year, last.month), (2027, 1));
    }

    #[test]
    fn bounds_the_walk_by_count_and_until() {
        let start = IcalRecurDateTime::date(1997, 9, 2);
        assert_eq!(expand("FREQ=DAILY;COUNT=3", start, 10).len(), 3);
        assert_eq!(
            expand("FREQ=DAILY;UNTIL=19970904", start, 10),
            dates(&[(1997, 9, 2), (1997, 9, 3), (1997, 9, 4)])
        );
        assert!(expand("FREQ=DAILY;COUNT=0", start, 10).is_empty());
    }

    #[test]
    fn drops_a_start_the_rule_does_not_generate() {
        let monday = IcalRecurDateTime::date(2026, 1, 5);
        assert_eq!(
            expand("FREQ=WEEKLY;BYDAY=TU", monday, 2),
            dates(&[(2026, 1, 6), (2026, 1, 13)])
        );
    }

    #[test]
    fn ends_on_an_unsatisfiable_rule() {
        let start = IcalRecurDateTime::date(2026, 1, 1);
        assert!(expand("FREQ=YEARLY;BYMONTH=2;BYMONTHDAY=30", start, 1).is_empty());
        assert!(expand("FREQ=MONTHLY;BYMONTHDAY=31;BYMONTH=2", start, 1).is_empty());
        assert!(expand("FREQ=DAILY;RSCALE=CHINESE", start, 1).is_empty());
    }

    #[test]
    fn selects_weeks_that_straddle_the_year() {
        let start = IcalRecurDateTime::date(2019, 1, 1);
        assert_eq!(
            expand("FREQ=YEARLY;BYWEEKNO=1;BYDAY=MO", start, 2),
            dates(&[(2019, 12, 30), (2021, 1, 4)])
        );
    }

    #[test]
    fn counts_backwards_from_the_end_of_a_period() {
        let start = IcalRecurDateTime::date(2026, 1, 1);
        assert_eq!(
            expand("FREQ=MONTHLY;BYMONTHDAY=-1", start, 4),
            dates(&[(2026, 1, 31), (2026, 2, 28), (2026, 3, 31), (2026, 4, 30)])
        );
        assert_eq!(
            expand("FREQ=YEARLY;BYYEARDAY=-1", start, 2),
            dates(&[(2026, 12, 31), (2027, 12, 31)])
        );
        assert_eq!(
            expand("FREQ=YEARLY;BYWEEKNO=-1;BYDAY=MO", start, 3),
            dates(&[(2026, 12, 28), (2027, 12, 27), (2028, 12, 25)])
        );
    }

    #[test]
    fn yields_nothing_rather_than_panicking_on_a_nonsense_start() {
        let start = IcalRecurDateTime {
            year: 2026,
            month: 0,
            day: 0,
            hour: 99,
            minute: 99,
            second: 99,
        };
        for freq in [
            "SECONDLY", "MINUTELY", "HOURLY", "DAILY", "WEEKLY", "MONTHLY", "YEARLY",
        ] {
            let rule = alloc::format!("FREQ={freq};COUNT=2");
            IcalRecurExpand::new(IcalRecurRule::parse(&rule).unwrap(), start)
                .take(2)
                .for_each(drop);
        }
    }

    #[test]
    fn counts_weekday_ordinals_even_where_byday_only_limits() {
        // NOTE: RFC 5545 3.3.10 admits an ordinal at MONTHLY and YEARLY whatever the
        // other BY parts hold, so the 15th qualifies only when it is also the
        // second (or second to last) Monday of its month.
        let start = IcalRecurDateTime::date(1997, 9, 2);
        assert_eq!(
            expand("FREQ=MONTHLY;BYMONTHDAY=15;BYDAY=2MO,-2MO", start, 3),
            dates(&[(1999, 2, 15), (2010, 2, 15), (2021, 2, 15)])
        );

        // NOTE: At YEARLY the ordinal counts inside the year, so this is January 1
        // in the years it opens on a Friday.
        assert_eq!(
            expand("FREQ=YEARLY;BYMONTHDAY=1;BYDAY=1FR", start, 3),
            dates(&[(1999, 1, 1), (2010, 1, 1), (2016, 1, 1)])
        );

        // NOTE: ... narrowed to the month when BYMONTH picks the months.
        assert_eq!(
            expand("FREQ=YEARLY;BYMONTH=11;BYDAY=1TU", start, 2),
            dates(&[(1997, 11, 4), (1998, 11, 3)])
        );
    }

    #[test]
    fn gives_up_on_a_period_set_pos_can_never_fill() {
        // NOTE: One candidate per period, and BYSETPOS asks for the second: without
        // a bound on empty periods this walks a century of seconds.
        let start = IcalRecurDateTime::date(2026, 1, 1);
        assert!(expand("FREQ=SECONDLY;BYSETPOS=2", start, 1).is_empty());
        assert!(expand("FREQ=MINUTELY;BYSECOND=0;BYSETPOS=3", start, 1).is_empty());
    }

    #[test]
    fn picks_several_positions_at_once() {
        let start = IcalRecurDateTime::date(2026, 1, 1);
        assert_eq!(
            expand("FREQ=MONTHLY;BYDAY=MO,WE,FR;BYSETPOS=1,-1", start, 4),
            dates(&[(2026, 1, 2), (2026, 1, 30), (2026, 2, 2), (2026, 2, 27)])
        );
    }
}
