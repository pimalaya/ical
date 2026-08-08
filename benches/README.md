# Benchmarks

Single-calendar [criterion](https://crates.io/crates/criterion) medians, run with `cargo bench --bench parse` (source in [parse.rs](./parse.rs)).

The comparison is level-matched, so each group compares like with like: content-line parsers stop at a line tree like our `IcalCst::parse` step, while model parsers build a decoded object like our `parse + decode` step.

## Parsing into content lines (no decoding)

| library | time | delta |
| --- | --- | --- |
| **ical-rs** (`IcalCst::parse`) | **2.33 µs** | baseline |
| [`ical`](https://crates.io/crates/ical) (`PropertyParser`) | 4.93 µs | +111% |

## Parsing into a decoded model

| library | time | delta |
| --- | --- | --- |
| **ical-rs** (`parse + decode`) | **4.44 µs** | baseline |
| [`calcard`](https://crates.io/crates/calcard) | 4.53 µs | +2% |
| [`ical`](https://crates.io/crates/ical) (`IcalParser`) | 5.63 µs | +27% |
| [`icalendar`](https://crates.io/crates/icalendar) | 13.5 µs | +203% |

## Reading the numbers

These are a ballpark, not a strict ranking: the libraries produce different representations (borrowed versus owned, shallow versus validating), so they do different amounts of work. The `ical` crate reads through a `BufRead`, which costs it a copy at both levels, and its `IcalParser` sits below the other model parsers since it nests components but leaves every value as a string, so it does less work than the line it is compared against. At the model level we are on par with `calcard`.
