---
cairn: change
id: until-on-the-series-clock
status: landed
created: 2026-11-19
---

# UNTIL on the series' clock

## Why

A JMAP calendar backend converts every event between iCalendar and JSCalendar. RFC 5545 3.3.10 states an `RRULE`'s `UNTIL` in UTC beside a zoned `DTSTART`, RFC 8984 4.3.3 states it as a local date-time in the object's `timeZone`. client-calendars-round-trip shifted it only on a `VTIMEZONE` the calendar itself carries, which a JMAP object almost never has: a zoned series stored with a UTC `UNTIL` still moved its end by the zone's offset, on read and on write, gaining or losing its last instance. The import also wrote a floating object's `UNTIL` in UTC and an all-day object's as a UTC date-time, both against RFC 5545 3.3.10.

## What

Behind the `tzdb` feature, tell `UNTIL` between UTC and the series' zone on the bundled database when the calendar defines no `VTIMEZONE` for it (a calendar's own definition completed from the database as `tzdb::zones` does), both directions, lone objects included. Write a floating `UNTIL` floating and an all-day one as a `DATE`. Without the feature, keep the unshifted digits and say so.
