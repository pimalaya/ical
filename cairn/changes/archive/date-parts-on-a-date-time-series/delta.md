---
cairn: delta
change: date-parts-on-a-date-time-series
---

## ADDED Requirements

### Requirement: A date part on a date-time series names a day

In [recurrence](../../spec/recurrence.md), under the recurrence set of a component. An `EXDATE` item of value type `DATE` beside a `DATE-TIME` `DTSTART` SHALL remove every instance whose identity falls on that day, read on `DTSTART`'s clock. An `UNTIL` of value type `DATE` beside a `DATE-TIME` `DTSTART` SHALL read as the start of that day, as every oracle does (amended on landing; the proposal had the whole day). Validation SHALL still report both, RFC 5545 3.3.10 and 3.8.5.1 asking for `DTSTART`'s type.

#### Scenario: A date exception on a timed series
- GIVEN `DTSTART;TZID=Europe/Paris:20260706T090000`, `RRULE:FREQ=DAILY;COUNT=4` and `EXDATE;VALUE=DATE:20260707`
- WHEN its set is expanded
- THEN 7 July is not an instance, and 6, 8 and 9 July are

#### Scenario: A date until on a timed series
- GIVEN `DTSTART;TZID=Europe/Paris:20260706T090000` and `RRULE:FREQ=DAILY;UNTIL=20260708`
- WHEN its set is expanded
- THEN it yields 6 and 7 July at 09:00
