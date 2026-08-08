#![no_main]

//! Coverage-guided fuzz target for the recurrence layer. Three oracles:
//! decoding an arbitrary rule must never panic, expanding whatever decodes
//! must never panic, and an expansion must terminate and come out in
//! chronological order.
//!
//! The bound below is what makes the last oracle a test rather than a hang: an
//! endless rule is legitimate, so the target takes a fixed prefix and checks
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

    let mut previous = None;
    for occurrence in IcalRecurExpand::new(rule, start).take(TAKE) {
        assert!(occurrence >= start, "an occurrence precedes the start");
        if let Some(previous) = previous {
            assert!(occurrence > previous, "occurrences are out of order");
        }
        previous = Some(occurrence);
    }
});
