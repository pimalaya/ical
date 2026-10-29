---
cairn: tasks
change: recurrence-benchmarks
---

- [x] Add `rrule` 0.14 as a dev-dependency and benches/recur.rs with the three groups
- [x] Check the cases give the same occurrences in both crates before timing them (a fast wrong answer is no comparison)
- [x] Time dateutil, libical and recurring-ical-events on the same cases through the harnesses
- [x] Profile the slowest group; fix only what the profile shows, numbers before and after
- [x] Re-run both corpora after any fix
- [x] Recurrence section in benches/README.md; log
