---
cairn: delta
change: structural-lines-are-not-properties
---

## ADDED Requirements

### Requirement: The merged calendar can always be read back

Whatever the three calendars hold, the merged calendar SHALL parse, and SHALL reparse to the same bytes. A merge never emits a calendar its own parser refuses, and never emits one that loses content on the next read.

`BEGIN` and `END` are the component envelope rather than properties, whichever side carries them. A bare, envelope-less record, which the parser accepts so a lone fragment round-trips, SHALL therefore contribute its properties alone: no side is reported as adding or removing a structural line, and none is ever copied into the merged calendar.

A line copied out of one side SHALL carry a line ending, since the last line of a truncated download has none and would swallow the line it lands in front of.

#### Scenario: A bare record as one side

- GIVEN a well-formed base and left side, and a right side that is an envelope-less fragment holding a `BEGIN` line
- WHEN they are merged
- THEN the merged calendar parses and reparses to itself
