---
cairn: change
id: date-parts-on-a-date-time-series
status: landed
created: 2026-10-29
---

# Date parts on a date-time series

## Why

RFC 5545 3.3.10 and 3.8.5.1 want `UNTIL` and `EXDATE` to share `DTSTART`'s value type, and producers break that. A probe on 2026-10-29, over `DTSTART;TZID=Europe/Paris:20260706T090000` and `FREQ=DAILY`, showed what the crate does today:

- `EXDATE;VALUE=DATE:20260707` removes nothing: no instance's identity equals a date, so the 7 July instance stays. The writer certainly meant to remove that day.
- `UNTIL=20260708` stops before 8 July 09:00: the date reads as midnight, so the last day it names is excluded although `UNTIL` is inclusive.
- `EXDATE:20260707T000000` beside a `DATE` start does remove 7 July, and a floating `UNTIL` reads on `DTSTART`'s clock. Those two are fine.

Neither case is covered by the recur corpus (its generator writes date-time `UNTIL`s only) or the timeline corpus, so no oracle has spoken.

## What

1. Ask the oracles first: dateutil (rrulestr with `forceset`), libical (`icalcomponent_foreach_recurrence`), recurring-ical-events 3.9.0, and ical.js once [a-second-timeline-oracle](../a-second-timeline-oracle/proposal.md) brings it in. Record their answers in the log, not just ours.
2. Recommended answer, pending that: a `DATE` `EXDATE` on a date-time series removes every instance whose identity falls on that day on `DTSTART`'s clock, and a `DATE` `UNTIL` on a date-time series includes the whole day it names. Both read the date as the day it says, which is the only reading under which the writer's value does anything. If the oracles split, this reading still stands; if they agree on another, that is a decision for the user, not a default.
3. Validation already reports the type mismatch (or should: check), so a strict reader is told even when expansion is liberal.

## Consequence

Behaviour change for calendars that rely on today's literal reading, which removes nothing and cuts a day early: none plausibly do on purpose. No API change.
