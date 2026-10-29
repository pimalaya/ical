//! # Calendar scales
//!
//! The calendar a rule counts its years, months and days in (RFC 7529
//! `RSCALE`).
//!
//! Expansion walks day numbers, 1970-01-01 being zero, whatever the scale: a
//! weekday and a time of day are the same in every calendar, so only the
//! fields a `BY` part reads within a year or a month ask the scale. The
//! Gregorian scale answers through the crate's own civil arithmetic. Behind
//! the `rscale` feature every other scale answers through `icu_calendar`,
//! the CLDR arithmetic RFC 7529 5 points at, and a day number goes back out
//! as a Gregorian date, so nothing past the rule ever sees another calendar.
//!
//! A month is addressed two ways. Its ordinal is its place in its year, 1 to
//! 13, which is what a walk steps through; its [`IcalRecurMonth`] is what a
//! rule names, the same regular month whether or not a leap month precedes
//! it in that year.

use crate::recur::{IcalRecurMonth, civil};

/// The first Gregorian year past what expansion looks at, 9999 being the
/// last an RFC 5545 date spells. Another scale's years run higher (the
/// Hebrew year 9999 is Gregorian 6239), so its cap is twice this, in its own
/// years, a bound on arithmetic rather than on what a rule yields.
const YEAR_CAP: i32 = 10_000;

/// The calendar a rule's years, months and days are counted in.
#[derive(Clone, Debug)]
pub(crate) enum IcalRecurScale {
    /// The proleptic Gregorian calendar, RFC 5545's own.
    Gregorian,
    /// Any other calendar `icu_calendar` carries an algorithm for.
    #[cfg(feature = "rscale")]
    Icu(icu_calendar::AnyCalendar),
}

impl IcalRecurScale {
    /// The scale an `RSCALE` value names, uppercased as the rule holds it.
    ///
    /// `None` for a scale this build cannot count in: every one but
    /// `GREGORIAN` without the `rscale` feature, and the observational
    /// Islamic calendars (`ISLAMIC`, `ISLAMIC-RGSA`) with it, which no
    /// algorithm predicts.
    pub(crate) fn of(name: Option<&str>) -> Option<Self> {
        match name {
            None | Some("GREGORIAN") => Some(Self::Gregorian),
            #[cfg(feature = "rscale")]
            Some(name) => icu::calendar(name).map(Self::Icu),
            #[cfg(not(feature = "rscale"))]
            Some(_) => None,
        }
    }

    /// The year, the month ordinal and the day of the month a day number
    /// falls on.
    pub(crate) fn date(&self, day: i64) -> (i32, u8, u8) {
        match self {
            Self::Gregorian => civil::civil_from_days(day),
            #[cfg(feature = "rscale")]
            Self::Icu(calendar) => icu::date(calendar, day),
        }
    }

    /// The day number of the first day of a month, given by its ordinal.
    ///
    /// A year past [`YEAR_CAP`] reads as that one, which the walk's own cap
    /// then ends, so no arithmetic runs on a year nobody can write.
    pub(crate) fn first(&self, year: i32, ordinal: u8) -> i64 {
        match self {
            Self::Gregorian => civil::days_from_civil(year.min(YEAR_CAP), ordinal, 1),
            #[cfg(feature = "rscale")]
            Self::Icu(calendar) => icu::first(calendar, year, ordinal),
        }
    }

    /// How many days a month has, given by its ordinal.
    pub(crate) fn days_in_month(&self, year: i32, ordinal: u8) -> u8 {
        match self {
            Self::Gregorian => civil::days_in_month(year, ordinal),
            #[cfg(feature = "rscale")]
            Self::Icu(_) => {
                let next = match ordinal < self.months_in_year(year) {
                    true => self.first(year, ordinal + 1),
                    false => self.first(year.saturating_add(1), 1),
                };

                (next - self.first(year, ordinal)) as u8
            }
        }
    }

    /// How many months a year has, leap months included.
    pub(crate) fn months_in_year(&self, year: i32) -> u8 {
        match self {
            Self::Gregorian => 12,
            #[cfg(feature = "rscale")]
            Self::Icu(calendar) => icu::months_in_year(calendar, year),
        }
    }

    /// How many days a year has.
    pub(crate) fn days_in_year(&self, year: i32) -> u16 {
        match self {
            Self::Gregorian => civil::days_in_year(year),
            #[cfg(feature = "rscale")]
            Self::Icu(_) => (self.first(year.saturating_add(1), 1) - self.first(year, 1)) as u16,
        }
    }

