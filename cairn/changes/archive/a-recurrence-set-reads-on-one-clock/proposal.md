---
cairn: change
id: a-recurrence-set-reads-on-one-clock
status: landed
created: 2026-10-09
---

# A recurrence set reads on one clock

## Why

`IcalRecurSet` read every date as the civil time it spells, which is right while they share a zone and wrong when they do not. A UTC `UNTIL` against a start in Paris, as RFC 5545 3.3.10 requires it to be written, ended the series an instance early (`DTSTART;TZID=Europe/Paris:20260706T090000` with `UNTIL=20260708T070000Z` lost 8 July), and an `EXDATE` or a `RECURRENCE-ID` written in UTC or another zone named no instance, so an override showed beside the instance it replaced. The Android app (rust/src/calendar/series.rs, `set_of`) rebuilt the rules, the dates and every override by hand, and read each override through a scratch set because `with_override` could not take an identity already told on the series' clock.

## What

The set holds the clock its times are told on (`IcalRecurSet::zone`, an `IcalRecurZone`: floating, UTC or a `TZID`), and brings every time written on another clock onto that one as it is read, through the `VTIMEZONE`s that define both: `UNTIL`, `RDATE`, `EXDATE`, and an override's `RECURRENCE-ID` and `DTSTART`, each read as a written `DATE-TIME` (3.3.5). `of_uid` finds the zones among the calendar's components; `of_component_in` and `with_override_in` take them; `IcalRecurOverride::of_component` reads one override onto a given clock without a set. Without the zones, the comparison stays as written.

The conversion happens while reading, not in `expand_in_zone`: the set's fields are civil times, and by then which clock each was written on is gone. `expand_in_zone` stays the 3.3.10 gap filter.
