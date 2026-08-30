---
cairn: change
id: a-grammar-belongs-to-its-value
status: landed
created: 2026-08-30
---

# A grammar belongs to its value

## Why

A `DURATION` and a `TZOFFSETTO` are kept as raw text, which is what byte-faithful round-tripping needs, and the value types offered no way to read them as numbers.

Every consumer that needed one therefore wrote its own parser. `timezone.rs` had a private `parse_offset`, the JSCalendar import a private `seconds`, and the JSCalendar export a private `duration` writing the same grammar back. Three copies of two grammars, none of them reachable by a caller of this crate, who is left doing the same thing a fourth time.

## What

Put each grammar on the value type that owns it: `IcalUtcOffset::seconds`, `IcalDuration::seconds` and `IcalDuration::from_seconds`, all public.

Both are total in the sense that matters here: neither grammar carries a month or a year, so no calendar is needed to say how long one is, and the answer is a plain number of seconds. The raw text stays the value, so nothing about round-tripping changes.

The three private copies go, and their callers ask the value.

## Consequence

A caller holding an `IcalDuration` can now say how long it is, which it could not before, and the crate stops re-deriving one grammar per consumer.
