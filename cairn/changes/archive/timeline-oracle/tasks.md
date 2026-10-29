---
cairn: tasks
change: timeline-oracle
---

- [x] Prototype the harness (Rust runner, Python runner, cross) in a scratch directory
- [x] First run against recurring-ical-events 3.9.0 and triage every difference
- [x] Settle the first open decisions (landed as what-the-timeline-oracle-found)
- [x] Settle the second round (landed as timeline-oracle-second-round)
- [x] Vendor the v3.9.0 calendars under tests/corpus/rie/ with an ATTRIBUTION
- [x] Move the harness to tests/corpus/timeline/harness with a run.sh fetching the oracle from nixpkgs
- [x] Commit windows.tsv, agreement.tsv and divergence.tsv with an ATTRIBUTION
- [x] Add the replay test, asserting the counts and that every series is listed
- [x] Fold the delta into spec/recurrence.md, log, archive
