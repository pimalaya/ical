---
cairn: log
change: vtimezone-offsets
landed: 2026-08-08
---

# VTIMEZONE offset resolution

`timezone::IcalTimezone::resolve` turns a civil date-time into a UTC offset from the `VTIMEZONE` the calendar carries, with no time-zone database and no new dependency. The rules travel inside the calendar, which is the whole point of RFC 5545 3.6.5, and the crate already had every piece: the `VTIMEZONE`, `STANDARD` and `DAYLIGHT` components, `TZOFFSETFROM` and `TZOFFSETTO`, and a rule expander.

An observance's onsets are read through `IcalRecurSet`, the recurrence set that landed just before this. A zone's transitions are a recurrence set like any other, so `DTSTART`, `RRULE` and `RDATE` inside a `STANDARD` or `DAYLIGHT` need no second implementation. That reuse is the reason this change is small.

The two hard cases are the reason it exists. A transition expresses one instant in two local times, the one before it and the one after, and the interval between them is either a hole or a repetition. `resolve` returns `One`, `Gap { before, after }` or `Fold { earlier, later }`, so a caller learns which of the three it is rather than being handed a plausible number. Choosing what to do with a skipped alarm belongs to the caller, who knows whether it should fire early, late or not at all. `unambiguous()` is there for the caller who only wants the easy case.

The boundary did not move: expansion is still civil, and nothing in `recur` resolves an offset. This module sits beside it, gated on the same `recur` feature since it is built on the expander.

Eight cases drive it: summer and winter, the spring gap and the fall fold with their unambiguous neighbours either side, a time before every transition, a zone that never shifts, an unknown `TZID`, and the committed RFC 5545 timezone fixture, whose observances state a single onset each and no rule.

Capabilities moved: `timezone` (ADDED: offset resolution from the calendar's own rules, the gap and the fold are reported).
