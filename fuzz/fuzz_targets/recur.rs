#![no_main]

//! Coverage-guided fuzz target for the recurrence layer. Four oracles:
//! decoding an arbitrary rule must never panic, expanding whatever decodes
//! must never panic, an expansion must terminate and come out in
//! chronological order, and skipping ahead must yield what the full walk
//! yields from that point on.
//!
//! The bound below is what makes the third oracle a test rather than a hang:
//! an endless rule is legitimate, so the target takes a fixed prefix and checks
//! that asking for it returns at all.

use ical::recur::{IcalRecurDateTime, IcalRecurRule, expand::IcalRecurExpand};
use libfuzzer_sys::fuzz_target;

/// How many occurrences one input is asked for.
const TAKE: usize = 32;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };

    // The rule and the start both come off the wire, so split the input into
    // the two of them rather than pinning the start.
    let (start, rule) = text.split_once('\n').unwrap_or(("19970902T090000", text));

    let Ok(rule) = IcalRecurRule::parse(rule) else {
        return;
    };
    let start = IcalRecurDateTime::parse(start).unwrap_or(IcalRecurDateTime::date(1997, 9, 2));

    let walked: Vec<_> = IcalRecurExpand::new(rule.clone(), start)
        .take(TAKE)
        .collect();

    let mut previous = None;
    for occurrence in &walked {
        assert!(*occurrence >= start, "an occurrence precedes the start");
        if let Some(previous) = previous {
            assert!(*occurrence > previous, "occurrences are out of order");
        }
        previous = Some(*occurrence);
    }

    // NOTE: Skip to an occurrence and to the second after the one before it,
    // so both an exact hit and a gap are checked.
    let half = walked.len() / 2;
    if half == 0 {
        return;
    }

    let after = IcalRecurDateTime::from_seconds(walked[half - 1].seconds() + 1);
    for from in [walked[half], after] {
        let mut expand = IcalRecurExpand::new(rule.clone(), start);
        let skipped: Vec<_> = expand.skip_to(from).take(TAKE - half).collect();

        assert_eq!(
            skipped,
            walked[half..],
            "skipping ahead disagrees with the full walk"
        );
    }
});
