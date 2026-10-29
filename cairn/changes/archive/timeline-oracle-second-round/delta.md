---
cairn: delta
change: timeline-oracle-second-round
---

## ADDED Requirements
### Requirement: A series cut before its start has no instance

In [recurrence](../../../spec/recurrence.md), under the recurrence set of a component. When the set has rules and every one ends on a day before its `DTSTART`, the `DTSTART` SHALL NOT be an instance; the `RDATE`s still stand.

#### Scenario: A series cut before its start
- GIVEN `DTSTART;VALUE=DATE:20231002` and `RRULE:FREQ=WEEKLY;UNTIL=20231001;INTERVAL=2;BYDAY=MO`
- WHEN its set is expanded
- THEN it yields nothing

## MODIFIED Requirements
### Requirement: Liberal rule parsing

In [recurrence](../../../spec/recurrence.md). A malformed value inside a part the module understands stays an error of `IcalRecurRule::parse`; reading a component's set SHALL drop that part and keep the rest of the rule, and validation SHALL report it.

### Requirement: Recurrence rules are validated

In [conformance](../../../spec/conformance.md). Calendar validation SHALL report a rule that does not parse as `IcalValidateError::RuleSyntax` rather than pass it silently.

### Requirement: A zone the calendar omits resolves from the database, opt-in

In [tz](../../../spec/tz.md). A Windows time zone name SHALL read as the IANA zone CLDR maps it to for territory 001; `tzdb::is_known` and `tzdb::vtimezone` stay IANA only.

## REMOVED Requirements
