---
cairn: tasks
change: tzdb-pinned-and-versioned
---

- [x] `tzdb::version`, from `jiff-tzdb::VERSION`
- [x] `tzdb::zone` and `tzdb::zones`, and `IcalTz::of_components` in the core; the calendar's own always wins
- [x] Tests (tests/recur_instant.rs, `tzdb` module): the version is a release name; an undefined `Europe/Paris` resolves as the defined one; a defined zone wins over the database; an unknown name reads floating; a database zone is exact across the 2007 United States rule change
- [x] Verify the bare core stays dependency-free and `no_std` with `tzdb` off
- [x] Fold tz.md, log, CHANGELOG
- [ ] Consumers: pimdir search-milestones (the version in `index_meta`), io-pimdir and the Android expansion
