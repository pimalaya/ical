---
cairn: tasks
change: occurrences-as-instants
---

- [x] `IcalRecurTimeline`, `IcalRecurTimelineExpand` and `IcalRecurInstant`, built on `IcalRecurSetExpand` and `skip_to`; `IcalRecurSet::of_uid_in`; `IcalDuration::nominal`
- [x] The end: `DTEND` (`DUE`) on its own clock, `DURATION` nominal or exact, the RFC 5545 3.6.1 default, an `RDATE` period's end
- [x] Gap and fold handling as recurrence and tz already state them
- [x] Tests (tests/recur_instant.rs): the three scenarios of delta.md, an end in another zone, an all-day series, a gap instance, a window far from `DTSTART`, an occurrence overlapping the window's start, an override moved into the window from years away and one an `EXDATE` removes, no `DTSTART`, a task with a `DUE` alone, `RDATE` periods, an unknown `TZID`; each checked to fail with its code mutated. Determinism is structural (no clock, no hash order, a stable sort of overrides) rather than tested by two runs, which would pass either way
- [ ] Differential check against the Android bridge's expansion on its own tests (rust/src/calendar/expand.rs), so the bridge can move onto this: left to the bridge's migration
- [x] Bare core stays dependency-free
- [x] Fold recurrence.md, log, CHANGELOG
