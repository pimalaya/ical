---
cairn: change
id: structural-lines-are-not-properties
status: landed
created: 2026-08-29
---

# A merge never emits a calendar its own parser refuses

## Why

`IcalCst::parse` accepts a bare, envelope-less record by design, so that a lone component fragment round-trips, and `parse_recovering` produces the same shape from a truncated download. In such a record a `BEGIN` or an `END` line is a property like any other. The merge copies an added property verbatim, so a structural keyword can be spliced into the middle of a well-formed calendar. The result is bytes this crate cannot read back, with an empty report, or, for a stray `END`, bytes that silently drop everything after it on the next parse.

A synchronisation engine that hands the merge whatever a server returned can be given a bare record, so this is not only a fuzzing curiosity. And the property behind it is stronger than the bug: whatever three calendars go in, what comes out must parse, and must reparse to itself.

## What

The merge treats `BEGIN` and `END` as the envelope rather than as properties. They are invisible to the diff, so no side is ever reported as having added or removed one, and invisible to the replay, so none is ever copied. A bare record contributes its real properties and nothing else.

A line the replay copies also gets a line ending where the side it came from had none, since the last line of a truncated download carries none and would otherwise swallow the line it lands in front of.

The invariant becomes a law rather than a repair: the merged calendar parses, and reparses to the same bytes. It is asserted by the generated property suite, by the corpus harness over every fixture, and by the fuzz target's oracle, so no future change can reintroduce the class quietly.
