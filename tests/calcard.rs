//! Cross-implementation decode comparison against calcard.
//!
//! Two independent parsers read the same corpus, and each states the *shape* it
//! read: how many times every (component, property) pair occurs. Neither can
//! fake that, and it exercises the whole path (parse, structure, decode)
//! without pinning either crate's value modelling.
//!
//! Disagreement is expected and is not a failure by itself: calcard and ical-rs
//! take different liberties with malformed input, which is exactly the axis the
//! outcome counts measure. What the test pins is the counts, so a fixture
//! moving from agreement to disagreement has to be read as the behaviour change
//! it is.
//!
//! `VERSION` is dropped from both shapes: ical-rs lifts it out of the property
//! list into a typed indicator on the calendar, calcard leaves it in place, and
//! that is a modelling choice rather than a reading of the wire.

#![cfg(feature = "parser")]

mod common;

use std::collections::BTreeMap;

use calcard::{Entry, Parser};
use ical::{component::IcalComponent, tree::cst::IcalCst};

/// How many times each `COMPONENT/PROPERTY` pair occurs in a calendar.
type Shape = BTreeMap<String, usize>;

/// How a corpus reads through the two implementations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Cross {
    /// Both parse it and read the same shape.
    agreed: usize,
    /// Both parse it and read different shapes.
    differed: usize,
    /// Only ical-rs parses it.
    ours_only: usize,
    /// Only calcard parses it.
    theirs_only: usize,
    /// Neither parses it.
    neither: usize,
    /// Not valid UTF-8, so calcard cannot be handed it at all.
    skipped: usize,
}

fn cross(corpus: &str, expected: usize) -> Cross {
    let mut tally = Cross::default();

    common::each_fixture(corpus, expected, |_name, input| {
        let Ok(text) = str::from_utf8(input) else {
            tally.skipped += 1;
            return;
        };

        let ours = IcalCst::parse(input).ok().map(|cst| our_shape(&cst));
        let theirs = their_shape(text);

        match (ours, theirs) {
            (Some(ours), Some(theirs)) if ours == theirs => tally.agreed += 1,
            (Some(_), Some(_)) => tally.differed += 1,
            (Some(_), None) => tally.ours_only += 1,
            (None, Some(_)) => tally.theirs_only += 1,
            (None, None) => tally.neither += 1,
        }
    });

    tally
}

/// The shape ical-rs reads.
fn our_shape(cst: &IcalCst<'_>) -> Shape {
    let ical = cst.decode();
    let mut shape = Shape::new();

    for prop in &ical.props {
        count(&mut shape, "VCALENDAR", &prop.name);
    }

    for component in &ical.components {
        walk(&mut shape, component);
    }

    shape
}

fn walk(shape: &mut Shape, component: &IcalComponent<'_>) {
    for prop in &component.props {
        count(shape, &component.name, &prop.name);
    }

    for child in &component.components {
        walk(shape, child);
    }
}

/// The shape calcard reads, or `None` when it refuses the input.
fn their_shape(input: &str) -> Option<Shape> {
    let Entry::ICalendar(ical) = Parser::new(input).entry() else {
        return None;
    };

    let mut shape = Shape::new();

    for component in &ical.components {
        let name = component.component_type.as_str();
        for entry in &component.entries {
            count(&mut shape, name, entry.name.as_str());
        }
    }

    Some(shape)
}

/// Records one occurrence, skipping the `VERSION` the two models place
/// differently.
fn count(shape: &mut Shape, component: &str, prop: &str) {
    if prop.eq_ignore_ascii_case("VERSION") {
        return;
    }

    *shape
        .entry(format!(
            "{}/{}",
            component.to_ascii_uppercase(),
            prop.to_ascii_uppercase()
        ))
        .or_default() += 1;
}

#[test]
fn rfc_corpus_crosses() {
    assert_eq!(
        cross("rfc", 7),
        Cross {
            agreed: 7,
            ..Cross::default()
        }
    );
}

#[test]
fn libical_corpus_crosses() {
    assert_eq!(
        cross("libical", 40),
        Cross {
            agreed: 25,
            differed: 4,
            ours_only: 1,
            theirs_only: 4,
            neither: 2,
            skipped: 4,
        }
    );
}

#[test]
fn ical4j_corpus_crosses() {
    assert_eq!(
        cross("ical4j", 104),
        Cross {
            agreed: 100,
            differed: 1,
            ours_only: 0,
            theirs_only: 2,
            neither: 0,
            skipped: 1,
        }
    );
}

#[test]
fn icaljs_corpus_crosses() {
    assert_eq!(
        cross("icaljs", 46),
        Cross {
            agreed: 46,
            ..Cross::default()
        }
    );
}
