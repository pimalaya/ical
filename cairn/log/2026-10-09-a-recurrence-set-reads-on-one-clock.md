---
cairn: log
change: a-recurrence-set-reads-on-one-clock
date: 2026-10-09
---

# A recurrence set reads on one clock

`IcalRecurSet` compared every date as the civil time it spells. The Android app found three ways that goes wrong against a zoned series: a UTC `UNTIL`, which RFC 5545 3.3.10 requires beside a zoned `DTSTART`, ended the series early (`DTSTART;TZID=Europe/Paris:20260706T090000` with `UNTIL=20260708T070000Z` lost the 09:00 instance on 8 July); an `EXDATE` in UTC or another zone removed nothing; and a `RECURRENCE-ID` in UTC named no instance, so the override showed beside the instance it meant to replace. The app rebuilt the whole set by hand, reading each override through a scratch set since `with_override` had no way to take an identity on the series' clock.

The set now holds the clock its times are told on, `IcalRecurSet::zone`, an `IcalRecurZone` read from `DTSTART` (floating, UTC, or the `TZID`). A time written on another clock is brought onto it while the set is read, through the `VTIMEZONE`s defining both, as a written `DATE-TIME` (3.3.5, `IcalTzOffset::literal_instant` there, `IcalTz::local` back): the `UNTIL` of a rule when written in UTC, every `RDATE` and `EXDATE` item, and an override's `RECURRENCE-ID` and `DTSTART`.

- `IcalRecurSet::of_uid` finds the zones among the components it is given and reads the series before its overrides, whatever order the calendar lists them in.
- `IcalRecurSet::of_component_in` and `with_override_in` take the zones; `of_component` and `with_override` are the same with none, and compare as written, as before.
- `IcalRecurOverride::of_component` reads one override onto a given clock with no set, which is what the app's scratch set was for: it can keep which component replaced which identity.

The conversion sits on reading, not on `expand_in_zone`: the set's fields stay civil times, and once read, which clock each one was written on is gone. `expand_in_zone` stays the 3.3.10 gap filter. A zone the calendar does not define, or a floating clock on either side, leaves the time as written.

`IcalRecurSet` gains a public field, so a struct literal naming every field needs `zone` (or `..Default::default()`).

Capabilities moved: recurrence.
