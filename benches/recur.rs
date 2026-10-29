//! Recurrence benchmarks, comparing like with like by level.
//!
//! `expand_first_1000` pits rule expansion against the `rrule` crate: the
//! first 1,000 occurrences of a handful of common rules on a zoned start,
//! each turned into an instant, since `rrule` yields zoned date-times. Ours
//! is the civil expansion through the zone's gap filter, then each occurrence
//! resolved through the zone's held transitions.
//!
//! `expand_far_window` asks both for one week of 2026 out of a rule started
//! in 1970: ours skips to the window, `rrule` walks to it.
//!
//! `timeline_walk` walks every series of a real calendar over its window. No
//! Rust crate walks a series with its overrides to instants, so it is tracked
//! against ourselves.
//!
//! Before timing, every group checks the two crates yield the same instants:
//! a fast wrong answer is no comparison.

use std::{hint::black_box, str::FromStr};

use criterion::{Criterion, criterion_group, criterion_main};

use ical::{
    prop::{IcalPropKind, IcalPropName},
    recur::{
        IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand, instant::IcalRecurTimeline,
    },
    tree::cst::IcalCst,
    tz::{IcalTz, IcalTzTransitions},
    value::IcalValue,
};
use rrule::RRuleSet;

/// Europe/Paris as calendars carry it.
const PARIS: &str = concat!(
    "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n",
    "BEGIN:VTIMEZONE\r\nTZID:Europe/Paris\r\n",
    "BEGIN:DAYLIGHT\r\nDTSTART:19810329T020000\r\n",
    "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU\r\n",
    "TZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nEND:DAYLIGHT\r\n",
    "BEGIN:STANDARD\r\nDTSTART:19961027T030000\r\n",
    "RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU\r\n",
    "TZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nEND:STANDARD\r\n",
    "END:VTIMEZONE\r\nEND:VCALENDAR\r\n",
);

/// Common rules, each started on Monday 5 January 2026 at 09:00 in Paris.
const RULES: &[(&str, &str)] = &[
    ("daily", "FREQ=DAILY"),
    ("weekly BYDAY", "FREQ=WEEKLY;BYDAY=MO,WE,FR"),
    (
        "monthly BYSETPOS",
        "FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1",
    ),
    (
        "yearly BYMONTH BYDAY",
        "FREQ=YEARLY;BYMONTH=3,6,9,12;BYDAY=1MO",
    ),
    ("hourly BYHOUR", "FREQ=HOURLY;BYHOUR=9,12,17"),
];

/// The calendar the timeline walk reads, and its window in Unix seconds.
const CALENDAR: &str = include_str!("../tests/corpus/rie/fablab_cottbus.ics");
const WINDOW: (i64, i64) = (1_483_228_800, 1_672_531_200);

fn paris() -> IcalTz {
    let cst = IcalCst::parse(PARIS).unwrap();
    IcalTz::of_calendar(&cst.decode(), "Europe/Paris").unwrap()
}

/// Our side: the first `take` occurrences at or after `from` as instants.
fn ours(rule: &str, start: IcalRecurDateTime, from: IcalRecurDateTime, take: usize) -> Vec<i64> {
    let zone = paris();
    let mut transitions = IcalTzTransitions::of_zone(zone.clone());
    let mut expand = IcalRecurExpand::new(IcalRecurRule::parse(rule).unwrap(), start).in_zone(zone);
    expand.skip_to(from);

    expand
        .take(take)
        .filter_map(|local| transitions.resolve(local).instant(local))
        .collect()
}

/// Our wall-clock times, for the check: the instants depend on how far each
/// side's zone data reaches, chrono-tz stopping summer time in 2100.
fn ours_civil(rule: &str, start: IcalRecurDateTime, take: usize) -> Vec<String> {
    IcalRecurExpand::new(IcalRecurRule::parse(rule).unwrap(), start)
        .in_zone(paris())
        .take(take)
        .map(|at| {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                at.year, at.month, at.day, at.hour, at.minute, at.second
            )
        })
        .collect()
}

