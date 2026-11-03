---
cairn: delta
change: a-folded-line-is-stripped-once-assembled
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Line normalisation

The parser SHALL resolve the wire shape of a line into logical content for every layer above it, and SHALL restore that shape on output:

- RFC 5545 3.1 folded continuation lines, unfolded to their logical content, with the break and its folding whitespace recorded.
- Blank lines before a content line, recorded against the line that follows them, and after the last one, recorded on the calendar.
- QUOTED-PRINTABLE soft line breaks (a value ending in `=` under `ENCODING=QUOTED-PRINTABLE`), joined, with a dangling trailing `=` recorded rather than lost.
- A missing final line break, accepted, with the line kept whole and no break invented.

The leading whitespace SHALL be stripped from the assembled logical line, not only from its first physical line, so a whitespace-only line whose continuation carries more than the one fold marker does not leave the leftover in front of the name.

A recorded shape SHALL go back out in offset order, whichever of the tokeniser and the line splitter recorded each piece, and two pieces recorded at one offset SHALL keep the order they were recorded in. A value ending on two `=` is recorded by both at once, the soft break past the last logical byte and the dangling `=` before it, and emitting them in list order writes a line break into the middle of the value.

An edited value SHALL drop the recorded shape of its own line rather than re-apply fold points that no longer match its bytes, and the line SHALL then be laid out as an encoded one is (see below). A line whose length is unchanged keeps its shape, since every offset still indexes what it did.

#### Scenario: A folded line
- GIVEN `NOTE:foo\r\n bar\r\n`
- WHEN it is parsed
- THEN the line holds the logical value `foobar` and serializes folded exactly as it arrived

#### Scenario: A continuation of a whitespace-only line
- GIVEN the input `"   \r\n  A:b\r\n"`
- WHEN it is parsed
- THEN the line is named `A`, not `" A"`, and it serializes back to the input

#### Scenario: A value ending on two soft-break markers
- GIVEN a `QUOTED-PRINTABLE` line whose value ends `x==`
- WHEN it is parsed and serialized
- THEN the output is the input, and it reparses to the same bytes rather than swallowing the line after it

## REMOVED Requirements
