---
cairn: tasks
change: recur-consensus-fixture
---

- [x] Rebuild the case generator (frequencies, BY parts, interacting pairs, seeded randoms, edge starts, bounds)
- [x] Rebuild the three runners: ical-rs, dateutil, libical, each with a per-case alarm
- [x] Cross the three answers, keep the cases where both oracles agree
- [x] Commit tests/corpus/recur/consensus.tsv with its ATTRIBUTION
- [x] Commit tests/corpus/recur/divergence.tsv covering the four settled behaviours
- [x] Add the replay test and assert the case counts
