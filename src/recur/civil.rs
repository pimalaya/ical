//! # Civil calendar arithmetic
//!
//! Proleptic Gregorian date arithmetic for recurrence expansion: day counts,
//! weekdays, days of the year and week numbers, integers throughout.
//!
//! The two day-count conversions are Howard Hinnant's, from his
//! [date algorithms](https://howardhinnant.github.io/date_algorithms.html),
//! with 1970-01-01 as day zero. They are exact over the whole `i32` year
//! range.
//!
//! Everything else here is derived from them, so the leap-year and era
//! arithmetic is written once.
//!
//! Week numbering is the ISO 8601 rule generalised to an arbitrary first day
//! of the week, which is what `BYWEEKNO` needs: RFC 5545 3.3.10 starts a week
//! on `WKST`, while the ISO definition of week one (the first week holding at
//! least four days of the year) carries over to any week start unchanged.

/// Whether the year is a leap year in the proleptic Gregorian calendar.
pub(crate) const fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// The length of the month, zero for a month outside 1 to 12.
///
/// The zero is what makes an out-of-range month fail a day range check rather
/// than quietly pass one.
pub(crate) const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// The length of the year, 365 days or 366.
pub(crate) const fn days_in_year(year: i32) -> u16 {
    if is_leap(year) { 366 } else { 365 }
}

/// The days from 1970-01-01 to the date, negative before the epoch.
///
/// Hinnant's constants. An era is the 400 years the calendar repeats
/// over (146097 days), 719468 its 0000-03-01 start to the epoch. Starting the
/// year in March puts the leap day last, so 153/5 needs no leap-year term.
pub(crate) const fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let month = month as i64;
    let year = year as i64 - (month <= 2) as i64;
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let march_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * march_month + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146097 + day_of_era - 719468
}

/// The date a day count denotes, the inverse of [`days_from_civil`].
pub(crate) const fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let days = days + 719468;
    let era = days.div_euclid(146097);
    let day_of_era = days.rem_euclid(146097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let march_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * march_month + 2) / 5 + 1;
    let month = march_month + if march_month < 10 { 3 } else { -9 };
    let year = year_of_era + era * 400 + (month <= 2) as i64;

    (year as i32, month as u8, day as u8)
}

/// The day of the week, Sunday being zero through Saturday six.
///
/// The numbering matches [`IcalRecurWeekday`](crate::recur::IcalRecurWeekday),
/// so a weekday is a cast and never a lookup.
///
/// Day zero, 1970-01-01, was a Thursday, hence the shift by four.
pub(crate) const fn weekday(year: i32, month: u8, day: u8) -> u8 {
    (days_from_civil(year, month, day) + 4).rem_euclid(7) as u8
}

/// The day of the year, 1 to 365 or 366.
pub(crate) const fn day_of_year(year: i32, month: u8, day: u8) -> u16 {
    (days_from_civil(year, month, day) - days_from_civil(year, 1, 1) + 1) as u16
}

/// The week-numbering year and the week the date falls in, counted from one.
///
/// The ISO 8601 rule generalised to any `WKST`, which is what RFC 5545 reads
/// `BYWEEKNO` against: a week starts on `week_start` (Sunday zero, as in
/// [`weekday`]) and week one is the first holding four days of the year, so
/// the year returned can differ from the calendar one around January 1.
///
/// A week belongs to the year of its fourth day, holding that day and
/// holding four days of a year being one condition; for week one that day
/// falls in the first seven of January, making the week its day of the year
/// divided by seven.
pub(crate) const fn week_number(year: i32, month: u8, day: u8, week_start: u8) -> (i32, u8) {
    let offset = (weekday(year, month, day) + 7 - week_start % 7) % 7;
    let middle = days_from_civil(year, month, day) + 3 - offset as i64;
    let (year, month, day) = civil_from_days(middle);

    (year, ((day_of_year(year, month, day) - 1) / 7 + 1) as u8)
}

