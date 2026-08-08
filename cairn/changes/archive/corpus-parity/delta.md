---
cairn: delta
change: corpus-parity
---

## ADDED Requirements
### Requirement: The real-world corpus is swept

Every committed real-world fixture SHALL parse, serialize to a fixpoint, decode without panicking, and survive a decode, encode and decode again unchanged. Each source directory SHALL carry its own attribution and an asserted fixture count, so a misfiled, renamed or newly added fixture is caught.

#### Scenario: A vendor calendar the parser cannot structure
- GIVEN a fixture the strict parser refuses
- WHEN the sweep runs
- THEN the fixture is classified as refused, not silently skipped
