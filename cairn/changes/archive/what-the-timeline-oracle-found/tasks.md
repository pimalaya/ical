---
cairn: tasks
change: what-the-timeline-oracle-found
---

- [x] SEQUENCE picks the series and the override, a former series is dropped (set::read_uid, shared by the set and the timeline)
- [x] An end before its start is swapped, the window widened back by it
- [x] A task with no DTSTART recurs from its DUE (set::anchor_of)
- [x] IcalRecurTimeline::of_component for a component with no UID
- [x] THISANDFUTURE carries the override's span forward
- [x] tzdb::zones completes a VTIMEZONE's past from the database
- [x] IcalTz::x_wr_timezone and IcalTz::apply_x_wr_timezone
- [x] A test per behaviour, each checked to fail against the code before it (the two new entry points excepted, which did not exist)
- [x] Fold into spec/recurrence.md and spec/tz.md, log, CHANGELOG
