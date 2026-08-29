---
cairn: delta
change: a-write-back-never-breaks-the-line
---

## ADDED Requirements

### Requirement: A written value never breaks its line

Serializing a value SHALL NOT emit a byte that ends the line it sits on, whatever the caller wrote into it. A newline is the one such byte the escapes exist for, and every version SHALL write it escaped.

vCalendar 1.0 has no newline escape, so a newline written into a 1.0 value SHALL go out as `\n` and read back as those two characters. That is the closest versit can carry, and the alternative is a calendar its own parser refuses.

#### Scenario: A newline set on a vCalendar 1.0 property
- GIVEN a 1.0 calendar and a caller setting a value holding a newline
- WHEN the calendar is serialized
- THEN the property is still one line and the calendar parses

## MODIFIED Requirements

### Requirement: The merged calendar can always be read back

Whatever the three calendars hold, the merged calendar SHALL parse, and SHALL reparse to the same bytes. A merge never emits a calendar its own parser refuses, and never emits one that loses content on the next read.

`BEGIN` and `END` are the component envelope rather than properties, whichever side carries them. A bare, envelope-less record, which the parser accepts so a lone fragment round-trips, SHALL therefore contribute its properties alone: no side is reported as adding or removing a structural line, and none is ever copied into the merged calendar.

A line copied out of one side SHALL carry a line ending. The last line of a truncated download has none, and copied into the middle of a calendar it would swallow the line after it. The untouched bytes of the baseline side are not affected: only what the replay copies is terminated.

What the replay writes into a line SHALL be the bytes the side that wrote them wrote, never a re-encoding of their decoded form. The two are not the same string: decoding a parameter resolves the value escapes and encoding one does not put them back, so a re-encoded parameter can carry a line break into a head. The decoded form is what the sides are compared on, and what is reported; it is not what is written.

#### Scenario: A bare record as one side

- GIVEN a well-formed base and left side, and a right side that is an envelope-less fragment holding a `BEGIN` line
- WHEN they are merged
- THEN the merged calendar parses and reparses to itself

#### Scenario: A parameter holding an escape

- GIVEN a right side that changed a parameter whose value holds a `\n`
- WHEN they are merged
- THEN the merged line carries the parameter as the right side wrote it and the calendar parses

## REMOVED Requirements
