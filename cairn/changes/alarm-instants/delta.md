---
cairn: delta
change: alarm-instants
---

## ADDED Requirements

The requirements below open a new capability, spec/alarm.md: when the alarms of one event or to-do fire. Always available and dependency-free, beside recurrence, and built on its walk of a series to instants.

### Requirement: The firings of a series over a window

`IcalAlarms::of_uid` SHALL read the alarms of one `UID` among a calendar's components, the `VALARM`s (RFC 5545 3.6.6) of its series and of every override, through a given list of zones, and `IcalAlarms::firings` SHALL list every firing whose instant falls in a window `[from, until)` of Unix seconds, in time order.

Each firing SHALL name the alarm (its index among the `VALARM`s of the component holding it, and its `UID` when RFC 9074 4 gives it one), the override holding it when one does (its identity), the occurrence it belongs to (absent for an item that does not recur), its rank in a repetition (zero for the trigger itself), its `ACTION` as written, the instant it fires, and the alarm's `ACKNOWLEDGED` instant when it carries one. The same components, zones and window SHALL always give the same firings.

#### Scenario: A single event
- GIVEN a `VEVENT` starting at `20261030T090000Z` with a `VALARM` whose `TRIGGER` is `-PT15M`
- WHEN its firings are listed over that day
- THEN one firing comes out at 08:45Z, alarm 0, with no occurrence

### Requirement: A relative trigger counts from the start or the end

A `TRIGGER` of value type `DURATION` SHALL count from the occurrence's start, or from its end with `RELATED=END` (RFC 5545 3.8.6.3). A `VEVENT`'s start SHALL be its `DTSTART`, and its end its `DTEND`, else `DTSTART` plus `DURATION`, else RFC 5545 3.6.1's default (the next day for a date, the start otherwise). A `VTODO`'s start SHALL be its `DTSTART`, and its end its `DUE`, else `DTSTART` plus `DURATION`; a `VTODO` with a `DTSTART` alone has no end.

The trigger's weeks and days SHALL be added on the anchor's wall clock, and its hours, minutes and seconds on the timeline (RFC 5545 3.3.6), so `-P1D` before a start falls at the same local time the day before across a daylight-saving change.

#### Scenario: Before the end
- GIVEN a `VEVENT` from 09:00Z to 10:00Z with `TRIGGER;RELATED=END:-PT5M`
- WHEN its firings are listed
- THEN it fires at 09:55Z

#### Scenario: A day before, across a transition
- GIVEN `DTSTART;TZID=Europe/Paris:20261025T090000`, the Paris `VTIMEZONE` and `TRIGGER:-P1D`
- WHEN its firings are listed
- THEN it fires at 09:00 Paris time on 24 October, 25 hours before the start

### Requirement: An absolute trigger fires once

A `TRIGGER` of value type `DATE-TIME` SHALL fire once at the UTC instant it names, whatever the recurrence of its parent, and SHALL belong to no occurrence. One written without its `Z`, which RFC 5545 forbids, SHALL be read on the floating clock (below).

#### Scenario: An absolute trigger on a daily series
- GIVEN a daily `VEVENT` holding a `VALARM` with `TRIGGER;VALUE=DATE-TIME:20261030T080000Z`
- WHEN its firings are listed over a week
- THEN one firing comes out, at 08:00Z on 30 October

### Requirement: A relative trigger fires once per occurrence

A relative trigger SHALL fire once for each occurrence of a recurring parent, counted from that occurrence's own start or end, and SHALL carry its identity. An occurrence an override places SHALL fire the override's own `VALARM`s instead of the series', none if it has none. An occurrence after an override carrying `RANGE=THISANDFUTURE`, and not overridden itself, SHALL fire the alarms of the latest such override before it (RFC 5545 3.8.4.4).

#### Scenario: A weekly series with a moved instance
- GIVEN a weekly series with a `-PT10M` alarm, and an override moving its second instance an hour later with a `-PT30M` alarm of its own
- WHEN its firings are listed over three weeks
- THEN the first and third instances fire ten minutes before their start, and the second thirty minutes before its moved start, naming the override

### Requirement: A repetition adds firings

`REPEAT` and `DURATION` together SHALL add that many firings after the first, each `DURATION` after the previous (RFC 5545 3.6.6). One without the other SHALL add none, the alarm still firing once, and SHALL be reported.

#### Scenario: Three snoozes
- GIVEN an alarm with `TRIGGER:-PT15M`, `REPEAT:3` and `DURATION:PT5M` on a start at 09:00Z
- WHEN its firings are listed
- THEN it fires at 08:45Z, 08:50Z, 08:55Z and 09:00Z, ranked 0 to 3

### Requirement: A missing anchor is reported, never guessed

A relative trigger whose anchor its parent lacks SHALL resolve to no firing and SHALL be reported by `IcalAlarms::problems`, naming the alarm and the anchor missing: a start for a `VEVENT` with no `DTSTART`, an end for a `VTODO` with neither `DUE` nor `DURATION`. An alarm with no `TRIGGER`, or one no reading makes sense of, SHALL be reported the same way.

As a leniency, a relative trigger counted from the start of a `VTODO` with a `DUE` and no `DTSTART`, which some clients write to mean "before it is due" and RFC 5545 3.8.6.3 forbids, SHALL count from its `DUE`.

#### Scenario: A to-do with no dates
- GIVEN a `VTODO` with no `DTSTART` and no `DUE`, holding a `TRIGGER:-PT15M` alarm
- WHEN its firings are listed
- THEN none come out, and the alarm is reported as missing its start

#### Scenario: Before a due date, leniently
- GIVEN a `VTODO` with `DUE:20261030T170000Z`, no `DTSTART`, and `TRIGGER:-PT1H`
- WHEN its firings are listed
- THEN it fires at 16:00Z

### Requirement: A floating anchor resolves in a supplied zone

An anchor read on no zone (a floating time, a date, or a `TZID` none of the zones defines) SHALL resolve in the zone the caller supplies to `firings`, as a written `DATE-TIME` (RFC 5545 3.3.5), else as if on UTC.

#### Scenario: A floating event on a Paris desktop
- GIVEN `DTSTART:20261030T090000` with `TRIGGER:-PT15M`, and the Paris zone supplied
- WHEN its firings are listed
- THEN it fires at 07:45Z

### Requirement: The resolution filters nothing

`ACKNOWLEDGED` (RFC 9074 6) SHALL be carried on each firing of its alarm, and no firing SHALL be dropped for it, nor for its parent's `STATUS`: what to skip is the consumer's choice.

#### Scenario: A dismissed alarm
- GIVEN an alarm carrying `ACKNOWLEDGED:20261030T084600Z`
- WHEN its firings are listed
- THEN its firing comes out carrying that instant

## MODIFIED Requirements

## REMOVED Requirements
