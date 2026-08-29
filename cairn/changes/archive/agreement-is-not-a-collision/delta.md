---
cairn: delta
change: agreement-is-not-a-collision
---

## ADDED Requirements

### Requirement: Agreement is not a collision

Two sides that made the same change SHALL NOT be reported as diverging, and the change SHALL land as an uncontested one. A collision is two people disagreeing, and two identical actions are not that.

#### Scenario: A side merged with itself

- GIVEN a base, and two sides holding the same edits of it
- WHEN they are merged
- THEN the merged calendar carries those edits and nothing is reported
