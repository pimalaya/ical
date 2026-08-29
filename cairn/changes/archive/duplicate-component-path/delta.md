---
cairn: delta
change: duplicate-component-path
---

## ADDED Requirements

### Requirement: Two components at one path

Where a calendar holds several components at one path, a `UID` written twice with no `RECURRENCE-ID` telling them apart, each component of one side SHALL be matched with at most one component of the other. Comparing two of them with the same one would report the difference between the duplicates as a change a side made.

#### Scenario: A calendar holding one UID twice

- GIVEN a calendar with two events sharing a `UID`, edited once
- WHEN it is merged with itself against the original
- THEN the merged calendar is that edit and nothing is reported
