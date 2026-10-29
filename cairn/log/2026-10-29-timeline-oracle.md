---
cairn: log
change: timeline-oracle
date: 2026-10-29
---

# A third oracle for the timeline

The recurrence corpus pins the rule expander against python-dateutil and libical, but only civilly. Everything built on it since (the set on one clock, overrides, gaps, ends, the tzdb fallback, `IcalRecurTimeline`) was guarded by scenarios written here alone. recurring-ical-events, the Python library expanding a calendar's recurrences, is now its oracle.

- tests/corpus/rie: its 95 test calendars, from the v3.9.0 tag, unedited, under their LGPL-3.0-or-later licence.
- tests/corpus/timeline/harness: a detached Rust runner over `IcalRecurTimeline` (with `tzdb::zones`, `X-WR-TIMEZONE` applied, UID-less components through `of_component`), a Python runner over `occurrences_between` of recurring-ical-events 3.9.0 from nixpkgs (icalendar 6.3.2), and a cross. One window per calendar; both sides fetch three days wider and the harness keeps the occurrences starting in the window, the first 200 of each series.
- tests/corpus/timeline: windows.tsv, agreement.tsv (966 series), divergence.tsv (12, hand-curated), and tests/timeline_corpus.rs, which replays both with no Python and fails on any series neither lists.

The first run agreed on 73 of 91 calendars. Triage turned ten differences into fixes, landed as what-the-timeline-oracle-found and timeline-oracle-second-round, and recorded the rest. The replay was checked to fail with one of those fixes switched off.

Capability moved: **recurrence** (the timeline corpus is replayed).
