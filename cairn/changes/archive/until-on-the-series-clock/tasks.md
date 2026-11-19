---
cairn: tasks
change: until-on-the-series-clock
---

- [x] `tzdb::zone_in` (crate-visible): a calendar's own zone completed from the database, else the database's
- [x] `until_zone` in src/jscalendar.rs, the database behind `tzdb`, the calendar's own zones without it
- [x] Export (`rule_to_json`) and import (`rule_from_json`, `utc_text`) through it, the year of the `UNTIL` bounding the zone
- [x] Import: a floating `UNTIL` floating, an all-day one a `DATE`
- [x] Tests in tests/jscalendar.rs: database zone both ways, lone and grouped, gap and fold, fold second pass, floating, UTC, all-day, an undefined zone, no database
- [x] tests/jscalendar_clients.rs reads fixtures with LF line endings
- [x] Module doc in src/jscalendar.rs and src/jscalendar/export/temporal.rs
- [x] Fold into cairn/spec/jscalendar.md, log, CHANGELOG
