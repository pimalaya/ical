//! RFC 7529 `SKIP`, the half of non-Gregorian recurrence that is Gregorian.
//!
//! `SKIP` says what a rule means when the date it names does not exist: the
//! 29th of February in a non-leap year, the 31st of a month with thirty days.
//! Without it those instances are simply missing, which is RFC 5545's rule and
//! almost never what a person meant. It needs no calendar system, only the
//! month lengths this crate already knows, which is why it is implemented here
//! and `RSCALE=CHINESE` is not.

use ical::recur::{
    IcalRecurDateTime, IcalRecurRule, IcalRecurRuleProblem, expand::IcalRecurExpand,
};

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
fn the_worked_example_of_rfc_7529_4_3_4() {
    // NOTE: The RFC's own table, verbatim: an anniversary on the 29th of
    // February, which without SKIP happens only in leap years.
    let plain = expand("FREQ=YEARLY", date(2012, 2, 29), 3);

    assert_eq!(
        plain,
        [date(2012, 2, 29), date(2016, 2, 29), date(2020, 2, 29)]
    );

    let skipped = expand(
        "RSCALE=GREGORIAN;FREQ=YEARLY;SKIP=FORWARD",
        date(2012, 2, 29),
        6,
    );

    assert_eq!(
        skipped,
        [
            date(2012, 2, 29),
            date(2013, 3, 1),
            date(2014, 3, 1),
            date(2015, 3, 1),
            date(2016, 2, 29),
            date(2017, 3, 1),
        ]
    );
}

#[test]
fn skips_a_leap_day_backward_onto_the_last_day_of_february() {
    let skipped = expand(
        "RSCALE=GREGORIAN;FREQ=YEARLY;SKIP=BACKWARD",
        date(2012, 2, 29),
        4,
    );

    assert_eq!(
        skipped,
        [
            date(2012, 2, 29),
            date(2013, 2, 28),
            date(2014, 2, 28),
            date(2015, 2, 28),
        ]
    );
}

#[test]
fn fills_in_the_months_that_have_no_thirty_first() {
    // NOTE: The other case people actually hit: a monthly event on the 31st.
    let skipped = expand(
        "RSCALE=GREGORIAN;FREQ=MONTHLY;SKIP=BACKWARD",
        date(2026, 1, 31),
        5,
    );

    assert_eq!(
        skipped,
        [
            date(2026, 1, 31),
            date(2026, 2, 28),
            date(2026, 3, 31),
            date(2026, 4, 30),
            date(2026, 5, 31),
        ]
    );
}

#[test]
fn moves_a_missing_day_of_the_month_forward_into_the_next_one() {
    let skipped = expand(
        "RSCALE=GREGORIAN;FREQ=MONTHLY;SKIP=FORWARD",
        date(2026, 1, 31),
        4,
    );

    assert_eq!(
        skipped,
        [
            date(2026, 1, 31),
            date(2026, 3, 1),
            date(2026, 3, 31),
            date(2026, 5, 1),
        ]
    );
}

#[test]
fn resolves_an_explicit_bymonthday_too() {
    let skipped = expand(
        "RSCALE=GREGORIAN;FREQ=MONTHLY;BYMONTHDAY=30;SKIP=BACKWARD",
        date(2026, 1, 30),
        3,
    );

    assert_eq!(
        skipped,
        [date(2026, 1, 30), date(2026, 2, 28), date(2026, 3, 30)]
    );
}

#[test]
fn leaves_a_negative_bymonthday_alone() {
    // NOTE: `-1` counts back from the end of the month, so it always names a
    // day that exists and there is nothing to resolve.
    let skipped = expand(
        "RSCALE=GREGORIAN;FREQ=MONTHLY;BYMONTHDAY=-1;SKIP=FORWARD",
        date(2026, 1, 31),
        3,
    );

    assert_eq!(
        skipped,
        [date(2026, 1, 31), date(2026, 2, 28), date(2026, 3, 31)]
    );
}

