---
cairn: delta
change: a-conflict-names-its-sides
---

## ADDED Requirements

### Requirement: A conflict names its two sides
*Folds into merge.md.*

A reported conflict SHALL carry one field per side, named `left` and `right` after the sides themselves, in that order. The left field SHALL carry the left side's action together with the kind of conflict it is, and the right field the action the right side wanted.

vcard-rs states the same contract and names its pair the same way. A caller holding both crates reads one shape, and a field named after a side cannot be read as a description of the other one.

The kind stays where it is. iCalendar has a conflict vCard has no counterpart for, a change to a series meeting a change to one of its instances, so the left field is an enum naming which of the two kinds this is rather than a bare action.

#### Scenario: Reading a divergence

- GIVEN both versions changing one property to different things
- WHEN the conflict is read
- THEN its left field carries the left side's action as a divergence and its right field the right side's action

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
