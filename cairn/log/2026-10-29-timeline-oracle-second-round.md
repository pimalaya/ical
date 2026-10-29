---
cairn: log
change: timeline-oracle-second-round
date: 2026-10-29
---

# The timeline oracle, second round

Three more differences against recurring-ical-events, all in components with no `UID` that the first round made reachable, each settled toward the oracle:

- `tzdb::zone` reads a Windows zone name through src/tzdb/windows.rs, generated from CLDR release-49 windowsZones.xml (territory 001, 142 names, Unicode License v3). `is_known` and `vtimezone` are unchanged.
- `IcalRecurSet` no longer yields its `DTSTART` when every rule ends on a day before it, the shape of a series deleted from its first instance on.
- `rule_of` falls back to `parse_liberally`, which keeps every part that parses; `IcalValidateError::RuleSyntax` reports what it dropped, where validation used to return early and say nothing about a rule it could not parse.

A truncated `VTIMEZONE`'s future (no `TZUNTIL`) stays as it was and is recorded as a divergence.

Four tests: three in tests/recur_instant.rs, each failing with its behaviour switched off (a date `UNTIL` on the start's day is the guard that passes either way), and one in tests/validate.rs.

Capabilities moved: **recurrence** (a series cut before its start, liberal parts in a set), **tz** (Windows zone names), **conformance** (an unparsable rule is reported).
