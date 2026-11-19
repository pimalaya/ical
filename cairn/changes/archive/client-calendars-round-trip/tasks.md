---
cairn: tasks
change: client-calendars-round-trip
---

- [x] Fixtures tests/corpus/clients: ios-series.ics, outlook-meeting.ics, google-invite.ics, thunderbird-todo.ics
- [x] tests/jscalendar_clients.rs: line-for-line round trip in RFC 8984 and 2.0, then a second pass equal to the first
- [x] A `LAST-MODIFIED` beside a `DTSTAMP` kept in the hatch (`Builder::stamped` in src/jscalendar/export.rs)
- [x] A UTC `UNTIL` told on the calendar's `VTIMEZONE` and back (src/jscalendar/export/temporal.rs, src/jscalendar/import/temporal.rs, `IcalTz::told` made crate-visible)
- [x] Tests in tests/jscalendar.rs for both
- [x] Fold into cairn/spec/jscalendar.md, log, CHANGELOG