    /// The day of the year, 1 for the first.
    pub(crate) fn day_of_year(&self, year: i32, ordinal: u8, date: u8) -> u16 {
        match self {
            Self::Gregorian => civil::day_of_year(year, ordinal, date),
            #[cfg(feature = "rscale")]
            Self::Icu(_) => {
                (self.first(year, ordinal) + i64::from(date) - self.first(year, 1)) as u16
            }
        }
    }

    /// The month a rule names that sits at an ordinal of a year.
    pub(crate) fn month(&self, year: i32, ordinal: u8) -> IcalRecurMonth {
        match self {
            Self::Gregorian => IcalRecurMonth::regular(ordinal),
            #[cfg(feature = "rscale")]
            Self::Icu(calendar) => icu::month(calendar, year, ordinal),
        }
    }

    /// The ordinal a month a rule names sits at in a year, `None` when the
    /// year lacks it: a leap month a common year does not have, or a number
    /// past the scale's last month.
    pub(crate) fn ordinal(&self, year: i32, month: IcalRecurMonth) -> Option<u8> {
        match self {
            Self::Gregorian => (!month.leap && month.number <= 12).then_some(month.number),
            #[cfg(feature = "rscale")]
            Self::Icu(_) => {
                (1..=self.months_in_year(year)).find(|ordinal| self.month(year, *ordinal) == month)
            }
        }
    }

    /// How many months separate two months, each a year and an ordinal,
    /// negative when the second comes first.
    pub(crate) fn months_between(&self, from: (i32, u8), to: (i32, u8)) -> i64 {
        let ordinals = i64::from(to.1) - i64::from(from.1);

        match self {
            Self::Gregorian => (i64::from(to.0) - i64::from(from.0)) * 12 + ordinals,
            #[cfg(feature = "rscale")]
            Self::Icu(_) => {
                let (low, high, sign) = match from.0 <= to.0 {
                    true => (from.0, to.0, 1),
                    false => (to.0, from.0, -1),
                };

                let years: i64 = (low..high)
                    .map(|year| i64::from(self.months_in_year(year)))
                    .sum();

                sign * years + ordinals
            }
        }
    }

    /// The month a number of months after another, each a year and an
    /// ordinal. A year past [`YEAR_CAP`] stops there.
    pub(crate) fn add_months(&self, (year, ordinal): (i32, u8), months: i64) -> (i32, u8) {
        let mut year = year;
        let mut index = i64::from(ordinal) - 1 + months;

        match self {
            Self::Gregorian => {
                let year = (i64::from(year) + index.div_euclid(12)).min(i64::from(YEAR_CAP));
                (year as i32, index.rem_euclid(12) as u8 + 1)
            }
            #[cfg(feature = "rscale")]
            Self::Icu(_) => {
                while index >= i64::from(self.months_in_year(year)) && year < YEAR_CAP * 2 {
                    index -= i64::from(self.months_in_year(year));
                    year += 1;
                }

                while index < 0 && year > -YEAR_CAP * 2 {
                    year -= 1;
                    index += i64::from(self.months_in_year(year));
                }

                (year, index.clamp(0, 12) as u8 + 1)
            }
        }
    }

    /// The week-numbering year and the week a day falls in, counted from one,
    /// as [`civil::week_number`] reads them, the year being the scale's.
    pub(crate) fn week_number(&self, day: i64, week_start: u8) -> (i32, u8) {
        match self {
            Self::Gregorian => {
                let (year, month, date) = civil::civil_from_days(day);
                civil::week_number(year, month, date, week_start)
            }
            #[cfg(feature = "rscale")]
            Self::Icu(_) => {
                let weekday = (day + 4).rem_euclid(7) as u8;
                let offset = (weekday + 7 - week_start % 7) % 7;
                let middle = day + 3 - i64::from(offset);
                let (year, ordinal, date) = self.date(middle);

                (
                    year,
                    ((self.day_of_year(year, ordinal, date) - 1) / 7 + 1) as u8,
                )
            }
        }
    }

    /// The number of weeks in a week-numbering year: the week of the day
    /// four before the next year starts, which always falls in the last.
    pub(crate) fn weeks_in_year(&self, year: i32, week_start: u8) -> u8 {
        match self {
            Self::Gregorian => civil::weeks_in_year(year, week_start),
            #[cfg(feature = "rscale")]
            Self::Icu(_) => {
                let day = self.first(year.saturating_add(1), 1) - 4;
                self.week_number(day, week_start).1
            }
        }
    }
}

