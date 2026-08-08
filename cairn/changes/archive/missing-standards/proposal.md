---
cairn: change
id: missing-standards
status: landed
created: 2026-08-08
---

# The missing standards

## Why

Three standards the crate's vocabulary does not yet cover. The CalDAV scheduling parameters matter most, since they are scheduling on a real server and therefore Calendula's path.

## What

- VAVAILABILITY and AVAILABLE, RFC 7953.
- `LINK`, `REFID` and `CONCEPT`, with the `RELTYPE` and `GAP` extensions, RFC 9253.
- The CalDAV scheduling parameters `SCHEDULE-AGENT`, `SCHEDULE-FORCE-SEND` and `SCHEDULE-STATUS`, RFC 6638.

Each is a spec entry plus a lens marker, the pattern the existing 70 properties already follow.

Done when each new property, parameter and component carries its spec, appears in the relevant `ALL` list, and is covered by a fixture.
