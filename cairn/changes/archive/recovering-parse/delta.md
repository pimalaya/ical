---
cairn: delta
change: recovering-parse
---

## ADDED Requirements
### Requirement: Recovering parse

A recovering parse entry point SHALL accept any input, keeping a physical line it cannot structure as an opaque item that round-trips byte for byte, and SHALL report every such line to the caller. A component left unclosed at end of input SHALL be closed with no `END` line, so its bytes still round-trip.

The strict entry point stays the default and its refusals are unchanged.

#### Scenario: A line with no colon
- GIVEN a calendar carrying one line with no colon
- WHEN it is parsed by the recovering entry point
- THEN the calendar parses, the line round-trips unchanged, and it is reported as recovered

#### Scenario: A component with no END
- GIVEN a calendar whose `VEVENT` is never closed
- WHEN it is parsed by the recovering entry point
- THEN the event is closed at end of input, serialization reproduces the input, and the missing `END` is reported
