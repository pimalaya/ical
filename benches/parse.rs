//! Parse/serialize benchmarks for the iCalendar pipeline.
//!
//! `ical_rs_pipeline` measures this crate's own stages over a realistic
//! `VCALENDAR` with a nested `VEVENT` and `VALARM`: parsing to the byte-faithful
//! CST, decoding onto the model, encoding back to bytes, and a full round-trip.

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use ical::tree::cst::IcalCst;

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

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("ical_rs_pipeline");

    group.bench_function("parse", |b| {
        b.iter(|| IcalCst::parse(black_box(CAL)).unwrap())
    });

    group.bench_function("parse_and_decode", |b| {
        b.iter(|| {
            let cst = IcalCst::parse(black_box(CAL)).unwrap();
            black_box(cst.decode().components.len())
        })
    });

    let cst = IcalCst::parse(CAL).unwrap();
    group.bench_function("to_bytes", |b| b.iter(|| black_box(&cst).to_bytes()));

    group.bench_function("round_trip", |b| {
        b.iter(|| {
            let cst = IcalCst::parse(black_box(CAL)).unwrap();
            let bytes = cst.to_bytes();
            IcalCst::parse(&bytes).unwrap().to_bytes()
        })
    });

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