/// The calendars `icu_calendar` counts in, read through day numbers.
#[cfg(feature = "rscale")]
mod icu {
    use icu_calendar::{
        AnyCalendar, Date, Gregorian, Iso, Ref,
        cal::{
            Buddhist, ChineseTraditional, Coptic, Ethiopian, EthiopianEraStyle, Hebrew, Hijri,
            Indian, KoreanTraditional, Persian, Roc,
            hijri::{TabularAlgorithmEpoch, TabularAlgorithmLeapYears},
        },
        types::{DateFields, RataDie},
    };

    use crate::recur::{IcalRecurMonth, civil, scale::YEAR_CAP};

    /// The rata die of 1970-01-01, day zero of the crate's day numbers.
    const EPOCH: i64 = 719_163;

    /// The calendar a CLDR name stands for, among those RFC 7529 can name and
    /// an algorithm can compute (RFC 7529 5, the IANA Calendar Systems
    /// registry). `JAPANESE` is Gregorian but for its eras, which a rule
    /// never reads.
    pub(super) fn calendar(name: &str) -> Option<AnyCalendar> {
        let tabular = |epoch| {
            AnyCalendar::HijriTabular(Hijri::new_tabular(TabularAlgorithmLeapYears::TypeII, epoch))
        };

        let calendar = match name {
            "JAPANESE" => AnyCalendar::Gregorian(Gregorian),
            "ISO8601" => AnyCalendar::Iso(Iso),
            "HEBREW" => AnyCalendar::Hebrew(Hebrew),
            "CHINESE" => AnyCalendar::Chinese(ChineseTraditional::new()),
            "DANGI" => AnyCalendar::Dangi(KoreanTraditional::new()),
            "ISLAMIC-CIVIL" => tabular(TabularAlgorithmEpoch::Friday),
            "ISLAMIC-TBLA" => tabular(TabularAlgorithmEpoch::Thursday),
            "ISLAMIC-UMALQURA" => AnyCalendar::HijriUmmAlQura(Hijri::new_umm_al_qura()),
            "COPTIC" => AnyCalendar::Coptic(Coptic),
            "ETHIOPIC" => AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(
                EthiopianEraStyle::AmeteMihret,
            )),
            "ETHIOAA" | "ETHIOPIC-AMETE-ALEM" => {
                AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(EthiopianEraStyle::AmeteAlem))
            }
            "INDIAN" => AnyCalendar::Indian(Indian),
            "PERSIAN" => AnyCalendar::Persian(Persian),
            "BUDDHIST" => AnyCalendar::Buddhist(Buddhist),
            "ROC" => AnyCalendar::Roc(Roc),
            _ => return None,
        };

        Some(calendar)
    }

    /// The date a day number falls on in a calendar.
    fn at(calendar: &AnyCalendar, day: i64) -> Date<Ref<'_, AnyCalendar>> {
        Date::from_rata_die(RataDie::new(day + EPOCH), Ref(calendar))
    }

    /// The first day of a month of a year, given by its ordinal.
    fn first_of(
        calendar: &AnyCalendar,
        year: i32,
        ordinal: u8,
    ) -> Option<Date<Ref<'_, AnyCalendar>>> {
        let mut fields = DateFields::default();
        fields.extended_year = Some(year);
        fields.ordinal_month = Some(ordinal);
        fields.day = Some(1);

        Date::try_from_fields(fields, Default::default(), Ref(calendar)).ok()
    }

    pub(super) fn date(calendar: &AnyCalendar, day: i64) -> (i32, u8, u8) {
        let date = at(calendar, day);
        (
            date.year().extended_year(),
            date.month().ordinal,
            date.day_of_month().0,
        )
    }

    /// The day number of the first day of a month. A year outside the range
    /// the calendar computes reads as a Gregorian year past the cap on that
    /// side, which ends the walk.
    pub(super) fn first(calendar: &AnyCalendar, year: i32, ordinal: u8) -> i64 {
        match first_of(calendar, year, ordinal) {
            Some(date) => date.to_rata_die().to_i64_date() - EPOCH,
            None => match year < 0 {
                true => civil::days_from_civil(-YEAR_CAP, 1, 1),
                false => civil::days_from_civil(YEAR_CAP, 1, 1),
            },
        }
    }

    pub(super) fn months_in_year(calendar: &AnyCalendar, year: i32) -> u8 {
        first_of(calendar, year, 1).map_or(12, |date| date.months_in_year())
    }

    pub(super) fn month(calendar: &AnyCalendar, year: i32, ordinal: u8) -> IcalRecurMonth {
        match first_of(calendar, year, ordinal) {
            Some(date) => {
                let month = date.month().to_input();
                IcalRecurMonth {
                    number: month.number(),
                    leap: month.is_leap(),
                }
            }
            None => IcalRecurMonth::regular(0),
        }
    }
}
