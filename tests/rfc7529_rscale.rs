//! RFC 7529 `RSCALE`: a rule expanded on another calendar than the Gregorian
//! one, behind the `rscale` feature.
//!
//! The worked examples of RFC 7529 4.3 are the oracle here, verbatim: each
//! table gives the date in the rule's own calendar and its Gregorian
//! conversion, and the conversion is what expansion yields.

#![cfg(feature = "rscale")]

use ical::recur::{IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand};

/// The first occurrences of a rule from a start.
fn expand(rule: &str, start: IcalRecurDateTime, take: usize) -> Vec<IcalRecurDateTime> {
    IcalRecurExpand::new(IcalRecurRule::parse(rule).expect("a readable rule"), start)
        .take(take)
        .collect()
}

/// A date, for brevity.
fn date(year: i32, month: u8, day: u8) -> IcalRecurDateTime {
    IcalRecurDateTime::date(year, month, day)
}

#[test]
fn chinese_new_year_of_rfc_7529_4_3_1() {
    assert_eq!(
        expand("RSCALE=CHINESE;FREQ=YEARLY", date(2013, 2, 10), 5),
        [
            date(2013, 2, 10),
            date(2014, 1, 31),
            date(2015, 2, 19),
            date(2016, 2, 8),
            date(2017, 1, 28),
        ]
    );
}

#[test]
fn the_ethiopic_13th_month_of_rfc_7529_4_3_2() {
    assert_eq!(
        expand(
            "RSCALE=ETHIOPIC;FREQ=MONTHLY;BYMONTH=13",
            date(2013, 9, 6),
            5
        ),
        [
            date(2013, 9, 6),
            date(2014, 9, 6),
            date(2015, 9, 6),
            date(2016, 9, 6),
            date(2017, 9, 6),
        ]
    );
}

#[test]
fn the_hebrew_anniversary_of_rfc_7529_4_3_3() {
    // NOTE: The 8th of Adar I (5L), which only 5774 and 5776 have: in the
    // other years SKIP=FORWARD moves it to the 8th of Adar (6).
    assert_eq!(
        expand(
            "RSCALE=HEBREW;FREQ=YEARLY;BYMONTH=5L;BYMONTHDAY=8;SKIP=FORWARD",
            date(2014, 2, 8),
            5
        ),
        [
            date(2014, 2, 8),
            date(2015, 2, 27),
            date(2016, 2, 17),
            date(2017, 3, 6),
            date(2018, 2, 23),
        ]
    );
}

#[test]
fn a_leap_month_a_year_lacks_moves_back_or_is_omitted() {
    let rule =
        |skip: &str| format!("RSCALE=HEBREW;FREQ=YEARLY;BYMONTH=5L;BYMONTHDAY=8;SKIP={skip}");

    // NOTE: Backward is the 8th of Shevat (5) in 5775, 5777 and 5778.
    assert_eq!(
        expand(&rule("BACKWARD"), date(2014, 2, 8), 3),
        [date(2014, 2, 8), date(2015, 1, 28), date(2016, 2, 17)]
    );
    assert_eq!(
        expand(&rule("OMIT"), date(2014, 2, 8), 3),
        [date(2014, 2, 8), date(2016, 2, 17), date(2019, 2, 13)]
    );
}

#[test]
fn a_start_in_a_leap_month_recurs_by_skip() {
    // NOTE: The same anniversary with no BYMONTH: the start supplies Adar I,
    // which RFC 7529 4.1 lists as a case for SKIP.
    assert_eq!(
        expand(
            "RSCALE=HEBREW;FREQ=YEARLY;SKIP=FORWARD",
            date(2014, 2, 8),
            3
        ),
        [date(2014, 2, 8), date(2015, 2, 27), date(2016, 2, 17)]
    );
}

#[test]
fn a_monthly_rule_counts_the_leap_month() {
    // NOTE: The first of every Hebrew month from Shevat 5784, a leap year:
    // Adar I and Adar II both come before Nisan.
    assert_eq!(
        expand("RSCALE=HEBREW;FREQ=MONTHLY", date(2024, 1, 11), 4),
        [
            date(2024, 1, 11),
            date(2024, 2, 10),
            date(2024, 3, 11),
            date(2024, 4, 9),
        ]
    );
}

#[test]
fn a_walk_skips_ahead_on_another_scale() {
    let rule = IcalRecurRule::parse("RSCALE=CHINESE;FREQ=YEARLY").unwrap();
    let mut walk = IcalRecurExpand::new(rule, date(2013, 2, 10));
    walk.skip_to(date(2025, 1, 1));

    assert_eq!(walk.next(), Some(date(2025, 1, 29)));
}

/// The occurrences of a case, as the corpora spell them.
fn answer(start: &str, rule: &str) -> String {
    let start = IcalRecurDateTime::parse(start).expect("a readable start");

    expand(rule, start, 12)
        .into_iter()
        .map(|at| {
            format!(
                "{:04}{:02}{:02}T{:02}{:02}{:02}",
                at.year, at.month, at.day, at.hour, at.minute, at.second
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// The cases of a corpus under tests/corpus/recur, comments left out.
fn corpus(name: &str) -> Vec<Vec<String>> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/recur")
        .join(name);
    let text = std::fs::read_to_string(&path).expect("read the corpus");

    text.lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| line.split('\t').map(String::from).collect())
        .collect()
}

#[test]
fn replays_what_libical_with_icu_answered_and_every_settled_divergence() {
    // NOTE: One oracle, because only one exists: python-dateutil does not
    // parse RSCALE, and libical expands another scale only through ICU4C,
    // which shares the CLDR algorithms with the ICU4X ical-rs uses but no
    // code.
    let agreed = corpus("rscale.tsv");
    let settled = corpus("rscale-divergence.tsv");

    assert_eq!(agreed.len(), 1912, "the RSCALE corpus changed size");
    assert_eq!(settled.len(), 12, "the RSCALE divergences changed");

    for case in agreed.iter().chain(&settled) {
        assert_eq!(
            answer(&case[0], &case[1]),
            case[2],
            "\n  start: {}\n  rule:  {}",
            case[0],
            case[1]
        );
    }
}
