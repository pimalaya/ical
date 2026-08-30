---
cairn: tasks
change: a-gap-instance-is-not-counted
---

- [x] Materialise a zone's transitions once, so a validity check is a lookup rather than an expansion
- [x] Add `IcalTz::is_gap`, the predicate the filter needs
- [x] Add `IcalRecurExpand::in_zone`, taking the zone owned so the iterator keeps no lifetime
- [x] Skip a gap candidate before `emitted` rises, and before the `UNTIL` bound is consulted
- [x] Pass the zone down to the rule streams of `IcalRecurSet`, never to its `RDATE`s
- [x] Add `IcalTzOffset::instant`, and say in its docs which answer is the RFC's
- [x] Say on `IcalTzOffset::Gap` that a recurrence instance landing in one is dropped by rule, not by choice
- [x] Test `COUNT` against a rule whose instance falls in a gap: five out, not four
- [x] Fold the spec and log the change
