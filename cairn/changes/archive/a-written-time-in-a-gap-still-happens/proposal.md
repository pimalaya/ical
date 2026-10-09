---
cairn: change
id: a-written-time-in-a-gap-still-happens
status: landed
created: 2026-10-09
---

# A written time in a gap still happens

## Why

`IcalTzOffset::instant` answers a local time in a gap with no instant. That is RFC 5545 3.3.10's answer for an instance a rule generates, but RFC 5545 3.3.5 reads a written `DATE-TIME` in a gap at the offset before it: such a time still happens. The Android app resolved every literal time by hand (`Zones::offset` in rust/src/calendar/zone.rs), and turned an instant back into a local time with a two-step guess, since the crate had no inverse.

## What

Name the literal reading beside the generated one on `IcalTzOffset`: `literal_offset` and `literal_instant`, and document on each which caller wants it. Add `IcalTz::local`, the local time a zone shows at an instant, which is never ambiguous.
