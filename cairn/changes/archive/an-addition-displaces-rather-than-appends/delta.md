---
cairn: delta
change: an-addition-displaces-rather-than-appends
---

## ADDED Requirements

### Requirement: An addition that wins replaces the one it beat

Where both sides added a property or a component the base lacked and the merge keeps the replayed side's, the addition it beat SHALL be taken out rather than left beside it. The merged calendar SHALL never hold more members of a group than the side that wrote the most, so a property RFC 5545 allows once is never emitted twice and `validate` never refuses what the merge produced.

Merging two byte-identical sides SHALL therefore return those bytes under either preference.

#### Scenario: Both sides setting a location the base lacked

- GIVEN a base with no `LOCATION` and two sides adding a different one, with the right side preferred
- WHEN they are merged
- THEN the merged event holds the right side's `LOCATION` alone and the collision is reported
