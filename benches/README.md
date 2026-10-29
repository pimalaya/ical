# Benchmarks

Single-calendar [criterion](https://crates.io/crates/criterion) medians, run with `cargo bench --bench parse` (source in [parse.rs](./parse.rs)), against ical 0.11, calcard 0.3 and icalendar 0.17. A dependency bump moves them, so re-run the bench after one.

The comparison is level-matched, so each group compares like with like: content-line parsers stop at a line tree like our `IcalCst::parse` step, while model parsers build a decoded object like our `parse + decode` step.

## Parsing into content lines (no decoding)

| library | time | delta |
| --- | --- | --- |
| **ical-rs** (`IcalCst::parse`) | **2.20 µs** | baseline |
| [`ical`](https://crates.io/crates/ical) (`PropertyParser`) | 4.81 µs | +119% |

## Parsing into a decoded model

| library | time | delta |
| --- | --- | --- |
| **ical-rs** (`parse + decode`) | **4.23 µs** | baseline |
| [`calcard`](https://crates.io/crates/calcard) | 4.60 µs | +9% |
| [`ical`](https://crates.io/crates/ical) (`IcalParser`) | 5.39 µs | +27% |
| [`icalendar`](https://crates.io/crates/icalendar) | 13.2 µs | +213% |

## Reading the numbers

These are a ballpark, not a strict ranking: the libraries produce different representations (borrowed versus owned, shallow versus validating), so they do different amounts of work.

The `ical` crate reads through a `BufRead`, which costs it a copy at both levels, and its `IcalParser` sits below the other model parsers since it nests components but leaves every value as a string, so it does less work than the line it is compared against.

At the model level `calcard` is the closest, within ten percent.

## Recurrence

[criterion](https://crates.io/crates/criterion) medians, run with `cargo bench --bench recur` (source in [recur.rs](./recur.rs)) on 2026-10-29, against rrule 0.14. Every group first checks both crates yield the same wall-clock times, so a fast wrong answer cannot win.

### The first 1,000 occurrences, as instants

Each rule starts on Monday 5 January 2026 at 09:00 in Europe/Paris. Before is the crate before the fix below.

| rule | **ical-rs** | before | [`rrule`](https://crates.io/crates/rrule) |
| --- | --- | --- | --- |
| `FREQ=DAILY` | **253 µs** | 891 µs | 127 µs |
| `FREQ=WEEKLY;BYDAY=MO,WE,FR` | **306 µs** | 1.35 ms | 121 µs |
| `FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1` | **1.31 ms** | 5.62 ms | 452 µs |
| `FREQ=YEARLY;BYMONTH=3,6,9,12;BYDAY=1MO` | **1.43 ms** | 8.39 ms | 489 µs |
| `FREQ=HOURLY;BYHOUR=9,12,17` | **393 µs** | 705 µs | 312 µs |

### One week of 2026 from a rule started in 1970

`FREQ=WEEKLY;BYDAY=MO,WE,FR`: ours skips to the window (`skip_to`), rrule walks to it (`after` plus `before`).

| library | time | before |
| --- | --- | --- |
| **ical-rs** | **61.6 µs** | 382 µs |
| [`rrule`](https://crates.io/crates/rrule) | 1.06 ms | |

### A timeline walk

Every series of tests/corpus/rie/fablab_cottbus.ics over 2017 to 2022, tracked against ourselves (no Rust counterpart): **56.1 µs**, 67.5 µs before.

### What the profile showed

A yearly period was scanned day by day (365 conversions), which also dominated every zoned walk through its `VTIMEZONE` rules; periods are now walked a month at a time. About half of what remains is `IcalTzTransitions` materialising a zone's history per walk.

### Across languages, a ballpark

Timed by [ballpark/run.sh](./ballpark/run.sh) through nix. Call overhead, FFI and zone handling differ: read the order of magnitude only.

| case | ical-rs | python-dateutil 2.9.0 | libical 3.0.20 |
| --- | --- | --- | --- |
| daily | 253 µs | 3.03 ms | 1.09 ms |
| weekly `BYDAY` | 306 µs | 2.86 ms | 1.14 ms |
| monthly `BYSETPOS` | 1.31 ms | 5.56 ms | 4.33 ms |
| yearly `BYMONTH` `BYDAY` | 1.43 ms | 8.10 ms | 3.28 ms |
| hourly `BYHOUR` | 393 µs | 3.94 ms | 1.02 ms |
| one week from 1970 | 61.6 µs | 8.02 ms | |

The timeline walk of fablab_cottbus.ics takes 9.72 ms through recurring-ical-events 3.9.0, against 56.1 µs here.
