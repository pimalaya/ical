---
cairn: spec
capability: tz
status: current
---

# Time zones

Turning a civil date-time into a UTC offset, using only the `VTIMEZONE` the calendar carries. Expansion stays civil ([recurrence](./recurrence.md)); this is the step after it. A `VTIMEZONE` is read into its `STANDARD` and `DAYLIGHT` observances, each with the offset before it, the offset after it, and the recurrence set saying when it takes effect. Built on the rule expander, and available with it.

### Requirement: Offset resolution from the calendar's own rules

A civil date and time plus a `VTIMEZONE` SHALL resolve to a UTC offset, using only the observances that travel inside the calendar. No time-zone database and no new dependency SHALL be required.

Every `DTSTART` inside an observance is local to the offset *before* the transition (RFC 5545 3.6.5). A local time before the first transition SHALL take the offset that transition states came before it. A zone with no observance at all SHALL resolve to UTC, since it states no offset to apply.

#### Scenario: An unambiguous local time
- GIVEN a `VTIMEZONE` with a `STANDARD` and a `DAYLIGHT` observance
- WHEN a civil time in mid-July is resolved
- THEN the daylight observance's `TZOFFSETTO` is returned

#### Scenario: A zone that never shifts
- GIVEN a `VTIMEZONE` whose single observance moves the clock by nothing
- WHEN any civil time is resolved
- THEN that observance's offset is returned

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
