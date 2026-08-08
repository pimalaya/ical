---
cairn: delta
change: test-surface
---

## ADDED Requirements

### Requirement: Every property belongs to a version

Each property SHALL state the versions that define it, and validation SHALL report one written in a version that does not. The legacy vCalendar 1.0 alarm properties belong to 1.0 alone; every property an extension RFC adds belongs to iCalendar 2.0 alone.

#### Scenario: An extension property in a vCalendar 1.0 file
- GIVEN `COLOR` (RFC 7986) in a calendar whose version is 1.0
- WHEN the calendar is validated
- THEN the property is reported as one the version does not define

### Requirement: The spec dispatch answers for the property it is asked about

The runtime bridge from a property kind to its static spec SHALL carry the kind it describes, and that kind SHALL be the one it was dispatched from.

#### Scenario: A marker under the wrong arm
- GIVEN the seventy-arm dispatch
- WHEN a marker's `KIND` does not match the arm it sits in
- THEN the mismatch is reported

## MODIFIED Requirements

### Requirement: Liberal rule parsing

An unrecognised rule part SHALL be ignored rather than refused, and a malformed value inside a part the module claims to understand SHALL be an error. A rule that breaks a constraint of RFC 5545 3.3.10 SHALL still parse: `UNTIL` and `COUNT` together are a validation problem, not a parse error, since strictness belongs on the way out.

#### Scenario: A rule bounded twice
- GIVEN `FREQ=DAILY;COUNT=10;UNTIL=20260101T000000Z`
- WHEN it is parsed
- THEN it parses, and validation reports the two bounds
