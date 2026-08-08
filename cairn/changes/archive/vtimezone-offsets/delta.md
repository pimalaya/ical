---
cairn: delta
change: vtimezone-offsets
---

## ADDED Requirements
### Requirement: Offset resolution from the calendar's own rules

A civil date and time plus a `VTIMEZONE` SHALL resolve to a UTC offset, using only the observances that travel inside the calendar. No time-zone database and no new dependency SHALL be required.

#### Scenario: An unambiguous local time
- GIVEN a `VTIMEZONE` with a `STANDARD` and a `DAYLIGHT` observance
- WHEN a civil time in mid-July is resolved
- THEN the daylight observance's `TZOFFSETTO` is returned

### Requirement: The gap and the fold are reported

A local time a spring-forward skips SHALL be reported as skipped, and a local time a fall-back gives twice SHALL be reported as ambiguous with both candidate offsets. Neither SHALL be silently resolved to one answer.

#### Scenario: A skipped local time
- GIVEN a transition that jumps 02:00 to 03:00
- WHEN 02:30 is resolved
- THEN the result reports a gap, naming the offsets either side

#### Scenario: A repeated local time
- GIVEN a transition that repeats 02:00 to 03:00
- WHEN 02:30 is resolved
- THEN the result reports a fold, carrying both the earlier and the later offset
