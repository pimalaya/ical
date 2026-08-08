---
cairn: log
change: missing-standards
landed: 2026-08-08
---

# The missing standards

Three standards joined the vocabulary, each following the pattern the existing seventy properties already use: a lens marker, a spec entry, a place in the `ALL` list, and a fixture.

**RFC 7953, availability.** `VAVAILABILITY` and `AVAILABLE` are components now, with `VAVAILABILITY` nesting `AVAILABLE` and `VCALENDAR` nesting `VAVAILABILITY`, so the nesting check that landed with validation knows where they belong. `BUSYTYPE` came with them.

**RFC 9253, relationships.** `LINK`, `REFID` and `CONCEPT` as properties, `LINKREL` and `GAP` as parameters, with `LINKREL` on `LINK` and `GAP` beside `RELTYPE` on `RELATED-TO`.

**RFC 6638, CalDAV scheduling.** `SCHEDULE-AGENT`, `SCHEDULE-FORCE-SEND` and `SCHEDULE-STATUS`, allowed on `ATTENDEE`, which meant writing out that property's parameter set rather than leaving it on the common default. This is the group that matters most: it is scheduling on a real server, and therefore Calendula's path.

A fixture, `tests/corpus/rfc/rfc7953_availability.ics`, carries all three at once: an availability window with its available period, and an event with a link, a reference, a concept, a gapped relation and a fully scheduled attendee. It goes through every sweep the other fixtures do, the round-trip, the recovering parse, the calcard cross-check and the jCal round-trip, so the new vocabulary is exercised by five tests rather than by one.

Capabilities moved: `decoded-model` (MODIFIED: one model, every version, now naming the three new RFCs).
