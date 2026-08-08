---
cairn: change
id: vtimezone-offsets
status: landed
created: 2026-08-08
---

# VTIMEZONE offset resolution

## Why

Expansion is civil by design and that boundary stays, but somebody has to turn a civil time into an instant, and today nobody can. A caller holding an occurrence and a `TZID` has no way to get a UTC offset out of this crate.

## What

An offset lookup built on what the crate already has: the `VTIMEZONE`, `STANDARD` and `DAYLIGHT` components, the `TZOFFSETFROM` and `TZOFFSETTO` properties, and an `RRULE` expander. It needs no time-zone database and no new dependency, since the rules travel inside the calendar. It must answer the two hard cases explicitly rather than guess: the local time a spring-forward skips, and the one a fall-back gives twice.

Done when a civil occurrence plus the `VTIMEZONE` of its own calendar yields a UTC offset, with the gap and the fold reported rather than guessed, and when the RFC 5545 timezone fixture drives the tests.
