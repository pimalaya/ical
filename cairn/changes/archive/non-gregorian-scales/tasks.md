---
cairn: tasks
change: non-gregorian-scales
---

- [x] Evaluate `icu_calendar`: no_std, MSRV, weight, Chinese year range, coverage of RFC 7529's calendar names
- [x] Decide icu_calendar or our own arithmetic calendars only
- [x] Make `by_month` carry a leap flag (`5L`) and any scale's month number (**breaking**)
- [x] Add the `rscale` feature; expand on the rule's scale, convert to Gregorian at the edge
- [x] `SKIP` for a leap month absent from a year (RFC 7529 3.1)
- [x] An RSCALE corpus crossed with libical built with ICU
- [x] Check `cargo build --no-default-features` stays dependency-free
- [x] Fold the spec (replacing "Only the Gregorian scale is expanded"), log, CHANGELOG
