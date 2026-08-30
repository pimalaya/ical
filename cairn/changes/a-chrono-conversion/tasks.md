---
cairn: tasks
change: a-chrono-conversion
---

- [ ] Add the `chrono` feature, off by default, on `IcalDateTime::civil` from its sibling change
- [ ] Convert `IcalRecurDateTime` to and from `chrono::NaiveDateTime`, fallibly outward
- [ ] Refuse second 60 rather than mapping it onto chrono's nanosecond overflow
- [ ] Offer no `DateTime<Tz>` conversion, and say in the module docs why
- [ ] Verify the bare core is still `no_std` with the feature off
- [ ] Fold the spec and log the change
