---
cairn: change
id: where-a-vtimezone-ends
status: landed
created: 2026-10-29
---

# Where a VTIMEZONE ends

## Why

A calendar's `VTIMEZONE` wins over the database from its earliest observance on, and before it a known name reads through the database (RFC 7808 7.1, landed in [timeline-oracle-second-round](../archive/timeline-oracle-second-round/proposal.md)). The other end is not read:

- `TZUNTIL` (RFC 7808 7.1), the producer's own statement that the definition stops, is ignored. It is not even a known property kind in src/.
- A `VTIMEZONE` truncated to the years its producer needed, with dated observances only and no `TZUNTIL`, holds its last offset forever. That is the one timeline divergence about instants rather than value types: fablab_cottbus.ics states Europe/Berlin onsets up to 2019-03-31, so every later winter reads at +0200, an hour off. The producer (the All-in-One Event Calendar WordPress plugin) is widespread, so this is not one calendar's problem.

## What

1. Read `TZUNTIL` into `IcalTz`. Past it, a name the database knows reads through the database; an unknown name keeps today's reading (the last offset), the feature off too.
2. Complete a finite `VTIMEZONE` forward the way `completed` already completes it backward: when no observance recurs without end (no `RRULE`, or every `RRULE` bounded by `UNTIL` or `COUNT`) and the database knows the name, add the database's onsets after the latest one the calendar states. A zone that really stopped changing gets nothing added, since the database has no later onsets either, so the answer moves only for a truncated definition.

Point 2 reverses what was decided in timeline-oracle-second-round, where fablab was kept as a divergence on the reading that a `VTIMEZONE` with no `TZUNTIL` is valid indefinitely. Recommended anyway: it is the same truncation the past completion already accepts, the risk (a deliberately frozen custom definition under an IANA name, past its last onset) is narrow, and the cost of the strict reading is real events an hour off. Point 1 alone is uncontroversial and can land without point 2.

Before deciding point 2, check RFC 5545 3.6.5 for what it says about a `VTIMEZONE` holding only the observances its date-times need.

## Consequence

The fablab line leaves divergence.tsv for agreement.tsv. Behaviour change only behind `tzdb`; the bare core keeps today's reading.
