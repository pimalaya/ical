//! Regeneration tool for the frozen timeline corpus.
//!
//! Three subcommands, chained by `run.sh`:
//!
//! - `windows DIR` writes one window per calendar of DIR: from a year before
//!   its earliest `DTSTART`, `DUE` or `RECURRENCE-ID` to three years after its
//!   latest.
//! - `expand DIR` reads that window list and answers it through ical-rs.
//! - `cross ICAL ORACLE` reads the two answer files and writes every series
//!   both answer alike to stdout, and every other one to stderr, for a human
//!   to triage into divergence.tsv or into a fix.
//!
//! An answer line is `calendar<TAB>uid<TAB>start<TAB>end`, the earliest
//! [`TAKE`] occurrences of a series starting in the window, a start spelled
//! as UTC (`Z`), floating, or a date. The components with no `UID` are one
//! series under the label `-`, as the oracle has no stable name for them.
//!
//! Nothing here ships with the crate: `tests/corpus/timeline/harness` is a
//! detached package that cargo never builds as part of ical-rs.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{BufRead, stdin},
    path::Path,
    process::exit,
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

/// How many occurrences of one series are compared, the earliest first.
const TAKE: usize = 200;

/// How far past the window both sides fetch, so their own window semantics
/// never matter: the cut is the harness's, on starts.
const MARGIN: i64 = 3 * 86_400;

fn main() {
    let args: Vec<String> = env::args().collect();

    match (args.get(1).map(String::as_str), args.get(2), args.get(3)) {
        (Some("windows"), Some(dir), None) => windows(Path::new(dir)),
        (Some("expand"), Some(dir), None) => expand(Path::new(dir)),
        (Some("cross"), Some(ical), Some(oracle)) => cross(ical, oracle),
        _ => {
            eprintln!("usage: harness windows DIR | expand DIR | cross ICAL ORACLE");
            exit(2);
        }
    }
}

fn windows(dir: &Path) {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("read the calendar directory")
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| name.ends_with(".ics"))
        .collect();
    names.sort();

    for name in names {
        let text = String::from_utf8_lossy(&fs::read(dir.join(&name)).expect("read")).into_owned();

        let years: Vec<i32> = text
            .lines()
            .filter(|line| {
                let upper = line.to_ascii_uppercase();
                ["DTSTART", "DUE", "RECURRENCE-ID"]
                    .iter()
                    .any(|prop| upper.starts_with(prop))
            })
            .filter_map(|line| line.rsplit(':').next()?.get(..4)?.parse().ok())
            .collect();

        let (lo, hi) = match (years.iter().min(), years.iter().max()) {
            (Some(lo), Some(hi)) => (*lo, *hi),
            _ => (2000, 2000),
        };

        println!("{name}\t{:04}0101\t{:04}0101", lo - 1, hi + 3);
    }
}

fn expand(dir: &Path) {
    for line in stdin().lock().lines() {
        let line = line.expect("read the window list");
        let mut fields = line.split('\t');
        let (Some(name), Some(from), Some(until)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };

        let bytes = fs::read(dir.join(name)).expect("read a calendar");

        for (uid, rows) in answer(&bytes, from, until) {
            for row in rows {
                println!("{name}\t{uid}\t{row}");
            }
        }
    }
}

/// The occurrences ical-rs gives every series of a calendar over a window,
/// as the replay test computes them too.
fn answer(bytes: &[u8], from: &str, until: &str) -> BTreeMap<String, Vec<String>> {
    let seconds = |day: &str| IcalRecurDateTime::parse(day).expect("a window day").seconds();
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

    let uids: BTreeSet<&str> = series.iter().filter_map(|component| uid_of(component)).collect();

    let mut timelines: Vec<(String, IcalRecurTimeline)> = uids
        .into_iter()
        .map(|uid| {
            let timeline = IcalRecurTimeline::of_uid(&ical.components, uid, &zones);
            (uid.to_string(), timeline)
        })
        .collect();

    for component in series.iter().filter(|component| uid_of(component).is_none()) {
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
        .map(|(uid, mut rows)| {
            rows.sort();
            let rows = rows.into_iter().take(TAKE).map(|(_, row)| row).collect();
            (uid, rows)
        })
        .collect()
}

/// An occurrence as `start<TAB>end`, a date-only end spelled as a date when it
/// falls on midnight.
fn row(occurrence: &IcalRecurInstant) -> String {
    let midnight = occurrence.end.rem_euclid(86_400) == 0;
    let start = spell(occurrence.start, occurrence.date, occurrence.floating);
    let end = spell(occurrence.end, occurrence.date && midnight, occurrence.floating);
    format!("{start}\t{end}")
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

type Answers = BTreeMap<(String, String), Vec<String>>;

fn read(path: &str) -> Answers {
    let mut answers = Answers::new();

    for line in fs::read_to_string(path).expect("read an answer file").lines() {
        let mut fields = line.splitn(3, '\t');
        let (Some(name), Some(uid), Some(rest)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };

        answers
            .entry((name.to_string(), uid.to_string()))
            .or_default()
            .push(rest.replace('\t', "/"));
    }

    answers
}

fn cross(ical: &str, oracle: &str) {
    let (ical, oracle) = (read(ical), read(oracle));
    let series: BTreeSet<&(String, String)> = ical.keys().chain(oracle.keys()).collect();
    let none = Vec::new();

    for key in series {
        let (name, uid) = key;
        let ours = ical.get(key).unwrap_or(&none);
        let theirs = oracle.get(key).unwrap_or(&none);

        if ours == theirs {
            println!("{name}\t{uid}\t{}", ours.join(","));
        } else {
            eprintln!("{name}\t{uid}\n  ical:   {}\n  oracle: {}", ours.join(","), theirs.join(","));
        }
    }
}
