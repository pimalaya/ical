---
cairn: change
id: a-time-zone-is-named-tz
status: landed
created: 2026-08-30
---

# A time zone is named tz

## Why

The time-zone layer named itself three different ways: the module was `timezone`, its types were `IcalTimezone`, `IcalObservance` and `IcalOffset`, and the wire names it reads are all `TZ`-prefixed (`TZID`, `TZNAME`, `TZOFFSETFROM`).

`IcalOffset` was the worst of the three. It is not an offset: it is the answer to a resolution, which may be one offset, a gap where the clock jumped, or a fold where it repeated. Beside `IcalUtcOffset`, which is the wire value, the name said the opposite of what it meant.

## What

The module becomes `tz` and its types `IcalTz`, `IcalTzObservance` and `IcalTzOffset`, so everything the layer owns is scoped by the same three letters the RFC uses.

`Observance` stays. It is not a word chrono, jiff or the tz database use, but it is RFC 5545 3.6.5's own word for one such rule, reused by RFC 7808, and no datetime library's word fits: a `FixedTimespan` is one span where an observance is a rule generating many, and `TimeZoneTransition` names the instants the rule generates, which is what the private `Transition` in this module already is. The header now says so, so the choice is not re-litigated.

## Consequence

A breaking rename with no behaviour change.
