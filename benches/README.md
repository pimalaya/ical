# Benchmarks

Single-calendar [criterion](https://crates.io/crates/criterion) medians, run with `cargo bench --bench parse` (source in [parse.rs](./parse.rs)), against ical 0.11, calcard 0.3 and icalendar 0.17. A dependency bump moves them, so re-run the bench after one.

The comparison is level-matched, so each group compares like with like: content-line parsers stop at a line tree like our `IcalCst::parse` step, while model parsers build a decoded object like our `parse + decode` step.

The figures below are the medians of three interleaved runs on 2026-11-03 after the second speed pass (Zen 5, rustc 1.99.1, `taskset -c 12`, the bench executables on ext4) on a quiet core. Earlier figures were taken on other days and toolchains: compare deltas, not times, across them.

## Parsing into content lines (no decoding)

| library | time | delta |
| --- | --- | --- |
| **ical-rs** (`IcalCst::parse`) | **0.54 µs** | baseline |
| [`ical`](https://crates.io/crates/ical) (`PropertyParser`) | 2.73 µs | +406% |

## Parsing into a decoded model

| library | time | delta |
| --- | --- | --- |
| **ical-rs** (`parse + decode`) | **1.04 µs** | baseline |
| [`calcard`](https://crates.io/crates/calcard) | 2.94 µs | +183% |
| [`ical`](https://crates.io/crates/ical) (`IcalParser`) | 3.31 µs | +219% |
| [`icalendar`](https://crates.io/crates/icalendar) | 9.81 µs | +846% |

## Reading the numbers

These are a ballpark, not a strict ranking: the libraries produce different representations (borrowed versus owned, shallow versus validating), so they do different amounts of work.

The `ical` crate reads through a `BufRead`, which costs it a copy at both levels, and its `IcalParser` sits below the other model parsers since it nests components but leaves every value as a string, so it does less work than the line it is compared against.

At the model level `calcard` is the closest, at nearly three times our time.

## This crate's pipeline

The same calendar through the bench's own rows, before and after the first speed pass ([the-lines-built-in-place](../cairn/log/2026-11-03-the-lines-built-in-place.md)), interleaved in the same session:

| step | before | after | delta |
| --- | ---: | ---: | ---: |
| `IcalCst::parse` | 1,607 ns | 818 ns | -49% |
| parse + `decode` | 2,667 ns | 1,528 ns | -43% |
| `decode` of a parsed calendar | 896 ns | 601 ns | -33% |
| `encode` of a decoded calendar | 2,509 ns | 2,154 ns | -14% |
| `to_string` of a decoded calendar | 4,292 ns | 326 ns | -92% |
| `to_bytes` | 393 ns | 210 ns | -47% |
| parse + `to_bytes` | 2,059 ns | 1,091 ns | -47% |

The rivals moved within 4% between the two sides.

The second pass ([the-parts-written-in-place](../cairn/log/2026-11-03-the-parts-written-in-place.md)), its second session interleaved with the tree its first session left (583be79), three runs each:

| step | before | after | delta |
| --- | ---: | ---: | ---: |
| `IcalCst::parse` | 620 ns | 539 ns | -13% |
| parse + `decode` | 1,223 ns | 1,037 ns | -15% |
| `decode` of a parsed calendar | 524 ns | 422 ns | -19% |
| `encode` of a decoded calendar | 2,133 ns | 1,565 ns | -27% |
| `to_string` of a decoded calendar | 345 ns | 313 ns | -9% |
| `to_bytes` | 184 ns | 181 ns | -2% |
| parse + `to_bytes` | 796 ns | 741 ns | -7% |

The first session's part shows between the two tables (`IcalCst::parse` 818 ns after the first pass, 620 ns before the second session), from runs of the same day. The rivals moved within 3% between the two sides.

## Cycles over the corpus

Criterion times one short calendar. The speed passes also count user-space cycles over tests/corpus (293 files, 3.4 MB, 2,881 calendars) and over the same short calendar parsed 300 times, with `perf stat`, which another process's load moves far less than wall time. The harness lives in [the speed-pass handoff](../cairn/changes/the-structure-scanned-once/proposal.md).

The first pass, medians of 11 interleaved runs a pass on a quiet core:

| workload | before | after | delta |
| --- | ---: | ---: | ---: |
| corpus `parse` | 57,766k | 34,257k | -41% |
| corpus `parse` + `decode` | 94,339k | 60,926k | -35% |
| corpus `parse` + `to_bytes` | 68,421k | 41,665k | -39% |
| corpus `to_string` of the decoded calendars | 126,519k | 12,071k | -90% |
| short calendar `parse` | 2,145k | 1,097k | -49% |
| short calendar `parse` + `decode` | 3,723k | 1,951k | -48% |
| short calendar `parse` + `to_bytes` | 2,705k | 1,391k | -49% |
| short calendar `to_string` | 5,717k | 479k | -92% |

`to_bytes` of a large calendar parsed long before, its tree out of cache, is the one row that lost (+24% over the corpus): see the pass's log.

The second pass ([the-parts-written-in-place](../cairn/log/2026-11-03-the-parts-written-in-place.md)), layout 0 medians of 7 interleaved runs, the delta the mean over four link layouts:

| workload | before | after | delta |
| --- | ---: | ---: | ---: |
| corpus `parse` | 34,409k | 24,224k | -29% |
| corpus `parse` + `decode` | 61,085k | 43,676k | -29% |
| corpus `parse` + `to_bytes` | 41,437k | 30,456k | -27% |
| corpus `to_string` of the decoded calendars | 12,088k | 11,387k | -7% |
| corpus `encode` of the decoded calendars | 67,430k | 54,214k | -20% |
| corpus `to_string` of the parsed calendars | 48,683k | 25,169k | -49% |
| short calendar `parse` | 1,105k | 778k | -29% |
| short calendar `parse` + `decode` | 1,969k | 1,503k | -24% |
| short calendar `parse` + `to_bytes` | 1,397k | 1,024k | -26% |
| short calendar `to_string` of the parsed calendar | 2,054k | 737k | -64% |

`to_bytes` of calendars parsed long before, the first pass's one loss, is 11% faster again.

## Content decoding

Cycles to decode, with the SIMD codecs the `base64` and `quoted-printable` features run on since [the-simd-codecs](../cairn/log/2026-11-03-the-simd-codecs.md), before the switch (base64 0.23, quoted_printable 0.5), after it with no CPU token installed, and after it with `SimdCpu::install_detected()` called first, medians of 7 interleaved runs on a quiet core:

| decoding | before | no token | detected token |
| --- | ---: | ---: | ---: |
| a folded inline `ATTACH` of 36 KiB | 47.3k | 41.1k | 2.0k |
| a 4.4 KB `QUOTED-PRINTABLE` description | 29.9k | 7.1k | 3.2k |
| the same through `charset` | 31.1k | 8.1k | 4.2k |
| 300 inline payloads of 48 bytes | 43.5k | 34.9k | 22.7k |
| 300 one-paragraph `QUOTED-PRINTABLE` values | 460.7k | 104.1k | 65.1k |

An application that never installs the token runs the codecs' portable code: install it once, first thing in `main`.

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
