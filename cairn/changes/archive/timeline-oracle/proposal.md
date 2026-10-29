---
cairn: change
id: timeline-oracle
status: landed
created: 2026-10-29
---

# A third oracle for the timeline

## Why

The rule expander is pinned against python-dateutil and libical (tests/corpus/recur/consensus.tsv), but that corpus is civil: one rule, one start, no zone. Everything [recurrence](../../../spec/recurrence.md) and [tz](../../../spec/tz.md) added on top (the set on one clock, overrides, `RANGE=THISANDFUTURE`, gaps, ends, the tzdb fallback, `IcalRecurTimeline`) is guarded only by scenarios written here, against our own reading.

recurring-ical-events (python, LGPL-3.0-or-later) is the one independent implementation of that layer with a real-world test suite behind it: about ninety calendars, most taken from bug reports. Run over the same calendars and windows, it is a differential oracle for the timeline the way dateutil and libical are for the expander.

## What

A detached harness, as for the recurrence corpus: a Rust runner walking `IcalRecurTimeline` (with `tzdb::zones`), a Python runner calling `occurrences_between` of recurring-ical-events 3.9.0 from nixpkgs, and a cross. One window per calendar, from a year before its earliest `DTSTART`, `DUE` or `RECURRENCE-ID` to three years after its latest; both sides fetch three days wider and the harness keeps the occurrences starting in the window, the first 200 of each `UID`, spelled as UTC, floating or date.

The calendars are vendored under tests/corpus/rie/ from the v3.9.0 tag, untouched, with an ATTRIBUTION keeping their licence, as tests/corpus/libical/ does. A single oracle is a second opinion, not a consensus, so no case is accepted automatically: every series the two answer differently is triaged by hand into a bug here (fixed in its own change), a recorded divergence with its reason, or a harness artifact.

Committed: tests/corpus/timeline/agreement.tsv (the occurrences of every series both answer alike), divergence.tsv (each settled divergence with this crate's answer and why), the harness, and a replay test needing no Python.

## First run (prototype, 2026-10-29)

95 calendars, 73 agree in full, 18 differ. Every difference is explained:

| Calendars | Cause | Kind |
|---|---|---|
| fablab_cottbus | A truncated `VTIMEZONE` (Europe/Berlin, observances from 2018 only): before its first transition ical-rs takes that transition's `TZOFFSETFROM` (+0200, summer) for winter dates, the oracle uses tzdb (+0100). One hour off on every event before 2018-10-28. | Bug candidate: a `VTIMEZONE` that does not cover a time, for a name tzdb knows, could fall back to tzdb |
| issue_75_range_parameter | `RANGE=THISANDFUTURE` overrides that change the duration: ical-rs shifts later starts but keeps the series' duration, the oracle applies the override's duration too. | Bug candidate: spec says only "shift by the offset" |
| issue_86_x_wr_timezone | Floating times plus `X-WR-TIMEZONE`: the oracle reads them in that zone, ical-rs as floating. | Decision: non-standard, but every Google Calendar export carries it |
| issue_253_additional_recurrence_id | A stale component (`RECURRENCE-ID`, `RRULE`, `SEQUENCE:1`) beside the rescheduled master (`SEQUENCE:2`) whose `UNTIL` no longer reaches it: ical-rs keeps the orphan override as an instance, the oracle drops it by `SEQUENCE`. | Decision |
| end_before_start_event, issue_132 | `DTEND` before `DTSTART`: ical-rs reads the end as the start (spec), the oracle swaps them. issue_132 also shares one `UID` between a `VEVENT` and a `VTODO`. | Divergence |
| issue_97_todo_nodtstart | `VTODO` with `DUE` and `RRULE`, no `DTSTART`: the oracle expands from `DUE`, ical-rs yields nothing (spec). | Divergence |
| bad_rrule_missing_until_event | `UNTL=` typo: ical-rs ignores the unknown part as RFC 5545 requires and expands unbounded, the oracle refuses the calendar. | Divergence |
| issue_201 (two files) | `DTSTART` a date, `DTEND` a date-time (invalid): same instants, the oracle promotes the start to a date-time, ical-rs keeps it a date. | Divergence (type only) |
| duration, issue_107, issue_113, issue_128, issue_15, rdate, rdate2, rdate_hackerpublicradio | Components with no `UID`: `IcalRecurTimeline::of_uid` has no way to reach them, so ical-rs yields nothing. | Gap: decide whether a UID-less component gets an entry point |

## Settled (2026-10-29)

Seven of the differences landed as [what-the-timeline-oracle-found](../what-the-timeline-oracle-found/proposal.md): SEQUENCE, the swapped end, the task recurring from its `DUE`, UID-less components, the `THISANDFUTURE` span, a truncated `VTIMEZONE`'s past, `X-WR-TIMEZONE`. The second run agrees on 84 of 92 compared calendars.

Kept as recorded divergences: `UNTL=` (bad_rrule_missing_until_event), the date `DTSTART` beside a date-time `DTEND` (issue_201, two files), one `UID` shared by a `VEVENT` and a `VTODO` (issue_132).

## Second round (2026-10-29)

Three more differences, in components with no `UID`, landed as [timeline-oracle-second-round](../timeline-oracle-second-round/proposal.md): Windows zone names, a series cut before its start, a malformed rule part. The `VTIMEZONE` ending early with no `TZUNTIL` (fablab_cottbus) is a recorded divergence.

The frozen run: 966 series agree, 12 are recorded divergences, and the oracle refuses one calendar outright.
