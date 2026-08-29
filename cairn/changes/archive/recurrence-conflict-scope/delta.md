---
cairn: delta
change: recurrence-conflict-scope
---

## MODIFIED Requirements

### Requirement: A series and its instances

A change to what defines a series and a change to one of its instances SHALL both survive, and SHALL be reported together: a rule that moved may have moved the ground the override stood on, and only the caller can know whether that matters.

What defines the series is its `DTSTART`, `DTEND`, `DURATION`, `RRULE`, `RDATE` and `EXDATE`, and the series component itself. A change to anything else the series carries cannot have moved an occurrence, and SHALL NOT be reported against one.

#### Scenario: A rule change against an instance change

- GIVEN one version changing the `RRULE` and the other changing an overriding instance's start
- WHEN they are merged
- THEN both changes are in the merged calendar and the pair is reported

#### Scenario: A description change against an instance change

- GIVEN one version changing the series' `LOCATION` and the other changing an overriding instance's summary
- WHEN they are merged
- THEN both changes are in the merged calendar and nothing is reported
