---
cairn: log
change: a-time-zone-is-named-tz
date: 2026-08-30
---

# A time zone is named tz

The layer named itself three ways: a `timezone` module, `IcalTimezone` / `IcalObservance` / `IcalOffset` types, and `TZ`-prefixed wire names throughout. It is now `tz`, `IcalTz`, `IcalTzObservance` and `IcalTzOffset`.

`IcalOffset` was the one that actually misled. It is not an offset. It is the answer to a resolution, which may be one offset, a gap the clock jumped over, or a fold it repeated, and it sat beside `IcalUtcOffset`, which is the wire value. Its documentation now says which of the two it is.

"Observance" stayed, and the header now says why so the question is not reopened. It is not a word chrono, jiff or the tz database use, but it is RFC 5545 3.6.5's own word for one such rule, reused by RFC 7808, and no datetime library's word fits: chrono-tz's `FixedTimespan` is one span where an observance is a rule generating many, and jiff's `TimeZoneTransition` names the instants such a rule generates, which is exactly what the private `Transition` in this module already is.

A breaking rename with no behaviour change. The spec file moved with it.

Capabilities moved: tz (renamed from timezone).
