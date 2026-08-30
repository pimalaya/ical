---
cairn: tasks
change: a-time-zone-is-named-tz
---

- [x] Rename the `timezone` module to `tz`
- [x] Rename `IcalTimezone`, `IcalObservance` and `IcalOffset` to `IcalTz`, `IcalTzObservance` and `IcalTzOffset`
- [x] Say in the header why "observance" is the RFC's word rather than a datetime library's
- [x] Say on `IcalTzOffset` that it answers a resolution rather than naming a value
- [x] Fold the spec and log the change
