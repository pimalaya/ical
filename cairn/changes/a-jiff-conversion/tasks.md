---
cairn: tasks
change: a-jiff-conversion
---

- [ ] Add `IcalDateTime::civil`, reading the wire text into an `IcalRecurDateTime`
- [ ] Add the `jiff` feature, off by default
- [ ] Convert `IcalRecurDateTime` to and from `jiff::civil::DateTime`, fallibly outward
- [ ] Refuse second 60 and an out-of-range year rather than mapping them
- [ ] Offer no zoned conversion, and say in the module docs why
- [ ] Verify the bare core is still `no_std` with the feature off
- [ ] Fold the spec and log the change
