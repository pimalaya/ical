---
cairn: delta
change: occurrences-as-instants
---

## ADDED Requirements

### Requirement: A series walks to instants over a window

`IcalRecurTimeline::of_uid` SHALL read the series of one `UID` among a calendar's components, its overrides included, through a given list of zones, and `IcalRecurTimeline::expand` SHALL walk it over a window `[from, until)` of Unix seconds, lazily and in identity order, to occurrences carrying their identity (civil on the series' clock, and as an instant), their start and their end as instants, whether an override placed them, whether they were read on no zone, and whether they are all-day.

Each civil time SHALL be resolved through the zone its own clock names only once the set has placed the occurrence, as a written `DATE-TIME` (RFC 5545 3.3.5); an instance a rule generates in a gap is dropped as recurrence drops it. A floating or date time, or one under a `TZID` none of the zones defines, SHALL be read as if on UTC and flagged floating, so its civil time reads back from its instant exactly.

The end SHALL come from the occurrence's own component: its `DTEND` (a `VTODO`'s `DUE`) on its own clock, else its `DURATION`, else RFC 5545 3.6.1's default, a day for a `VEVENT` on a date and none otherwise. A `DTEND` span SHALL apply to every other instance as the exact seconds it measures (RFC 5545 3.8.5.3), an `RDATE` period's own span to its instance. A nominal duration SHALL be added in wall-clock days on the start's clock before resolution, an exact one in seconds after. An end before its start SHALL read as the start.

An occurrence SHALL be yielded when it starts before `until` and ends after `from`, or, lasting no time, starts within the window, an override moving an instance in from outside the window included. The walk SHALL skip ahead to the window rather than walk from `DTSTART`, and SHALL bound nothing itself: a consumer bounding what one series yields takes from it. A series with no `DTSTART` SHALL yield nothing and not fail, save a task with a `DUE`, no `DTSTART` and no `RRULE`, which is one occurrence lasting no time at its due time. The same components, zones and window SHALL always yield the same occurrences.

#### Scenario: A one-day duration across a transition
- GIVEN `DTSTART;TZID=Europe/Paris:20261024T120000`, `DURATION:P1D` and the Paris `VTIMEZONE`
- WHEN the series is walked over that weekend
- THEN the occurrence ends at 12:00 Paris time on 25 October, 25 hours after it starts

#### Scenario: An exact duration across the same transition
- GIVEN the same start and `DURATION:PT24H`
- WHEN the series is walked
- THEN the occurrence ends 24 hours after it starts, at 11:00 Paris time

#### Scenario: An override in another zone
- GIVEN a daily Paris series and an override moving one instance to `TZID=America/New_York`
- WHEN the series is walked
- THEN that occurrence's start and end are the New York instants its own component states, and its identity is still the Paris instance it replaced

#### Scenario: An end in another zone
- GIVEN `DTSTART;TZID=Europe/Paris:20260706T090000`, `DTEND;TZID=America/New_York:20260706T090000` and a daily rule
- WHEN the series is walked
- THEN every occurrence lasts six hours, from 07:00Z to 13:00Z on its day

#### Scenario: An override moved in from years away
- GIVEN a weekly series from 2020 and an override moving its second instance to 8 July 2026
- WHEN the series is walked over 7 to 9 July 2026
- THEN that override is the one occurrence, unless an `EXDATE` removes the instance it names

## MODIFIED Requirements

### Requirement: Civil expansion

Expansion SHALL be civil: no time zone, no offset. RFC 5545 defines expansion on the local wall-clock time of `DTSTART`, so no UTC offset is ever needed and none is ever resolved. Turning an occurrence into an instant is the step after it, served by [tz](./tz.md) for one time and by the walk of a series over a window of instants for a whole series (below). The times a set compares are first told on one clock (see below), which is a reading of the calendar, not a change to how it expands.

A zone MAY be supplied to an expansion, and SHALL then be consulted for one purpose only: to drop the instances RFC 5545 3.3.10 forbids counting. It SHALL never change how an occurrence is represented or how the walk steps, both of which stay total arithmetic on civil times.

#### Scenario: A daily rule on a zoned start
- GIVEN `DTSTART;TZID=Europe/Paris:20260301T090000` and `FREQ=DAILY`
- WHEN the rule is expanded
- THEN every occurrence is 09:00:00 civil, across a daylight-saving transition

### Requirement: A set reads every time on its start's clock

A recurrence set SHALL tell every time it holds on one clock, its `DTSTART`'s (`IcalRecurSet::zone`: floating, UTC, or the zone a `TZID` names), since its identities, its exclusions and its bounds are compared with one another.

A time written on another clock SHALL be brought onto that one as the set is read, through the `VTIMEZONE`s that define both clocks, each read as a written `DATE-TIME` (RFC 5545 3.3.5): a UTC `UNTIL`, which RFC 5545 3.3.10 requires beside a zoned `DTSTART`; an `RDATE` or `EXDATE` item in UTC or under another `TZID` (3.8.5.1, 3.8.5.2); and an override's `RECURRENCE-ID` and `DTSTART` (3.8.4.4). `IcalRecurSet::of_uid` SHALL find the zones among the calendar components it is given; `of_uid_in`, `of_component_in` and `with_override_in` SHALL take them, a list the time zone database completes included ([tz](./tz.md)); `IcalRecurOverride::of_component` SHALL read one override onto a given clock without a set.

Without the zones (`of_component`, `with_override`), on a floating clock, or for a zone the calendar does not define, a time SHALL be compared as it is written, which is what the set did before.

#### Scenario: A UTC bound on a zoned series
- GIVEN `DTSTART;TZID=Europe/Paris:20260706T090000`, `RRULE:FREQ=DAILY;UNTIL=20260708T070000Z` and the Paris `VTIMEZONE`
- WHEN the set of its `UID` is expanded
- THEN 6, 7 and 8 July at 09:00 are its occurrences, 07:00Z being 09:00 in Paris

#### Scenario: An exclusion and an override on other clocks
- GIVEN the same series with an `EXDATE` at `20260707T070000Z` and another under `TZID=America/New_York` at `20260708T030000`, or an override whose `RECURRENCE-ID` is `20260707T070000Z`
- WHEN the set is expanded
- THEN each names the Paris 09:00 instance it means: the exclusions remove theirs, and the override replaces its own rather than standing beside it

#### Scenario: No zones at hand
- GIVEN the UTC-bounded series read with `of_component` alone
- WHEN it is expanded
- THEN the bound is compared as written and 8 July is not an occurrence

## REMOVED Requirements
