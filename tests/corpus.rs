//! Round-trip robustness sweep over the corpus.
//!
//! Every fixture must parse, serialize to a byte-stable fixpoint (its own output
//! reparses identically), and decode without panicking. This is the crate's
//! central guarantee: byte-faithful parsing of any real calendar.

#![cfg(feature = "parser")]

mod common;

use ical::tree::cst::IcalCst;

fn round_trips(name: &str, input: &str) {
    let cst = IcalCst::parse(input).unwrap_or_else(|e| panic!("parse {name}: {e}"));

    // Byte-faithful: a clean (unfolded) fixture serializes back to itself.
    assert_eq!(cst.to_string(), input, "not byte-faithful: {name}");

    // Fixpoint: the serialized bytes reparse to the same bytes.
    let bytes = cst.to_bytes();
    let reparsed = IcalCst::parse(&bytes).unwrap_or_else(|e| panic!("reparse {name}: {e}"));
    assert_eq!(reparsed.to_bytes(), bytes, "not idempotent: {name}");

    // Decoding must never panic.
    let _ = cst.decode();
}

#[test]
fn rfc_corpus_round_trips() {
    common::each_fixture("rfc", 6, round_trips);
}

#[test]
fn vcalendar_corpus_round_trips() {
    common::each_fixture("vcalendar", 1, round_trips);
}
