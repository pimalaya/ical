---
cairn: tasks
change: date-parts-on-a-date-time-series
---

- [x] Ask dateutil, libical and recurring-ical-events (and ical.js if available) for both cases; keep the answers
- [x] Check validation reports a `DATE` `UNTIL` or `EXDATE` beside a date-time `DTSTART`, and add it if not
- [x] Read a `DATE` `EXDATE` as removing every instance on that day, on `DTSTART`'s clock
- [x] Read a `DATE` `UNTIL` as inclusive of the day it names (declined: midnight, as the oracles read it)
- [x] Regression tests that fail on today's code (the two probe cases)
- [x] Re-run the timeline corpus and record any series that moved
- [x] Fold the spec, log, CHANGELOG
