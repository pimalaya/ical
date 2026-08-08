//! Round-trip robustness sweep over the corpus.
//!
//! Every fixture must parse, serialize to a byte-stable fixpoint (its own output
//! reparses identically), and decode without panicking. This is the crate's
//! central guarantee: byte-faithful parsing of any real calendar.
//!
//! The hand-written corpora (rfc, vcalendar) assert byte-identity outright. The
//! vendored real-world corpora (libical, ical4j, icaljs) are *classified*
//! instead, one count per outcome, and the counts are asserted: a fixture that
//! moves from one outcome to another is a change in behaviour and has to be
//! read as one, whichever direction it moves in.

#![cfg(feature = "parser")]

mod common;

use ical::tree::cst::IcalCst;

use crate::common::Tally;

fn round_trips(name: &str, input: &[u8]) {
    let cst = IcalCst::parse(input).unwrap_or_else(|e| panic!("parse {name}: {e}"));

    // Byte-faithful: the fixture serializes back to itself.
    assert_eq!(cst.to_bytes(), input, "not byte-faithful: {name}");

    // Fixpoint: the serialized bytes reparse to the same bytes.
    let bytes = cst.to_bytes();
    let reparsed = IcalCst::parse(&bytes).unwrap_or_else(|e| panic!("reparse {name}: {e}"));
    assert_eq!(reparsed.to_bytes(), bytes, "not idempotent: {name}");

    // Decoding must never panic.
    let _ = cst.decode();
}

#[test]
fn rfc_corpus_round_trips() {
    common::each_fixture("rfc", 7, round_trips);
}

#[test]
fn vcalendar_corpus_round_trips() {
    common::each_fixture("vcalendar", 1, round_trips);
}

#[test]
fn libical_corpus_classifies() {
    assert_eq!(
        common::classify_corpus("libical", 40),
        Tally {
            identical: 29,
            normalised: 0,
            refused: 10,
            empty: 1,
        }
    );
}

#[test]
fn ical4j_corpus_classifies() {
    assert_eq!(
        common::classify_corpus("ical4j", 104),
        Tally {
            identical: 102,
            normalised: 0,
            refused: 2,
            empty: 0,
        }
    );
}

#[test]
fn icaljs_corpus_classifies() {
    assert_eq!(
        common::classify_corpus("icaljs", 46),
        Tally {
            identical: 46,
            normalised: 0,
            refused: 0,
            empty: 0,
        }
    );
}