/// The number of weeks in the week-numbering year, 52 or 53.
///
/// December 28 falls in the last week whatever the week start, the mirror of
/// January 4 always falling in the first.
pub(crate) const fn weeks_in_year(year: i32, week_start: u8) -> u8 {
    week_number(year, 12, 28, week_start).1
}

#[cfg(test)]
mod tests {
    use crate::recur::civil::*;

    const SUNDAY: u8 = 0;
    const MONDAY: u8 = 1;

    #[test]
    fn leap_years_follow_the_century_rule() {
        assert!(is_leap(2024));
        assert!(!is_leap(2023));
        assert!(is_leap(2000));
        assert!(!is_leap(1900));
        assert!(!is_leap(2100));
        assert!(is_leap(1600));
        assert!(is_leap(0));
        assert!(is_leap(-4));
        assert!(!is_leap(-1));
        assert!(!is_leap(-100));
        assert!(is_leap(-400));

        assert_eq!(days_in_year(2024), 366);
        assert_eq!(days_in_year(1900), 365);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(2024, 1), 31);
        assert_eq!(days_in_month(2024, 4), 30);
        assert_eq!(days_in_month(2024, 12), 31);
        assert_eq!(days_in_month(2024, 0), 0);
        assert_eq!(days_in_month(2024, 13), 0);
    }

    #[test]
    fn day_counts_anchor_on_the_epoch() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(1970, 1, 2), 1);
        assert_eq!(days_from_civil(1969, 12, 31), -1);
        assert_eq!(days_from_civil(2000, 1, 1), 10957);
        assert_eq!(days_from_civil(1, 1, 1), -719162);
        assert_eq!(days_from_civil(0, 1, 1), -719528);

        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(1), (1970, 1, 2));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(civil_from_days(10957), (2000, 1, 1));
        assert_eq!(civil_from_days(-719162), (1, 1, 1));
        assert_eq!(civil_from_days(-719528), (0, 1, 1));
    }

    #[test]
    fn day_counts_round_trip() {
        for days in -800_000..=800_000 {
            let (year, month, day) = civil_from_days(days);
            assert!((1..=12).contains(&month), "{days}");
            assert!((1..=days_in_month(year, month)).contains(&day), "{days}");
            assert_eq!(days_from_civil(year, month, day), days);
        }

        for year in [i32::MIN, -2_000_000_000, -400, -1, 0, 1, i32::MAX] {
            for (month, day) in [(1, 1), (2, 28), (7, 15), (12, 31)] {
                let days = days_from_civil(year, month, day);
                assert_eq!(civil_from_days(days), (year, month, day));
            }
        }
    }

    #[test]
    fn weekdays_match_the_known_ones() {
        assert_eq!(weekday(1970, 1, 1), 4);
        assert_eq!(weekday(2026, 8, 8), 6);
        assert_eq!(weekday(2026, 1, 1), 4);
        assert_eq!(weekday(2000, 1, 1), 6);
        assert_eq!(weekday(1900, 1, 1), 1);
        assert_eq!(weekday(1, 1, 1), 1);
        assert_eq!(weekday(0, 1, 1), 6);
        assert_eq!(weekday(-1, 1, 1), 5);

        for days in -400_000..=400_000 {
            let (year, month, day) = civil_from_days(days);
            assert_eq!(weekday(year, month, day) as i64, (days + 4).rem_euclid(7));
        }
    }

    #[test]
    fn days_of_the_year_round_trip() {
        for year in [2023, 2024, 1900, 2000, -1, 0] {
            let january = days_from_civil(year, 1, 1);
            for number in 1..=days_in_year(year) {
                let (walked, month, day) = civil_from_days(january + number as i64 - 1);
                assert_eq!(walked, year);
                assert!((1..=days_in_month(year, month)).contains(&day));
                assert_eq!(day_of_year(year, month, day), number);
            }
        }

        assert_eq!(day_of_year(2024, 2, 29), 60);
        assert_eq!(day_of_year(2024, 3, 1), 61);
        assert_eq!(day_of_year(2023, 3, 1), 60);
        assert_eq!(day_of_year(2024, 12, 31), 366);
        assert_eq!(day_of_year(2023, 12, 31), 365);
    }

    #[test]
    fn weeks_number_from_the_first_four_day_week() {
        assert_eq!(week_number(2026, 1, 1, MONDAY), (2026, 1));
        assert_eq!(week_number(2026, 1, 4, MONDAY), (2026, 1));
        assert_eq!(week_number(2026, 1, 5, MONDAY), (2026, 2));
        assert_eq!(week_number(2026, 12, 31, MONDAY), (2026, 53));
        assert_eq!(week_number(2027, 1, 1, MONDAY), (2026, 53));
        assert_eq!(week_number(2027, 1, 4, MONDAY), (2027, 1));

        assert_eq!(week_number(2020, 12, 31, MONDAY), (2020, 53));
        assert_eq!(week_number(2021, 1, 1, MONDAY), (2020, 53));
        assert_eq!(week_number(2021, 1, 4, MONDAY), (2021, 1));
        assert_eq!(week_number(2016, 1, 1, MONDAY), (2015, 53));
    }

    #[test]
    fn weeks_follow_the_week_start() {
        assert_eq!(week_number(2026, 1, 1, SUNDAY), (2025, 53));
        assert_eq!(week_number(2026, 1, 3, SUNDAY), (2025, 53));
        assert_eq!(week_number(2026, 1, 4, SUNDAY), (2026, 1));
        assert_eq!(week_number(2026, 1, 5, SUNDAY), (2026, 1));
        assert_eq!(week_number(2026, 1, 11, SUNDAY), (2026, 2));
        assert_eq!(week_number(2026, 12, 31, SUNDAY), (2026, 52));

        assert_eq!(week_number(2021, 1, 1, SUNDAY), (2020, 53));
        assert_eq!(week_number(2021, 1, 3, SUNDAY), (2021, 1));
    }

    #[test]
    fn years_hold_fifty_two_weeks_or_fifty_three() {
        assert_eq!(weeks_in_year(2026, MONDAY), 53);
        assert_eq!(weeks_in_year(2025, MONDAY), 52);
        assert_eq!(weeks_in_year(2024, MONDAY), 52);
        assert_eq!(weeks_in_year(2020, MONDAY), 53);
        assert_eq!(weeks_in_year(2015, MONDAY), 53);

        assert_eq!(weeks_in_year(2026, SUNDAY), 52);
        assert_eq!(weeks_in_year(2025, SUNDAY), 53);
        assert_eq!(weeks_in_year(2020, SUNDAY), 53);

        for year in 1970..2070 {
            for week_start in 0..7 {
                let weeks = weeks_in_year(year, week_start);
                assert!(weeks == 52 || weeks == 53, "{year} holds {weeks} weeks");
                assert_eq!(week_number(year, 12, 28, week_start), (year, weeks));
                assert_eq!(week_number(year, 1, 4, week_start), (year, 1));
            }
        }
    }

    #[test]
    fn weeks_run_contiguously_across_year_boundaries() {
        for week_start in 0..7 {
            let mut expected = week_number(1980, 1, 1, week_start);
            for days in days_from_civil(1980, 1, 1)..days_from_civil(2040, 1, 1) {
                let (year, month, day) = civil_from_days(days);
                let week = week_number(year, month, day, week_start);
                assert_eq!(week, expected, "{year}-{month}-{day}");

                if weekday(year, month, day) == (week_start + 6) % 7 {
                    expected = if expected.1 == weeks_in_year(expected.0, week_start) {
                        (expected.0 + 1, 1)
                    } else {
                        (expected.0, expected.1 + 1)
                    };
                }
            }
        }
    }
}
