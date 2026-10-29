---
cairn: change
id: what-the-timeline-oracle-found
status: landed
created: 2026-10-29
---

# What the timeline oracle found

## Why

The first run of [timeline-oracle](../timeline-oracle/proposal.md), recurring-ical-events 3.9.0 over the 95 calendars of its own test suite, agreed with `IcalRecurTimeline` on 73 and differed on 18. Read one by one, seven of the differences were this crate reading real calendars worse than it could, each settled with the maintainer on 2026-10-29.

## What

1. **SEQUENCE picks the revision.** Of two components with no `RECURRENCE-ID`, the one with the highest `SEQUENCE` is the series; of two overrides naming one instance, the highest wins (RFC 5545 3.8.7.4). Listing order broke ties before and still does. An override carrying its own `RRULE`, with a lower `SEQUENCE` than the series, and naming an instance the series no longer generates is a former series a reschedule left behind, and is dropped (recurring-ical-events issue 253). Every other override naming no generated instance still stands.
2. **An end before its start is swapped with it**, rather than read as the start: the occurrence runs from the earlier to the later, and its identity stays on `DTSTART` so a `RECURRENCE-ID` or `EXDATE` still names it.
3. **A task with no `DTSTART` recurs from its `DUE`**, as RFC 8984 4.3.3 recurs a Task with no `start`, which is also what this crate's own JSCalendar import writes back. Before, only a task with no `RRULE` yielded, as one point.
4. **A component with no `UID` is a series of its own**, through `IcalRecurTimeline::of_component`: RFC 5545 requires the `UID`, a calendar in the wild may lack it.
5. **`RANGE=THISANDFUTURE` carries the override's duration forward**: a later instance it shifts ends as the override's own component does, not as the series does.
6. **A truncated `VTIMEZONE` defers to the database for its past.** A VTIMEZONE is valid from its earliest observance (RFC 7808 7.1); before it, `tzdb::zones` reads a name the database knows through the database's own transitions rather than through the offset the first observance came from. `TZUNTIL`, the other end, is not read yet.
7. **`X-WR-TIMEZONE` is applied** by `IcalTz::apply_x_wr_timezone`, as the x-wr-timezone library behind the oracle applies it: every UTC or floating date-time of a `VEVENT`'s `DTSTART`, `DTEND`, `RDATE`, `RECURRENCE-ID` and `EXDATE` is told on that zone under a `TZID`.

## Not changed

Recorded as divergences for the corpus instead: an unknown rule part (`UNTL=`) ignored rather than the calendar refused; a date `DTSTART` beside a date-time `DTEND` kept a date; one `UID` shared by a `VEVENT` and a `VTODO` read as one series.