/// The `rrule` crate's wall-clock times, for the same check.
fn theirs_civil(rule: &str, start: &str, take: u16) -> Vec<String> {
    let text = format!("DTSTART;TZID=Europe/Paris:{start}\nRRULE:{rule}");

    RRuleSet::from_str(&text)
        .unwrap()
        .all(take)
        .dates
        .into_iter()
        .map(|date| date.naive_local().format("%Y-%m-%dT%H:%M:%S").to_string())
        .collect()
}

/// The `rrule` crate's side, the same question in its own terms.
fn theirs(rule: &str, start: &str, window: Option<(i64, i64)>, take: u16) -> Vec<i64> {
    let text = format!("DTSTART;TZID=Europe/Paris:{start}\nRRULE:{rule}");
    let mut set = RRuleSet::from_str(&text).unwrap();

    if let Some((from, until)) = window {
        let tz = rrule::Tz::Tz(chrono_tz::Europe::Paris);
        let at = |seconds| {
            chrono::DateTime::from_timestamp(seconds, 0)
                .unwrap()
                .with_timezone(&tz)
        };
        set = set.after(at(from)).before(at(until));
    }

    set.all(take)
        .dates
        .into_iter()
        .map(|date| date.timestamp())
        .collect()
}

const START: IcalRecurDateTime = IcalRecurDateTime {
    year: 2026,
    month: 1,
    day: 5,
    hour: 9,
    minute: 0,
    second: 0,
};

fn expand_first_1000(c: &mut Criterion) {
    let mut group = c.benchmark_group("expand_first_1000");

    for (name, rule) in RULES {
        assert!(
            ours_civil(rule, START, 1000) == theirs_civil(rule, "20260105T090000", 1000),
            "{name}: the two crates disagree"
        );

        group.bench_function(format!("ical-rs: {name}"), |b| {
            b.iter(|| ours(black_box(rule), START, START, 1000))
        });
        group.bench_function(format!("rrule: {name}"), |b| {
            b.iter(|| theirs(black_box(rule), "20260105T090000", None, 1000))
        });
    }

    group.finish();
}

fn expand_far_window(c: &mut Criterion) {
    let mut group = c.benchmark_group("expand_far_window");

    let rule = "FREQ=WEEKLY;BYDAY=MO,WE,FR";
    let start = IcalRecurDateTime {
        year: 1970,
        ..START
    };
    let from = IcalRecurDateTime::date(2026, 7, 6);
    // NOTE: Monday 6 to Monday 13 July 2026, midnight in Paris.
    let window = (1_783_288_800, 1_783_893_600);

    let week = |instants: Vec<i64>| -> Vec<i64> {
        instants
            .into_iter()
            .filter(|instant| (window.0..window.1).contains(instant))
            .collect()
    };

    let expected = week(ours(rule, start, from, 4));
    assert_eq!(expected.len(), 3);
    assert_eq!(expected, theirs(rule, "19700105T090000", Some(window), 10));

    group.bench_function("ical-rs: skip_to", |b| {
        b.iter(|| week(ours(black_box(rule), start, from, 4)))
    });
    group.bench_function("rrule: after + before", |b| {
        b.iter(|| theirs(black_box(rule), "19700105T090000", Some(window), 10))
    });

    group.finish();
}

fn timeline_walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeline_walk");

    let cst = IcalCst::parse(CALENDAR).unwrap();
    let ical = cst.decode();
    let zones = IcalTz::of_components(&ical.components);

    let mut uids: Vec<&str> = ical
        .components
        .iter()
        .flat_map(|component| &component.props)
        .filter_map(|prop| match (&prop.name, &prop.value) {
            (IcalPropName::Kind(IcalPropKind::Uid), IcalValue::Text(text)) => Some(&*text.0),
            _ => None,
        })
        .collect();
    uids.sort_unstable();
    uids.dedup();

    let walk = || -> usize {
        uids.iter()
            .map(|uid| {
                IcalRecurTimeline::of_uid(&ical.components, uid, &zones)
                    .expand(WINDOW.0, WINDOW.1)
                    .count()
            })
            .sum()
    };

    assert!(walk() > 0);

    group.bench_function("ical-rs: fablab_cottbus.ics, every series", |b| {
        b.iter(|| black_box(walk()))
    });

    group.finish();
}

criterion_group!(benches, expand_first_1000, expand_far_window, timeline_walk);
criterion_main!(benches);
