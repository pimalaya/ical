//! The recovering parser, over the calendars the strict one refuses.
//!
//! Twelve of the 190 vendored fixtures cannot be structured: a content line
//! with no colon, a component never closed, a name that is not UTF-8. The
//! strict entry point refuses all twelve, which is what it is for. The
//! recovering one keeps what it cannot structure as opaque bytes and carries
//! on, so the rest of the calendar survives.
//!
//! Whatever it recovers, nothing is lost: the recovered calendars serialize
//! back to the input byte for byte, and every problem worked around is
//! reported.

#![cfg(feature = "parser")]

mod common;

use ical::tree::cst::IcalCst;

/// How a corpus reads through the recovering parser.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Recovered {
    /// Fixtures the strict parser already accepts, recovered with no problem.
    clean: usize,
    /// Fixtures the strict parser refuses, recovered with problems reported.
    repaired: usize,
}

fn recover(corpus: &str, expected: usize) -> Recovered {
    let mut tally = Recovered::default();

    common::each_fixture(corpus, expected, |name, input| {
        let recovery = IcalCst::parse_recovering(input);

        // Nothing is ever lost, however broken the input was.
        assert_eq!(recovery.to_bytes(), input, "not byte-faithful: {name}");

        // And the recovered bytes are stable: reparsing them recovers the same.
        let output = recovery.to_bytes();
        let again = IcalCst::parse_recovering(&output);
        assert_eq!(again.to_bytes(), input, "not a fixpoint: {name}");
        assert_eq!(
            again.problems.len(),
            recovery.problems.len(),
            "problems are not stable: {name}"
        );

        // Decoding what was recovered must never panic.
        for cst in &recovery.calendars {
            let _ = cst.decode();
        }

        if recovery.is_clean() {
            tally.clean += 1;
        } else {
            tally.repaired += 1;
        }
    });

    tally
}

#[test]
fn libical_corpus_recovers() {
    assert_eq!(
        recover("libical", 40),
        Recovered {
            clean: 30,
            repaired: 10,
        }
    );
}

#[test]
fn ical4j_corpus_recovers() {
    assert_eq!(
        recover("ical4j", 104),
        Recovered {
            clean: 102,
            repaired: 2,
        }
    );
}

#[test]
fn icaljs_corpus_recovers() {
    assert_eq!(
        recover("icaljs", 46),
        Recovered {
            clean: 46,
            repaired: 0,
        }
    );
}

#[test]
fn rfc_corpus_recovers() {
    assert_eq!(
        recover("rfc", 7),
        Recovered {
            clean: 7,
            repaired: 0,
        }
    );
}
