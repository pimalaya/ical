---
cairn: change
id: recovering-parse
status: landed
created: 2026-08-08
---

# A recovering parse mode

## Why

One content line without a colon, or one missing `END`, loses the whole calendar. Twelve of the 191 real-world fixtures are rejected that way, two of them ical4j exporter samples. The crate's stated posture is that any real calendar is accepted. libical keeps going and records `X-LIC-ERROR`; ical4j has a relaxed mode.

## What

Add a parse entry point that keeps a malformed physical line as an opaque leaf and carries on, so the rest of the calendar survives and the line still round-trips byte for byte. The strict entry point stays the default, and a recovered calendar reports the lines it could not structure.

Done when the twelve refused fixtures parse under the recovering entry point, round-trip, and report their bad lines.
