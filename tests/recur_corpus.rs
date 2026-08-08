//! The frozen recurrence corpus, replayed.
//!
//! `tests/corpus/recur/consensus.tsv` holds every generated case on which
//! python-dateutil 2.9 and libical 3.0.20 answered alike, with the answer they
//! agreed on. `divergence.tsv` beside it holds the cases where this crate
//! deliberately answers something an oracle does not, with the reason.
//!
//! Replaying them needs neither Python nor a C toolchain, which is the point:
//! the evidence of a differential run against two independent implementations
//! is a file in this repository rather than a claim in a commit message. The
//! harness that produced it is under `tests/corpus/recur/harness`.

use std::{fs, path::PathBuf};

use ical::recur::{IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand};

/// How many occurrences each case states, matching the harness.
const TAKE: usize = 12;

/// One case: a start, a rule, and the occurrences expected of them.
struct Case {
    start: String,
    rule: String,
    expected: String,
    note: String,
}

fn cases(name: &str) -> Vec<Case> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/recur")
        .join(name);
    let text = fs::read_to_string(&path).expect("read the corpus");

    text.lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let mut fields = line.splitn(4, '\t');
            Case {
                start: fields.next().unwrap_or_default().to_string(),
                rule: fields.next().unwrap_or_default().to_string(),
                expected: fields.next().unwrap_or_default().to_string(),
                note: fields.next().unwrap_or_default().to_string(),
            }
        })
        .collect()
}

/// The occurrences this crate yields for one case, in the corpus's spelling.
fn answer(case: &Case) -> String {
    let start = IcalRecurDateTime::parse(&case.start).expect("a readable start");
    let rule = IcalRecurRule::parse(&case.rule).expect("a readable rule");

    IcalRecurExpand::new(rule, start)
        .take(TAKE)
        .map(|at| {
            format!(
                "{:04}{:02}{:02}T{:02}{:02}{:02}",
                at.year, at.month, at.day, at.hour, at.minute, at.second
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn replays_what_both_oracles_agreed_on() {
    let cases = cases("consensus.tsv");

    // The exact count, so a corpus that silently shrinks is caught too.
    assert_eq!(cases.len(), 4331, "the consensus corpus changed size");

    for case in &cases {
        assert_eq!(
            answer(case),
            case.expected,
            "\n  start: {}\n  rule:  {}",
            case.start,
            case.rule
        );
    }
}

#[test]
fn replays_every_deliberate_divergence() {
    let cases = cases("divergence.tsv");

    assert_eq!(cases.len(), 4, "the divergence corpus changed size");

    for case in &cases {
        assert!(!case.note.is_empty(), "a divergence with no reason given");
        assert_eq!(
            answer(case),
            case.expected,
            "\n  start: {}\n  rule:  {}\n  why:   {}",
            case.start,
            case.rule,
            case.note
        );
    }
}
