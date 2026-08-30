//! Parse and serialize benchmarks, comparing like with like by level.
//!
//! `parse_to_content_lines` pits this crate's byte-faithful CST against the
//! `ical` crate's `PropertyParser`, which also stops at content lines with no
//! value decoding.
//!
//! `parse_to_model` pits our parse plus decode (to the `Ical` model, not the
//! validated `IcalValid<Ical>`) against the eager model parsers `calcard` and
//! `icalendar`, plus the `ical` crate's `IcalParser`, which builds a component
//! tree but leaves values as strings.
//!
//! Representations still differ in laziness, ownership and decoding depth, so
//! read these as ballpark rather than a strict ranking.

use std::{hint::black_box, io::BufReader};

use criterion::{Criterion, criterion_group, criterion_main};

use ical::tree::cst::IcalCst;

/// A realistic iCalendar 2.0 calendar with a nested `VEVENT` and `VALARM`.
const CAL: &str = concat!(
    "BEGIN:VCALENDAR\r\n",
    "VERSION:2.0\r\n",
    "PRODID:-//Example Corp//Calendar 1.0//EN\r\n",
    "CALSCALE:GREGORIAN\r\n",
    "METHOD:REQUEST\r\n",
    "BEGIN:VEVENT\r\n",
    "UID:19970901T130000Z-123401@example.com\r\n",
    "DTSTAMP:19970901T130000Z\r\n",
    "DTSTART:19970903T163000Z\r\n",
    "DTEND:19970903T190000Z\r\n",
    "SUMMARY:Annual Employee Review\r\n",
    "DESCRIPTION:Come prepared with your goals for the year\\, and a coffee.\r\n",
    "CATEGORIES:BUSINESS,HUMAN RESOURCES\r\n",
    "ORGANIZER;CN=Jane Boss:mailto:jane@example.com\r\n",
    "ATTENDEE;ROLE=REQ-PARTICIPANT;RSVP=TRUE;CN=Joe:mailto:joe@example.com\r\n",
    "RRULE:FREQ=YEARLY;COUNT=5\r\n",
    "BEGIN:VALARM\r\n",
    "ACTION:DISPLAY\r\n",
    "DESCRIPTION:Review reminder\r\n",
    "TRIGGER:-PT1H\r\n",
    "END:VALARM\r\n",
    "END:VEVENT\r\n",
    "END:VCALENDAR\r\n",
);

/// Content-line level: a lazy, byte-faithful split, no value decoding. Compared
/// against the `ical` crate's property parser, which works at the same level.
fn parse_to_content_lines(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_to_content_lines");

    group.bench_function("ical-rs: IcalCst::parse", |b| {
        b.iter(|| IcalCst::parse(black_box(CAL)).unwrap())
    });
    group.bench_function("ical: PropertyParser", |b| {
        b.iter(|| {
            let lines = ical_crate::LineReader::new(BufReader::new(black_box(CAL).as_bytes()));

            ical_crate::PropertyParser::new(lines)
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        })
    });

    group.finish();
}

/// Decoded-model level: full parse into a typed model. Ours is parse + decode
/// into `Ical` (not the validated `IcalValid<Ical>`), compared against the
/// eager model parsers.
fn parse_to_model(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_to_model");

    group.bench_function("ical-rs: parse + decode", |b| {
        b.iter(|| {
            let cst = IcalCst::parse(black_box(CAL)).unwrap();
            black_box(cst.decode());
        })
    });
    group.bench_function("calcard", |b| {
        b.iter(|| black_box(calcard::icalendar::ICalendar::parse(black_box(CAL))))
    });
    group.bench_function("icalendar", |b| {
        b.iter(|| black_box(black_box(CAL).parse::<icalendar::Calendar>()))
    });
    // NOTE: shallower than the others, values stay as strings.
    group.bench_function("ical: IcalParser", |b| {
        b.iter(|| {
            ical_crate::IcalParser::new(BufReader::new(black_box(CAL).as_bytes()))
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        })
    });

    group.finish();
}

/// This crate's own decode/encode pipeline.
fn ical_rs_pipeline(c: &mut Criterion) {
    let cst = IcalCst::parse(CAL).unwrap();
    let cal = cst.decode();

    c.bench_function("ical-rs: decode", |b| {
        b.iter(|| black_box(black_box(&cst).decode()))
    });
    c.bench_function("ical-rs: encode", |b| {
        b.iter(|| black_box(black_box(&cal).encode()))
    });
    c.bench_function("ical-rs: to_bytes", |b| {
        b.iter(|| black_box(black_box(&cst).to_bytes()))
    });
    c.bench_function("ical-rs: round-trip (parse + to_bytes)", |b| {
        b.iter(|| IcalCst::parse(black_box(CAL)).unwrap().to_bytes())
    });
}

criterion_group!(
    benches,
    parse_to_content_lines,
    parse_to_model,
    ical_rs_pipeline
);
criterion_main!(benches);
