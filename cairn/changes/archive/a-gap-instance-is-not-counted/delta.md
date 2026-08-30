---
cairn: delta
change: a-gap-instance-is-not-counted
---

## ADDED Requirements

### Requirement: A nonexistent local time is dropped and not counted

A rule-generated instance whose local time does not exist in the zone the component references SHALL be omitted from the recurrence set, and SHALL NOT consume a `COUNT` slot (RFC 5545 3.3.10). A rule bounded by `COUNT` therefore yields as many occurrences as it names, running further in time to do so.

The rule SHALL apply to instances a rule generated. A date named by an `RDATE` SHALL be kept, the specification's clause being about what rules generate rather than about every date-time in a gap.

An expansion given no zone SHALL behave as it did before, since a validity it cannot check is one it does not apply.

#### Scenario: A COUNT slot a gap does not consume
- GIVEN a daily rule at a local time the clock jumps over, bounded by `COUNT=5`
- WHEN it is expanded in the zone that jumps
- THEN five occurrences come out, and the series runs one period past where it would have ended

#### Scenario: A date an RDATE names in a gap
- GIVEN an `RDATE` naming a local time the clock jumps over
- WHEN the set is expanded in that zone
- THEN the date is kept, having been named rather than generated

### Requirement: The crossing to an instant is named once

`IcalTzOffset::instant` SHALL give the instant a civil time names, as seconds since the Unix epoch. It SHALL return nothing in a gap, which is the RFC's answer rather than a refusal to answer, and the earlier of the two in a fold, which is a default the RFC does not mandate and which the variant's fields still expose.

## MODIFIED Requirements

### Requirement: Civil expansion

Expansion SHALL be civil: no time zone, no offset. RFC 5545 defines expansion on the local wall-clock time of `DTSTART`, so no UTC offset is ever needed and none is ever resolved. Turning an occurrence into an instant is the caller's step, served by [tz](./tz.md).

A zone MAY be supplied to an expansion, and SHALL then be consulted for one purpose only: to drop the instances RFC 5545 3.3.10 forbids counting. It SHALL never change how an occurrence is represented or how the walk steps, both of which stay total arithmetic on civil times.