#[test]
fn omit_is_the_default_and_changes_nothing() {
    let with = expand(
        "RSCALE=GREGORIAN;FREQ=MONTHLY;SKIP=OMIT",
        date(2026, 1, 31),
        3,
    );
    let without = expand("FREQ=MONTHLY", date(2026, 1, 31), 3);

    assert_eq!(with, without);
    assert_eq!(
        with,
        [date(2026, 1, 31), date(2026, 3, 31), date(2026, 5, 31)]
    );
}

#[test]
fn yields_no_duplicate_when_a_resolved_day_lands_on_a_real_one() {
    // NOTE: The 30th and the 31st both resolve backward onto the 28th in
    // February, and an occurrence is emitted once (RFC 5545 3.8.5.3).
    let skipped = expand(
        "RSCALE=GREGORIAN;FREQ=MONTHLY;BYMONTHDAY=30,31;SKIP=BACKWARD",
        date(2026, 2, 1),
        3,
    );

    assert_eq!(
        skipped,
        [date(2026, 2, 28), date(2026, 3, 30), date(2026, 3, 31)]
    );
}

#[test]
fn keeps_the_occurrences_in_order() {
    // NOTE: A forward resolution lands in the next month, after days the same
    // period already produced, so the buffer has to be sorted.
    let skipped = expand(
        "RSCALE=GREGORIAN;FREQ=YEARLY;BYMONTH=1,2;BYMONTHDAY=30;SKIP=FORWARD",
        date(2026, 1, 1),
        4,
    );

    assert_eq!(
        skipped,
        [
            date(2026, 1, 30),
            date(2026, 3, 1),
            date(2027, 1, 30),
            date(2027, 3, 1),
        ]
    );
}

#[test]
fn an_alien_calendar_scale_still_yields_nothing() {
    // NOTE: RSCALE names a CLDR calendar system, and this crate expands only
    // the Gregorian one. Yielding nothing is the honest answer; yielding
    // Gregorian dates under a Hebrew rule would be a wrong one.
    let hebrew = expand(
        "RSCALE=HEBREW;FREQ=YEARLY;SKIP=FORWARD",
        date(2026, 1, 1),
        3,
    );

    assert!(hebrew.is_empty());
}

#[test]
fn reports_a_skip_with_no_rscale_beside_it() {
    // NOTE: RFC 7529 4 states it as a syntax rule, but a rule carrying it is
    // still a rule, so this is a validation problem rather than a parse error.
    let rule = IcalRecurRule::parse("FREQ=YEARLY;SKIP=FORWARD").expect("a readable rule");

    assert_eq!(rule.problems(), [IcalRecurRuleProblem::SkipWithoutScale]);

    let proper =
        IcalRecurRule::parse("RSCALE=GREGORIAN;FREQ=YEARLY;SKIP=FORWARD").expect("a readable rule");

    assert!(proper.problems().is_empty());
}

#[test]
fn replays_what_libical_answered() {
    // NOTE: One oracle rather than the corpus's two, because only one exists:
    // python-dateutil does not parse RSCALE at all. libical resolves SKIP for
    // the Gregorian scale without an ICU build, which is exactly the half of
    // RFC 7529 this crate implements, so it is the one implementation that can
    // be asked.
    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/recur/skip.tsv");
    let text = std::fs::read_to_string(&path).expect("read the corpus");

    let cases: Vec<&str> = text
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .collect();

    assert_eq!(cases.len(), 10, "the SKIP corpus changed size");

    for case in cases {
        let mut fields = case.splitn(3, '\t');
        let start = fields.next().unwrap_or_default();
        let rule = fields.next().unwrap_or_default();
        let expected = fields.next().unwrap_or_default();

        let occurrences: Vec<String> = expand(
            rule,
            IcalRecurDateTime::parse(start).expect("a readable start"),
            12,
        )
        .into_iter()
        .map(|at| {
            format!(
                "{:04}{:02}{:02}T{:02}{:02}{:02}",
                at.year, at.month, at.day, at.hour, at.minute, at.second
            )
        })
        .collect();

        assert_eq!(
            occurrences.join(","),
            expected,
            "\n  start: {start}\n  rule:  {rule}"
        );
    }
}
