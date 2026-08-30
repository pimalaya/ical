---
cairn: log
change: a-grammar-belongs-to-its-value
date: 2026-08-30
---

# A grammar belongs to its value

`IcalDuration` and `IcalUtcOffset` keep their raw text, which is what byte-faithful round-tripping needs, and offered no way to read that text as a number. Every consumer that needed one wrote its own parser: `timezone.rs` had `parse_offset`, the JSCalendar import had `seconds`, the JSCalendar export had `duration` writing the same grammar back. Three private copies of two grammars, none of them reachable by a caller, who was left writing a fourth.

The grammars moved onto the values that own them: `IcalUtcOffset::seconds`, `IcalDuration::seconds` and `IcalDuration::from_seconds`, all public. Neither grammar carries a month or a year, so the answer is a plain number of seconds and no calendar is needed to reach it. A week comes back spelled in days, since `P7D` and `P1W` are the same length and only one survives a round trip through a number.

The three private copies are gone and their callers ask the value. The raw text is still the value, so nothing about round-tripping changed.

Capabilities moved: decoded-model.
