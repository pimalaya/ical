//! Shared corpus harness for the round-trip integration tests.
//!
//! Each corpus is a directory under tests/corpus/ (provenance and licensing
//! live in its ATTRIBUTION.md). It is a robustness harness, not a golden-output
//! suite: the fixtures are real-world vCalendar 1.0 / iCalendar 2.0 objects plus
//! the RFC 5545 examples. The one version-agnostic parser handles them all, so
//! each test sweeps the whole corpus and asserts every fixture parses,
//! serializes to a fixpoint (stable under reparse) and decodes without
//! panicking.
//!
//! Fixtures are read as bytes, never as text: a real calendar may carry a value
//! in a foreign charset, which the parser keeps and a `String` would not.

// Each integration test compiles this module separately and uses only the part
// of it that it needs.
#![allow(dead_code)]

use std::{fs, path::PathBuf};

use ical::tree::cst::IcalCst;

/// What a fixture does when it is parsed and serialized straight back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Serializes back to the exact input bytes.
    Identical,
    /// Parses, but the output differs from the input somewhere the parser
    /// resolves the wire shape (folding, blank lines, QUOTED-PRINTABLE soft
    /// breaks) without restoring it.
    Normalised,
    /// The strict parser refuses it.
    Refused,
    /// Holds no content line at all.
    Empty,
}

/// How a whole corpus classifies, one count per outcome.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    /// Fixtures that come back byte for byte.
    pub identical: usize,
    /// Fixtures whose wire shape the parser resolved away.
    pub normalised: usize,
    /// Fixtures the strict parser refuses.
    pub refused: usize,
    /// Fixtures with no content line.
    pub empty: usize,
}

/// Runs `check` against every `.ics` fixture of `corpus`, asserting exactly
/// `expected` of them are present so a misfiled, renamed or newly added fixture
/// is caught. `check` receives the fixture name and its raw bytes.
pub fn each_fixture(corpus: &str, expected: usize, mut check: impl FnMut(&str, &[u8])) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus")
        .join(corpus);

    let mut total = 0;

    for entry in fs::read_dir(&dir).expect("read corpus dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("ics") {
            continue;
        }

        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let input = fs::read(&path).expect("read fixture");

        total += 1;
        check(&name, &input);
    }

    assert_eq!(
        total, expected,
        "expected {expected} fixtures in {corpus}, found {total}"
    );
}

/// Sweeps a corpus of real-world calendars and classifies each fixture.
///
/// Whatever the outcome, a fixture that parses must serialize to a fixpoint,
/// decode without panicking, and survive a decode, encode and decode again
/// unchanged. Only the byte-identity of the first serialization varies, which
/// is what the returned [`Tally`] counts.
pub fn classify_corpus(corpus: &str, expected: usize) -> Tally {
    let mut tally = Tally::default();

    each_fixture(corpus, expected, |name, input| {
        match classify(name, input) {
            Outcome::Identical => tally.identical += 1,
            Outcome::Normalised => tally.normalised += 1,
            Outcome::Refused => tally.refused += 1,
            Outcome::Empty => tally.empty += 1,
        };
    });

    tally
}

/// Classifies one fixture, asserting every guarantee that does not depend on
/// its outcome.
pub fn classify(name: &str, input: &[u8]) -> Outcome {
    if input.iter().all(|byte| byte.is_ascii_whitespace()) {
        return Outcome::Empty;
    }

    let Some(output) = parse_whole(input) else {
        return Outcome::Refused;
    };

    // Whatever it looked like on the wire, the output must reparse to itself.
    let reparsed = parse_whole(&output).unwrap_or_else(|| panic!("reparse {name}"));
    assert_eq!(reparsed, output, "not a serialize fixpoint: {name}");

    // Decoding must never panic, and the model must survive a round-trip
    // through the syntax tree.
    for cst in calendars(input).expect("already parsed") {
        let encoded = IcalCst::from(cst.decode());
        let redecoded = encoded.decode();
        assert_eq!(
            IcalCst::from(redecoded).to_bytes(),
            encoded.to_bytes(),
            "decode is not stable: {name}"
        );
    }

    if output == input {
        Outcome::Identical
    } else {
        Outcome::Normalised
    }
}

/// Every top-level calendar of a file, in order, or `None` when the file cannot
/// be structured. A file holding no `BEGIN` at all is read as one bare,
/// envelope-less record.
fn calendars(input: &[u8]) -> Option<Vec<IcalCst<'_>>> {
    let mut all = Vec::new();

    for result in IcalCst::parse_many(input) {
        match result {
            Ok(cst) => all.push(cst),
            Err(_) if all.is_empty() => return IcalCst::parse(input).ok().map(|cst| vec![cst]),
            Err(_) => return None,
        }
    }

    Some(all)
}

/// The whole file, parsed and serialized straight back.
fn parse_whole(input: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();

    for cst in calendars(input)? {
        out.extend_from_slice(&cst.to_bytes());
    }

    Some(out)
}
