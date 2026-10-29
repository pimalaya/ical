---
cairn: log
change: date-parts-on-a-date-time-series
date: 2026-10-29
---

# Date parts on a date-time series

Oracles on a 09:00 Paris daily series: a `DATE` `EXDATE` is removed by libical 3.0.20, recurring-ical-events 3.9.0 and ical.js 2.2.1, kept by python-dateutil 2.9.0; a `DATE` `UNTIL` reads as midnight in all four.

- A `DATE` `EXDATE` beside a `DATE-TIME` `DTSTART` removes that day's instances (`IcalRecurSet::exdays`).
- A `DATE` `UNTIL` keeps the midnight reading: the user chose to follow the oracles over the proposal.
- `IcalValidateError::StartType` reports both mismatches.

Capabilities moved: **recurrence**, **conformance**.
