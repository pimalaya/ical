---
cairn: change
id: a-zone-change-contests-a-time-change
status: landed
created: 2026-10-09
---

# A zone change contests a time change

## Why

From the Android calendar's conflict work: base `DTSTART:20260105T090000`, one side `DTSTART:20260105T080000`, the other `DTSTART;TZID=/example.org/Romance:20260105T100000`. The merge wrote `DTSTART;TZID=/example.org/Romance:20260105T080000`, a time neither side wrote. The judge let a `VALUE` change contest a value change, and no other parameter, though `TZID` changes what the value means just as much (RFC 5545 3.2.19).

## What

A parameter that says what its value means contests a concurrent value-level change, both ways. Audit RFC 5545 and 7986 (and the extensions the crate covers) for which those are: how the value is read (`VALUE`, `TZID`, `ENCODING`, `CHARSET`, `LANGUAGE`, `FMTTYPE`) and what it denotes (`RANGE`, `RELATED`, `FBTYPE`, `RELTYPE`, `LINKREL`). A parameter describing the thing the value names (`CN`, `PARTSTAT`, `ROLE`, `ALTREP`, `DISPLAY`, `EMAIL`, `FEATURE`, `LABEL`, `GAP`) still merges beside it.
