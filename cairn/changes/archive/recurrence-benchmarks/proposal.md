---
cairn: change
id: recurrence-benchmarks
status: landed
created: 2026-10-29
---

# Recurrence benchmarks

## Why

The claims about recurrence speed are about worst cases (bounded work per `next`, `skip_to` jumping to a window, the recur corpus expanding in about 75 s where both oracles need a 2 s alarm per case). Throughput on ordinary rules has never been measured, against anyone. benches/ holds only the parser bench, so "fast" is not a claim this crate can make yet.

## What

1. A criterion bench, benches/recur.rs, level-matched like benches/parse.rs:
   - rule expansion only, against the `rrule` crate 0.14 (dev-dependency, as the parser bench uses ical, calcard and icalendar): the first 1,000 occurrences of a handful of common rules (daily, weekly `BYDAY`, monthly `BYSETPOS`, yearly `BYMONTH`+`BYDAY`, an hourly rule with `BYHOUR`) on a zoned start;
   - a window far from `DTSTART` (one week in 2026 of a rule starting 1970), against `rrule`'s `between`;
   - a timeline walk over a real calendar from tests/corpus/rie (no Rust counterpart; tracked against ourselves).
2. A cross-language ballpark, outside criterion: time the same cases through dateutil, libical and recurring-ical-events with the existing harnesses, and report them in a separate table marked as a ballpark (interpreter start-up, FFI and output cost differ).
3. Profile before touching code (`perf` through nix). Fix only a hot spot a profile shows, each with its before and after numbers in the log. Candidates to check, not to assume: allocation per occurrence in `IcalRecurExpand`, zone lookups per occurrence in the timeline, the `SLACK` widening of the window.
4. benches/README.md gains a recurrence section in the parser section's shape.

## Consequence

A new dev-dependency (`rrule`, which brings chrono and chrono-tz into dev builds only). No behaviour change; a speed fix that changes no answer is checked by both corpora.
