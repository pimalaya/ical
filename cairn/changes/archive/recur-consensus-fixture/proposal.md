---
cairn: change
id: recur-consensus-fixture
status: landed
created: 2026-08-08
---

# Freeze the recurrence consensus as a committed fixture

## Why

The differential run against python-dateutil and libical is the crate's strongest evidence and it currently exists nowhere: the harness lived in a scratch directory. Every future change to the expander is therefore unguarded by it.

## What

Rebuild the harness, then commit the generated corpus together with the answer both oracles agreed on, as tests/corpus/recur/consensus.tsv (start, rule, occurrences), covering only the cases where dateutil and libical agree. Commit a second file for the settled divergences with this crate's own answer and a comment naming which oracle it parts from and why. A test replays both files, so CI gets the same signal without needing Python or a C toolchain, and every future change to the expander is a regression check against two real implementations.

Done when `cargo test` replays roughly 3,600 consensus cases and the divergence file documents each of the four settled behaviours.
