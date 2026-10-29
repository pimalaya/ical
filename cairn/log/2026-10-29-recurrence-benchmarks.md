---
cairn: log
change: recurrence-benchmarks
date: 2026-10-29
---

# Recurrence benchmarks

- benches/recur.rs against rrule 0.14 (dev-dependency), each group checked for equal answers first; benches/ballpark/run.sh times dateutil, libical and recurring-ical-events.
- Callgrind (perf is refused here) showed 365 `civil_from_days` per yearly period, which also dominated every zoned walk through its `VTIMEZONE` rules. Periods are now walked a month at a time: yearly 8.39 ms to 1.43 ms, daily 891 µs to 253 µs, far window 382 µs to 61.6 µs (full table in benches/README.md). Both corpora unchanged.
- Next candidate, not touched: `IcalTzTransitions` re-materialising a zone's history, about half of each case.

No capability moved.
