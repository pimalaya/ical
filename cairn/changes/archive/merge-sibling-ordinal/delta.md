---
cairn: delta
change: merge-sibling-ordinal
---

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Instance identity

A component SHALL be matched across versions by `UID` plus `RECURRENCE-ID`, so an override of one instance is never confused with the series it belongs to, however the two are ordered in the file. A component carrying no `UID` SHALL be matched by its position among its same-named siblings, and that position SHALL be counted the same way wherever the merge counts it: differently-named children do not shift each other.

#### Scenario: An override beside its series

- GIVEN a series and an override sharing a `UID`
- WHEN a version edits only the override, and writes it before the series
- THEN the series is untouched and only the override merges

#### Scenario: A change to a component that is not the first child

- GIVEN a `VTIMEZONE` whose `STANDARD` is written before its `DAYLIGHT`
- WHEN one version changes a property of the `DAYLIGHT`
- THEN the change is in the merged calendar

## REMOVED Requirements

None.
