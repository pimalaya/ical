---
cairn: delta
change: what-the-timeline-oracle-found
---

## ADDED Requirements
### Requirement: The highest SEQUENCE is the revision

In [recurrence](../../../spec/recurrence.md). Of the components of one `UID` with no `RECURRENCE-ID`, the one with the highest `SEQUENCE` SHALL be the series, the last listed of equals; of the overrides naming one instance, the highest `SEQUENCE` SHALL win, the first listed of equals; a missing `SEQUENCE` SHALL count as 0. An override carrying an `RRULE`, with a lower `SEQUENCE` than the series, and naming an instance the series does not generate SHALL be dropped.

#### Scenario: A former series left behind
- GIVEN a series at `SEQUENCE:2` whose `UNTIL` stops on 20 July, and a component at `SEQUENCE:1` carrying an `RRULE` and a `RECURRENCE-ID` of 29 July
- WHEN the series is walked
- THEN 29 July is not an occurrence

### Requirement: A task recurs from its DUE

In [recurrence](../../../spec/recurrence.md). A `VTODO` with no `DTSTART` SHALL start its recurrence set, its overrides and its occurrences from its `DUE` (RFC 8984 4.3.3), each occurrence lasting no time.

#### Scenario: A weekly task with a due time
- GIVEN a `VTODO` with `DUE:20260710T170000Z`, `RRULE:FREQ=WEEKLY;COUNT=3` and no `DTSTART`
- WHEN it is walked
- THEN it yields 10, 17 and 24 July at 17:00Z

### Requirement: A component with no UID walks alone

In [recurrence](../../../spec/recurrence.md). `IcalRecurTimeline::of_component` SHALL read one component as a series with no override, so a component carrying no `UID` still has occurrences.

### Requirement: X-WR-TIMEZONE is applied on request

In [tz](../../../spec/tz.md). `IcalTz::apply_x_wr_timezone` SHALL tell every UTC or floating date-time of a `VEVENT`'s `DTSTART`, `DTEND`, `RDATE`, `RECURRENCE-ID` and `EXDATE` on the zone `X-WR-TIMEZONE` names, under a `TZID` naming it, a UTC time at the same instant and a floating one at the same wall-clock time. Dates and times already under a `TZID` SHALL be left alone, and the whole calendar when the given zones do not define the zone.

#### Scenario: A weekly UTC event under X-WR-TIMEZONE
- GIVEN `X-WR-TIMEZONE:Europe/Brussels` and `DTSTART:20261021T070000Z` with a weekly rule
- WHEN the extension is applied and the series walked
- THEN both weeks start at 09:00 in Brussels, across the 25 October transition

## MODIFIED Requirements
### Requirement: A series walks to instants over a window

In [recurrence](../../../spec/recurrence.md). "An end before its start SHALL read as the start" becomes: an end before its start SHALL be swapped with it, the identity staying on `DTSTART`, and the walk SHALL reach an occurrence whose swapped start falls in the window. An instance a `RANGE=THISANDFUTURE` override shifts SHALL end as that override's own component does, the latest in force winning. "A series with no `DTSTART` SHALL yield nothing, save a task with a `DUE`, no `DTSTART` and no `RRULE`" becomes: a series with no start, `DTSTART` or a task's `DUE`, SHALL yield nothing.

### Requirement: A zone the calendar omits resolves from the database, opt-in

In [tz](../../../spec/tz.md). "A `VTIMEZONE` the calendar carries SHALL always win over the database" becomes: it SHALL win over the span it is valid for, from its earliest observance on (RFC 7808 7.1); before that, a name the database knows SHALL read through the database's own transitions.

## REMOVED Requirements
