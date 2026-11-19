---
cairn: change
id: client-calendars-round-trip
status: landed
created: 2026-11-18
---

# Client calendars round-trip

## Why

Pimalaya Server converts every calendar it stores between iCalendar (CalDAV) and JSCalendar (JMAP). A conversion that loses a line loses user data. The JSCalendar escape hatch already carried unknown properties and parameters, but nothing checked a calendar as real clients write it coming back line for line: only the JSON side was held to a fixpoint.

## What

Realistic client calendars under tests/corpus/clients (iOS, Exchange, Google, Thunderbird) held to a line-for-line round trip in RFC 8984 and in 2.0, and the two losses that check found fixed: a `DTSTAMP` beside a `LAST-MODIFIED` was dropped, and a UTC `UNTIL` came back local, moving the end of a zoned series by the zone's offset.
