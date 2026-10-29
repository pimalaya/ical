---
cairn: log
change: what-the-timeline-oracle-found
date: 2026-10-29
---

# What the timeline oracle found

A prototype differential run of `IcalRecurTimeline` against recurring-ical-events 3.9.0 (python, LGPL-3.0, from nixpkgs), over the 95 calendars of that project's test suite, agreed on 73 and differed on 18. Seven differences were this crate's to fix, each settled with the maintainer:

- `set::read_uid`, now shared by `IcalRecurSet::of_uid_in` and `IcalRecurTimeline::of_uid`, picks the series and each override by highest `SEQUENCE` and drops a former series a reschedule left behind (recurring-ical-events issue 253). The two paths read overrides apart before; the set's sort is now stable too.
- An end before its start is swapped rather than clamped, its identity kept on `DTSTART`; the window reaches back by the span.
- `set::anchor_of`: a `VTODO` with no `DTSTART` recurs from its `DUE` (RFC 8984 4.3.3). The timeline's one-point special case for a lone `DUE` went with it.
- `IcalRecurTimeline::of_component` reads a component with no `UID`.
- A `RANGE=THISANDFUTURE` override carries its own span to the instances it shifts.
- `tzdb::zones` completes a `VTIMEZONE` before its earliest observance (RFC 7808 7.1) from the database, for a name it knows.
- `IcalTz::x_wr_timezone` and `IcalTz::apply_x_wr_timezone`, the extension applied as the x-wr-timezone library applies it.

After them the run agrees on 84 of 92 compared calendars. Fifteen tests in tests/recur_instant.rs: the eight covering changed behaviour fail against the code before the change, three guards pass on both, and the four on the two new entry points had nothing to fail against.

Capabilities moved: **recurrence** (the highest SEQUENCE, a task recurring from its DUE, a component with no UID, the swapped end, the THISANDFUTURE span), **tz** (a truncated VTIMEZONE's past, X-WR-TIMEZONE).
