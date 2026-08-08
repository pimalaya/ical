---
cairn: delta
change: three-way-merge
---

## ADDED Requirements
### Requirement: Three-way merge against a stored base

Two divergent versions of a calendar SHALL reconcile against their common base, never by last-writer-wins. Every action taken and every conflict left SHALL be reported to the caller, and the merged calendar SHALL keep the untouched bytes of the left side.

#### Scenario: Two edits to different properties
- GIVEN a base event, a left version with a new summary and a right version with a new location
- WHEN they are merged
- THEN both edits survive and no conflict is reported

#### Scenario: Two edits to the same property
- GIVEN both versions setting a different summary
- WHEN they are merged
- THEN a conflict is reported rather than one side silently winning

### Requirement: Instance identity

A component SHALL be matched across versions by `UID` plus `RECURRENCE-ID`, so an override of one instance is never confused with the series it belongs to.

#### Scenario: An override beside its series
- GIVEN a series and an override sharing a `UID`
- WHEN a version edits only the override
- THEN the series is untouched and only the override merges

### Requirement: A series and its instances

A change to a series and a change to one of its instances SHALL both survive, and SHALL be reported together.

#### Scenario: A rule change against an instance change
- GIVEN one version changing the `RRULE` and the other changing an overriding instance's start
- WHEN they are merged
- THEN both changes are in the merged calendar and the pair is reported

### Requirement: Organiser authority

Where the caller says which calendar address the right side edits as, a right-side change to a property only the organiser may set SHALL be refused and reported (RFC 5546 3.2).

#### Scenario: An attendee moving a meeting
- GIVEN a right side speaking for an attendee, changing the start of a meeting someone else organises
- WHEN it is merged
- THEN the start does not change and the refusal is reported

## MODIFIED Requirements

### Requirement: A single text value is not split on its commas

A property whose value is one text SHALL keep an unescaped comma as data.

#### Scenario: An unescaped comma in a summary
- GIVEN `SUMMARY:Standup, moved`
- WHEN it is decoded
- THEN the summary reads `Standup, moved`
