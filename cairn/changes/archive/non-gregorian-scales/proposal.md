---
cairn: change
id: non-gregorian-scales
status: landed
created: 2026-10-29
---

# Non-Gregorian scales

## Why

An `RSCALE` other than `GREGORIAN` parses and yields nothing (spec/recurrence.md, "Only the Gregorian scale is expanded"). That is honest but empty: a Hebrew-calendar birthday or a Chinese New Year rule is a real use, and libical built with ICU expands them. nixpkgs' libical is built against icu4c 76.1, so an oracle is already one `nix build` away.

Two gaps beyond arithmetic:

- `IcalRecurRule::by_month` is `Vec<u8>` read as 1 to 12, so RFC 7529's leap months (`BYMONTH=5L`) and a Hebrew 13th month do not parse. That is a public field: changing it is **breaking**.
- Everything after the rule (zones, sets, the timeline) is Gregorian civil time, and should stay so.

## What

1. Behind an off-by-default `rscale` feature, expand on a scale: the rule steps years, months and days and resolves `BYMONTH`, `BYMONTHDAY`, `BYYEARDAY` and `SKIP` in the rule's own calendar, then converts each date to a Gregorian `IcalRecurDateTime`. Time of day and weekdays are scale-independent and stay where they are. The rest of the crate never sees a non-Gregorian date.
2. Calendar arithmetic from icu4x's `icu_calendar`, the Rust counterpart of what RFC 7529 5 points at, rather than our own: Hebrew is small enough to write, but Chinese and Dangi need astronomical tables, and Islamic variants differ per variant. To verify before choosing: that it builds `no_std`, its MSRV, its dependency weight with compiled data, the range of years it covers for Chinese, and that every CLDR calendar RFC 7529 names maps to one it has. If it fails those, scope down to the arithmetic calendars (Hebrew, Islamic tabular, Coptic, Ethiopic, Indian, Persian) written here, and keep Chinese yielding nothing.
3. `by_month` becomes a type carrying a month number and a leap flag, parsed for every scale and range-checked against the scale at expansion. **Breaking**, so it waits for the next minor release.
4. An RSCALE corpus crossed with libical+ICU, the one oracle there is (dateutil does not parse `RSCALE`), frozen like the SKIP corpus.

## Consequence

A second optional dependency in the model layer, after the planned jiff conversion; the bare core stays dependency-free. Recommended to land last of the five: it is the largest, the only one with a breaking change, and the rarest in real calendars.
