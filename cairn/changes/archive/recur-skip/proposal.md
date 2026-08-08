---
cairn: change
id: recur-skip
status: landed
created: 2026-08-08
---

# RFC 7529 SKIP, for the Gregorian scale

## Why

RFC 7529 is two features in one document. `SKIP` says what a rule means when the date it names does not exist, and needs nothing but the month lengths this crate already has. Non-Gregorian calendar systems need the CLDR arithmetic the RFC itself points implementers at ICU for. The crate parsed and stored both and expanded neither, so `RSCALE=GREGORIAN;FREQ=YEARLY;SKIP=FORWARD` on a leap-day start produced the leap-years-only answer that the parameter exists to fix.

## What

Expand `SKIP` for the Gregorian scale, report it when no `RSCALE` accompanies it, and restate the non-Gregorian claim so it cannot be read as more than it is. Freeze the answers against libical, which resolves `SKIP` without an ICU build.
