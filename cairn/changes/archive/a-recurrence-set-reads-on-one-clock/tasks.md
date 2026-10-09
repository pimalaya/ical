---
cairn: tasks
change: a-recurrence-set-reads-on-one-clock
---

# Tasks

- [x] `IcalRecurZone` and `IcalRecurSet::zone`, read from `DTSTART`.
- [x] `of_component_in`, `with_override_in`, `IcalRecurOverride::of_component`; `of_component` and `with_override` as the zone-less forms; `of_uid` reads the series first and finds the zones itself.
- [x] Tests: the UTC `UNTIL` repro, a UTC and a New York `EXDATE`, a UTC `RECURRENCE-ID`, one override read alone, and the zone-less comparison.
- [x] Fold into recurrence, log, changelog.
