---
cairn: tasks
change: merge-idempotence-regression
---

- [ ] tests/merge.rs: the 225-byte input of proposal.md inlined (from the hex), split on NUL; base, left, left merges to left with no conflict; the fuzz target's other oracles on base, left, right
- [ ] Check it fails on fb4011b (before caa1422) and passes now
- [ ] Re-run the merge fuzz target with real time now that its corpus replays (s3-report finding 9: it got none)
- [ ] Log entry (no capability moves)
