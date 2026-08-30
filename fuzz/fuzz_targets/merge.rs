#![no_main]

//! Coverage-guided fuzz target for the three-way merge. The input is three
//! calendars separated by a NUL byte.
//!
//! Five oracles: merging never panics; the merged calendar reparses to its own
//! bytes; a merge of one calendar with itself against itself changes nothing; a
//! merge of two identical sides yields them unchanged and reports nothing,
//! unless the calendar holds two components at one path, which no addressing
//! can tell apart; and every conflict names an action the right side is
//! reported to have taken, so the report cannot contradict itself.

use ical::tree::{
    cst::{IcalCst, IcalItem},
    merge::IcalMerge,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut parts = data.split(|byte| *byte == 0);

    let (Some(base), Some(left), Some(right)) = (parts.next(), parts.next(), parts.next()) else {
        return;
    };

    let (Ok(base), Ok(left), Ok(right)) = (
        IcalCst::parse(base),
        IcalCst::parse(left),
        IcalCst::parse(right),
    ) else {
        return;
    };

    let report = IcalMerge {
        base: &base,
        left: &left,
        right: &right,
    }
    .merge();

    let bytes = report.merged.to_bytes();
    let reparsed = IcalCst::parse(&bytes).expect("the merged calendar must reparse");
    assert_eq!(
        reparsed.to_bytes(),
        bytes,
        "the merged calendar is not a fixpoint"
    );

    for conflict in &report.conflicts {
        assert!(
            report.right.contains(&conflict.right),
            "a conflict names an action the right side did not take"
        );
    }

    let quiet = IcalMerge {
        base: &base,
        left: &base,
        right: &base,
    }
    .merge();

    assert_eq!(
        quiet.merged.to_bytes(),
        base.to_bytes(),
        "merging a calendar with itself changed it"
    );
    assert!(
        quiet.conflicts.is_empty(),
        "merging a calendar with itself reported a conflict"
    );

    // NOTE: A calendar holding two components at one path, one `UID` written
    // twice with no `RECURRENCE-ID` telling them apart, cannot be addressed
    // unambiguously, and the merge says so by reporting rather than by
    // guessing. Nothing can be asserted about what it does with them.
    if ambiguous(&base) || ambiguous(&left) {
        return;
    }

    let idempotent = IcalMerge {
        base: &base,
        left: &left,
        right: &left,
    }
    .merge();

    assert_eq!(
        idempotent.merged.to_bytes(),
        left.to_bytes(),
        "merging two identical sides changed them"
    );
    assert!(
        idempotent.conflicts.is_empty(),
        "two identical sides were reported as colliding"
    );
});

/// Whether a calendar holds two components one path names, at any depth.
///
/// A component carrying no `UID` is matched by its position, so several of
/// them under one parent are not ambiguous. Two carrying one `UID` are.
fn ambiguous(cst: &IcalCst<'_>) -> bool {
    let mut seen: Vec<(String, String)> = Vec::new();

    for child in cst.items.iter().filter_map(|item| match item {
        IcalItem::Component(child) => Some(&**child),
        _ => None,
    }) {
        let held = (name_of(child), key_of(child));

        if (!held.1.is_empty() && seen.contains(&held)) || ambiguous(child) {
            return true;
        }

        seen.push(held);
    }

    false
}

/// A component's name, uppercase.
fn name_of(cst: &IcalCst<'_>) -> String {
    cst.begin
        .as_ref()
        .map(|begin| begin.raw_value_str().to_ascii_uppercase())
        .unwrap_or_default()
}

/// A component's `UID` with its `RECURRENCE-ID` after a solidus, the merge's
/// own identity, or the empty string where it carries no `UID` and its
/// position is what tells it apart.
fn key_of(cst: &IcalCst<'_>) -> String {
    let raw = |name: &str| {
        cst.items
            .iter()
            .filter_map(|item| match item {
                IcalItem::Prop(line) => Some(line),
                _ => None,
            })
            .find(|line| line.name.get().eq_ignore_ascii_case(name))
            .map(|line| line.raw_value_str().into_owned())
    };

    match (raw("UID"), raw("RECURRENCE-ID")) {
        (Some(uid), Some(held)) => format!("{uid}/{held}"),
        (Some(uid), None) => uid,
        (None, _) => String::new(),
    }
}
