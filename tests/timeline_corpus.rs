//! The frozen timeline corpus, replayed.
//!
//! The corpus crosses this crate with two oracles, recurring-ical-events 3.9.0
//! and ical.js 2.2.1, over the test calendars of recurring-ical-events, ical.js
//! and libical (tests/corpus/rie, tests/corpus/icaljs, tests/corpus/libical)
//! and the windows of tests/corpus/timeline/windows.tsv. Beside it,
//! `agreement.tsv` holds every series all three answered alike, `split.tsv`
//! every one the oracles split on with this crate's answer and the oracle it
//! agrees with, and `divergence.tsv` every one this crate deliberately answers
//! as neither oracle does, with the reason.
//!
//! Replaying them needs neither Python nor Node. The answer is computed as the
//! harness under tests/corpus/timeline/harness computes it, and the two must
//! stay alike.

#![cfg(all(feature = "parser", feature = "tzdb"))]

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use ical::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    prop::{IcalPropKind, IcalPropName},
    recur::{
        IcalRecurDateTime,
        instant::{IcalRecurInstant, IcalRecurTimeline},
    },
    tree::cst::IcalCst,
    tz::IcalTz,
    tzdb,
    value::IcalValue,
};

/// How many occurrences of one series the corpus states, matching the harness.
const TAKE: usize = 200;

/// How far past the window the walk reaches, matching the harness.
const MARGIN: i64 = 3 * 86_400;

fn corpus(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus")
        .join(path)
}

/// The non-comment lines of a corpus file, split on tabs.
fn lines(path: &str) -> Vec<Vec<String>> {
    fs::read_to_string(corpus(path))
        .expect("read the corpus")
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| line.split('\t').map(str::to_string).collect())
        .collect()
}

/// The occurrences this crate gives every series of a calendar over a window,
/// in the corpus's spelling.
fn answer(bytes: &[u8], from: &str, until: &str) -> BTreeMap<String, String> {
    let seconds = |day: &str| IcalRecurDateTime::parse(day).expect("a day").seconds();
    let (from, until) = (seconds(from), seconds(until));
    let through = IcalRecurDateTime::from_seconds(until).year;

    let Ok(cst) = IcalCst::parse(bytes) else {
        return BTreeMap::new();
    };

    let mut ical = cst.decode();

    if let Some(name) = IcalTz::x_wr_timezone(&ical).map(str::to_string) {
        let mut zones = tzdb::zones(&ical.components, through);
        zones.extend(tzdb::zone(&name, through));
        IcalTz::apply_x_wr_timezone(&mut ical, &zones);
    }

    let zones = tzdb::zones(&ical.components, through);

    let series: Vec<&IcalComponent<'_>> = ical
        .components
        .iter()
        .filter(|component| {
            matches!(
                component.name,
                IcalComponentName::Kind(
                    IcalComponentKind::VEvent
                        | IcalComponentKind::VTodo
                        | IcalComponentKind::VJournal
                )
            )
        })
        .collect();

    let uids: BTreeSet<&str> = series
        .iter()
        .filter_map(|component| uid_of(component))
        .collect();

    let mut timelines: Vec<(String, IcalRecurTimeline)> = uids
        .into_iter()
        .map(|uid| {
            let timeline = IcalRecurTimeline::of_uid(&ical.components, uid, &zones);
            (uid.to_string(), timeline)
        })
        .collect();

    for component in series
        .iter()
        .filter(|component| uid_of(component).is_none())
    {
        let timeline = IcalRecurTimeline::of_component(component, &zones);
        timelines.push(("-".to_string(), timeline));
    }

    let mut rows: BTreeMap<String, Vec<(i64, String)>> = BTreeMap::new();

    for (uid, timeline) in &timelines {
        rows.entry(uid.clone()).or_default().extend(
            timeline
                .expand(from - MARGIN, until + MARGIN)
                .filter(|occurrence| occurrence.start >= from && occurrence.start < until)
                .map(|occurrence| (occurrence.start, row(&occurrence))),
        );
    }

    rows.into_iter()
        .filter(|(_, rows)| !rows.is_empty())
        .map(|(uid, mut rows)| {
            rows.sort();
            let rows: Vec<String> = rows.into_iter().take(TAKE).map(|(_, row)| row).collect();
            (uid, rows.join(","))
        })
        .collect()
}

fn row(occurrence: &IcalRecurInstant) -> String {
    let midnight = occurrence.end.rem_euclid(86_400) == 0;
    let start = spell(occurrence.start, occurrence.date, occurrence.floating);
    let end = spell(
        occurrence.end,
        occurrence.date && midnight,
        occurrence.floating,
    );
    format!("{start}/{end}")
}

fn spell(seconds: i64, date: bool, floating: bool) -> String {
    let at = IcalRecurDateTime::from_seconds(seconds);
    let day = format!("{:04}{:02}{:02}", at.year, at.month, at.day);
    let time = format!("T{:02}{:02}{:02}", at.hour, at.minute, at.second);

    match (date, floating) {
        (true, _) => day,
        (false, true) => day + &time,
        (false, false) => day + &time + "Z",
    }
}

fn uid_of<'a>(component: &'a IcalComponent<'_>) -> Option<&'a str> {
    component.props.iter().find_map(|prop| {
        if !matches!(prop.name, IcalPropName::Kind(IcalPropKind::Uid)) {
            return None;
        }

        match &prop.value {
            IcalValue::Text(text) => Some(&*text.0),
            _ => None,
        }
    })
}

#[test]
fn replays_what_the_oracles_answered_and_every_settled_divergence() {
    let mut ours = BTreeMap::new();

    for window in lines("timeline/windows.tsv") {
        // NOTE: A bare name is one of tests/corpus/rie, the corpus's first
        // folder; every other one is named by its path under tests/corpus.
        let path = match window[0].contains('/') {
            true => corpus(&window[0]),
            false => corpus("rie").join(&window[0]),
        };

        let bytes = fs::read(path).expect("read a calendar");

        for (uid, answer) in answer(&bytes, &window[1], &window[2]) {
            ours.insert((window[0].clone(), uid), answer);
        }
    }

    let agreement = lines("timeline/agreement.tsv");
    let split = lines("timeline/split.tsv");
    let divergence = lines("timeline/divergence.tsv");

    // NOTE: The exact counts, so a corpus that silently shrinks is caught too.
    assert_eq!(agreement.len(), 924, "the agreement corpus changed size");
    assert_eq!(split.len(), 82, "the split corpus changed size");
    assert_eq!(divergence.len(), 7, "the divergence corpus changed size");

    for line in &split {
        assert!(
            matches!(line[3].as_str(), "rie" | "icaljs"),
            "a split names no oracle: {} {}",
            line[0],
            line[1]
        );
    }

    let mut listed = BTreeSet::new();

    for line in agreement.iter().chain(&split).chain(&divergence) {
        let key = (line[0].clone(), line[1].clone());
        let answer = ours.get(&key).map_or("", String::as_str);

        assert_eq!(
            answer, line[2],
            "\n  calendar: {}\n  uid:      {}",
            line[0], line[1]
        );

        listed.insert(key);
    }

    for key in ours.keys() {
        assert!(
            listed.contains(key),
            "a series no corpus lists: {} {}",
            key.0,
            key.1
        );
    }
}
