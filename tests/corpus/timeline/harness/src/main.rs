//! Regeneration tool for the frozen timeline corpus.
//!
//! Three subcommands, chained by `run.sh`, CORPUS being tests/corpus:
//!
//! - `windows CORPUS` writes one window per calendar of the [`DIRS`] under
//!   CORPUS: from a year before its earliest `DTSTART`, `DUE` or
//!   `RECURRENCE-ID` to three years after its latest.
//! - `expand CORPUS` reads that window list and answers it through ical-rs.
//! - `cross ICAL RIE ICALJS SPLIT` reads the three answer files (ical-rs,
//!   recurring-ical-events, ical.js). It writes every series all three answer
//!   alike to stdout; every one the oracles split on, ical-rs agreeing with
//!   one of them, to SPLIT with ical-rs's answer and that oracle's name; and
//!   every one ical-rs agrees with neither oracle on, and every calendar an
//!   oracle refuses, to stderr with the answers labelled, for a human to
//!   triage into divergence.tsv or into a fix.
//!
//! An answer line is `calendar<TAB>uid<TAB>start<TAB>end`, the earliest
//! [`TAKE`] occurrences of a series starting in the window, a start spelled
//! as UTC (`Z`), floating, or a date. The components with no `UID` are one
//! series under the label `-`, as the oracles have no stable name for them.
//!
//! Nothing here ships with the crate: `tests/corpus/timeline/harness` is a
//! detached package that cargo never builds as part of ical-rs.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{BufRead, stdin},
    path::{Path, PathBuf},
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

/// The calendar folders walked, under tests/corpus. The first one's calendars
/// are named bare, as the corpus first named them; every other one's by its
/// path under tests/corpus, so no two names collide.
const DIRS: [&str; 3] = ["rie", "icaljs", "libical"];

fn main() {
    let args: Vec<String> = env::args().collect();

    match (args.get(1).map(String::as_str), &args[2.min(args.len())..]) {
        (Some("windows"), [corpus]) => windows(Path::new(corpus)),
        (Some("expand"), [corpus]) => expand(Path::new(corpus)),
        (Some("cross"), [ical, rie, icaljs, split]) => cross(ical, rie, icaljs, split),
        _ => {
            eprintln!(
                "usage: harness windows CORPUS | expand CORPUS | cross ICAL RIE ICALJS SPLIT"
            );
            exit(2);
        }
    }
}

/// The file a calendar name stands for: a path under CORPUS, or a bare name in
/// the first of the [`DIRS`].
fn path_of(corpus: &Path, name: &str) -> PathBuf {
    match name.contains('/') {
        true => corpus.join(name),
        false => corpus.join(DIRS[0]).join(name),
    }
}

fn windows(corpus: &Path) {
    for (index, dir) in DIRS.iter().enumerate() {
        let mut names: Vec<String> = fs::read_dir(corpus.join(dir))
            .expect("read a calendar directory")
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .filter(|name| name.ends_with(".ics"))
            .map(|name| match index {
                0 => name,
                _ => format!("{dir}/{name}"),
            })
            .collect();
        names.sort();

        for name in names {
            window(corpus, &name);
        }
    }
}

fn window(corpus: &Path, name: &str) {
    let bytes = fs::read(path_of(corpus, name)).expect("read a calendar");
    let text = String::from_utf8_lossy(&bytes).into_owned();

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

    // NOTE: Clamped to four-digit years, which a malformed calendar (a
    // year 0000 or 9999) would otherwise leave.
    let (lo, hi) = ((lo - 1).max(0), (hi + 3).min(9999));

    println!("{name}\t{lo:04}0101\t{hi:04}0101");
}

fn expand(corpus: &Path) {
    for line in stdin().lock().lines() {
        let line = line.expect("read the window list");
        let mut fields = line.split('\t');
        let (Some(name), Some(from), Some(until)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };

        let bytes = fs::read(path_of(corpus, name)).expect("read a calendar");

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
    let seconds = |day: &str| {
        IcalRecurDateTime::parse(day)
            .expect("a window day")
            .seconds()
    };
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
    let end = spell(
        occurrence.end,
        occurrence.date && midnight,
        occurrence.floating,
    );
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

    for line in fs::read_to_string(path)
        .expect("read an answer file")
        .lines()
    {
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

fn cross(ical: &str, rie: &str, icaljs: &str, split: &str) {
    let answers = [read(ical), read(rie), read(icaljs)];
    let series: BTreeSet<&(String, String)> = answers.iter().flat_map(Answers::keys).collect();
    let none = Vec::new();
    let mut splits = String::new();

    for key in series {
        let (name, uid) = key;
        let [ours, rie, icaljs] = answers
            .each_ref()
            .map(|answers| answers.get(key).unwrap_or(&none).join(","));

        // NOTE: An oracle's refusal is no series: its calendar is listed for
        // the human, whose series the other oracle answers are crossed as
        // usual.
        if uid == "ERROR" {
            eprintln!("{name}\tERROR\n  rie:    {rie}\n  icaljs: {icaljs}");
            continue;
        }

        let oracle = match (ours == rie, ours == icaljs) {
            (true, true) => {
                println!("{name}\t{uid}\t{ours}");
                continue;
            }
            (true, false) => "rie",
            (false, true) => "icaljs",
            (false, false) => {
                eprintln!("{name}\t{uid}\n  ical:   {ours}\n  rie:    {rie}\n  icaljs: {icaljs}");
                continue;
            }
        };

        splits.push_str(&format!("{name}\t{uid}\t{ours}\t{oracle}\n"));
    }

    fs::write(split, splits).expect("write the split answers");
}
