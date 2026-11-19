---
cairn: delta
change: until-on-the-series-clock
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: What JSCalendar normalises

An `RRULE`'s `UNTIL` is stated in UTC beside a zoned `DTSTART` (RFC 5545 3.3.10), and RFC 8984 states it as a local date-time in the object's `timeZone` (4.3.3). Exporting, a UTC `UNTIL` of a zoned object SHALL be told on that zone's clock, and importing, the `until` of a zoned object SHALL be told back to UTC, read as RFC 5545 3.3.5 reads a written time (the earlier of a fold, the offset before a gap). The zone is the `VTIMEZONE` the calendar defines under the start's `TZID` (importing, one the Group's hatch carries), and behind the `tzdb` feature a zone the database knows, IANA or Windows named, a calendar's own definition completed from the database as `tzdb::zones` completes it; this holds for a lone object too. A UTC `UNTIL` in a fold's second pass reads back as the first, JSCalendar having no way to say which. A floating object's `UNTIL` SHALL stay floating and a UTC object's in UTC. An object shown without a time SHALL keep a `DATE` `UNTIL`, `T00:00:00` in JSCalendar. Without the `tzdb` feature, or for a zone neither the calendar nor the database defines, the wall-clock digits SHALL be carried across unshifted and written back without `Z`: exact for a floating or UTC object, off by the zone's offset exporting any other, and importing a local `UNTIL` that this crate's expansion compares on the start's clock as RFC 8984 does, though RFC 5545 wants UTC there.

#### Scenario: A UTC end of recurrence in a zone only the database knows

- GIVEN the `tzdb` feature, no `VTIMEZONE`, and a series starting `DTSTART;TZID=Europe/Paris:20261203T100000` whose `RRULE` ends `UNTIL=20270603T080000Z`
- WHEN it is converted to JSCalendar and back
- THEN the rule's `until` is `2027-06-03T10:00:00`, and the `RRULE` comes back ending `UNTIL=20270603T080000Z`

#### Scenario: A local end of recurrence from a JMAP object

- GIVEN the `tzdb` feature and an Event, lone or in a Group with no hatch, in `Europe/Paris` whose rule's `until` is `2027-06-03T10:00:00`
- WHEN it is converted to iCalendar
- THEN the `RRULE` ends `UNTIL=20270603T080000Z`

#### Scenario: A local end of recurrence in a gap or a fold

- GIVEN the `tzdb` feature and a rule in `Europe/Paris` whose `until` is `2027-03-28T02:30:00`, or `2027-10-31T02:30:00`
- WHEN it is converted to iCalendar
- THEN the `RRULE` ends `UNTIL=20270328T013000Z` (the offset before the gap), or `UNTIL=20271031T003000Z` (the earlier of the fold)

#### Scenario: Floating, UTC and all-day ends of recurrence

- GIVEN a floating series ending `UNTIL=20270603T100000`, a UTC one ending `UNTIL=20270603T090000Z` and an all-day one ending `UNTIL=20270603`
- WHEN each is converted to JSCalendar and back
- THEN each `RRULE` comes back with its `UNTIL` as written

## REMOVED Requirements
