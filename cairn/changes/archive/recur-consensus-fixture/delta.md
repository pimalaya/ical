---
cairn: delta
change: recur-consensus-fixture
---

## ADDED Requirements
### Requirement: The consensus corpus is replayed

Expansion SHALL be pinned by a committed corpus of cases on which python-dateutil and libical agree with each other, replayed by the test suite with no Python and no C toolchain present. A second committed corpus SHALL pin each deliberate divergence with this crate's own answer and the oracle it parts from.

#### Scenario: A regression in the expander
- GIVEN a change to the expander that alters an agreed-upon answer
- WHEN the test suite runs
- THEN the consensus replay fails, naming the rule and the start
