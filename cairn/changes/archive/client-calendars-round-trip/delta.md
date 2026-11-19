---
cairn: delta
change: client-calendars-round-trip
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Nothing is dropped

As before, plus: a component stating both `DTSTAMP` and `LAST-MODIFIED` SHALL give `updated` its `DTSTAMP` and keep its `LAST-MODIFIED` whole in the hatch, whatever their order.

### Requirement: What JSCalendar normalises

A round trip SHALL change a calendar in these ways and no other.

An `RRULE`'s `UNTIL` is stated in UTC whenever `DTSTART` is, and RFC 8984 states it in the object's own time zone. Exporting, a UTC `UNTIL` SHALL be told on the `VTIMEZONE` the calendar defines under the start's `TZID`, and importing, a local `UNTIL` of a zoned object SHALL be told back to UTC on the `VTIMEZONE` the Group's hatch carries, read as RFC 5545 3.3.5 reads a written time (the earlier of a fold, the offset before a gap). With no such `VTIMEZONE` (a zone only the time-zone database knows, a lone object), the wall-clock digits are carried across unshifted, which is exact for a floating or UTC object and off by that zone's offset for any other.

A `DTEND` becomes a duration, so an event that ended in another time zone than it started in comes back with the start's zone on both ends.

Ordering is lost: of the lines inside a component, of the parameters on a line, of the parts of a recurrence rule (written back in RFC 5545 order) and of the items of a `CATEGORIES` (keywords are a set). An `EXDATE` or `RDATE` listing several dates comes back one line per date. A parameter stating what a property means with none (`VALUE` of the default type, `ROLE=REQ-PARTICIPANT`, `RELATED=START`) is not written back, and a parameter value is quoted only where RFC 5545 3.2 needs it. Folds and the case of names are the writer's.

#### Scenario: An event in a named zone

- GIVEN `DTSTART;TZID=Europe/Berlin` and `DTEND;TZID=Europe/Berlin` an hour and a half later
- WHEN the event is converted
- THEN it states `PT1H30M` and the zone once

#### Scenario: A UTC end of recurrence in a defined zone

- GIVEN a calendar defining a `VTIMEZONE` and a series starting in it whose `RRULE` ends `UNTIL=20270603T080000Z`
- WHEN it is converted to JSCalendar and back
- THEN the rule's `until` is the zone's `2027-06-03T10:00:00`, and the `RRULE` comes back ending `UNTIL=20270603T080000Z`

#### Scenario: Calendars as clients write them

- GIVEN each of tests/corpus/clients: an iOS series with `X-APPLE-*` properties and parameters, alarms, a `VTIMEZONE`, an `EXDATE` and a moved occurrence; an Exchange meeting with `X-MICROSOFT-*` properties, attendees, an alarm, a monthly rule ending in UTC, an `EXDATE` and an exception; a Google invitation with `X-NUM-GUESTS` parameters, two alarms, an `EXDATE` list and a moved occurrence; a Thunderbird task and all-day event with `X-MOZ-*` properties
- WHEN each is converted to JSCalendar, in RFC 8984 and in 2.0, and back
- THEN it comes back line for line once the orderings, lists, defaults, quotes and folds above are normalized, and a second conversion equals the first

## REMOVED Requirements
