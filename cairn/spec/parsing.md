---
cairn: spec
capability: parsing
status: current
---

# Parsing

The byte-faithful side of the crate, gated behind the `parser` feature. `IcalCst` models a component as an optional `BEGIN` / `END` envelope wrapping an ordered body of property lines and nested components, in source order. It knows nothing of what a property means: it reproduces the wire.

Parsing is maximally liberal, per Postel's law. Any real calendar is accepted, including components, properties, parameters and value types no version defines. Strictness lives only on the way out (see [conformance](./conformance.md)).

### Requirement: Round-trip fidelity

The parser SHALL preserve every byte of a parsed calendar, its folds, its blank lines and its QUOTED-PRINTABLE soft breaks included, so serializing an unedited calendar reproduces the input exactly, and editing one property leaves every other byte intact.

Everything survives byte for byte: line endings, parameter order, casing, whitespace inside a value, unknown vocabulary, and the wire layout itself.

A file holding several calendars round-trips through the multi-calendar entry point, whose output concatenates to the input. The single-calendar entry point reads the first calendar and stops, so anything after it is not part of what it returns.

#### Scenario: An unedited calendar
- GIVEN any calendar, however it is laid out
- WHEN it is parsed and serialized
- THEN the output is byte-identical to the input

#### Scenario: A folded real-world calendar
- GIVEN a calendar folded at 75 octets, with blank lines between components
- WHEN it is parsed and serialized
- THEN the output is byte-identical to the input

An edit through a cursor that names a component rewrites that component alone. An edit through one that names none replaces the whole value, being the inverse of the reader that names none.

#### Scenario: One edited property
- GIVEN a parsed, folded calendar
- WHEN one property value is set through its lens cursor
- THEN every line but the edited one keeps its original wire shape

### Requirement: A truncating read names the component it truncates at

A value node SHALL read the whole value through readers that take no index (`decode`, `decode_list`, `decode_bytes`), keeping every `;` the value carries literal, and SHALL read one `;`-component only through readers that name it (`decode_component`, `decode_component_list`). No reader SHALL cut a value at both a `;` and a `,`.

The un-indexed writers (`set`, `set_bytes`) SHALL replace the whole value, so a value read whole and written back comes back unchanged. The component writers (`set_component`, `set_component_bytes`) SHALL rewrite nothing but the component they name.

The value cursor SHALL follow the same split: `text`, `bytes`, `list` and their setters address the whole value, `component` and `set_component` address one slot.

#### Scenario: A description read past its first semicolon
- GIVEN a calendar carrying `DESCRIPTION:a;b`
- WHEN the value is read through its lens cursor
- THEN it reads `a;b` rather than stopping at the semicolon

#### Scenario: A value read whole and written straight back
- GIVEN a value of several `;`-components
- WHEN it is read whole and written back through the un-indexed setter
- THEN it reads back as it went in, with no component of the old value left behind

### Requirement: Serialization fixpoint

Serializing a parsed calendar SHALL produce bytes that reparse to the same bytes, whatever the input, so output is always stable under a second pass.

#### Scenario: A folded input
- GIVEN a calendar folded at 75 octets
- WHEN it is parsed, serialized, and the output reparsed and serialized again
- THEN the two outputs are byte-identical

### Requirement: Line normalisation

The parser SHALL resolve the wire shape of a line into logical content for every layer above it, and SHALL restore that shape on output:

- RFC 5545 3.1 folded continuation lines, unfolded to their logical content, with the break and its folding whitespace recorded.
- Blank lines before a content line, recorded against the line that follows them, and after the last one, recorded on the calendar.
- QUOTED-PRINTABLE soft line breaks (a value ending in `=` under `ENCODING=QUOTED-PRINTABLE`), joined, with a dangling trailing `=` recorded rather than lost.
- A missing final line break, accepted, with the line kept whole and no break invented.

A recorded shape SHALL go back out in offset order, whichever of the tokeniser and the line splitter recorded each piece, and two pieces recorded at one offset SHALL keep the order they were recorded in. A value ending on two `=` is recorded by both at once, the soft break past the last logical byte and the dangling `=` before it, and emitting them in list order writes a line break into the middle of the value.

An edited value SHALL drop the recorded shape of its own line rather than re-apply fold points that no longer match its bytes. A line whose length is unchanged keeps its shape, since every offset still indexes what it did.

#### Scenario: A folded line
- GIVEN `NOTE:foo\r\n bar\r\n`
- WHEN it is parsed
- THEN the line holds the logical value `foobar` and serializes folded exactly as it arrived

#### Scenario: A value ending on two soft-break markers
- GIVEN a `QUOTED-PRINTABLE` line whose value ends `x==`
- WHEN it is parsed and serialized
- THEN the output is the input, and it reparses to the same bytes rather than swallowing the line after it

### Requirement: Raw value bytes

A property value SHALL be kept as raw bytes, so a value in a foreign `CHARSET` survives unaltered. A property name and its parameters MUST be valid UTF-8, as every version's grammar guarantees; a non-UTF-8 name or parameter is a parse error.

#### Scenario: A foreign charset value
- GIVEN a property whose value is not valid UTF-8
- WHEN the calendar is parsed and serialized
- THEN the value's bytes are unchanged

### Requirement: Strict parse refusal

The default parse entry point SHALL refuse a calendar it cannot structure, rather than guess: a content line with no colon, a component with no `END`, a non-UTF-8 header, or an input holding no content line at all.

#### Scenario: A missing END
- GIVEN a calendar whose `VEVENT` is never closed
- WHEN it is parsed
- THEN parsing fails with `MissingEnd` and no partial calendar is returned

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

### Requirement: Multi-calendar input

The parser SHALL iterate every top-level calendar in a file lazily, one item per calendar, skipping blank lines between them.

### Requirement: Envelope-less records

An input whose first line is not `BEGIN` SHALL parse as a bare record, every line a property, so a lone component fragment round-trips.

### Requirement: The real-world corpus is swept

Every committed real-world fixture SHALL parse, serialize to a fixpoint, decode without panicking, and survive a decode, encode and decode again unchanged. Each source directory SHALL carry its own attribution and an asserted fixture count, so a misfiled, renamed or newly added fixture is caught.

#### Scenario: A vendor calendar the parser cannot structure
- GIVEN a fixture the strict parser refuses
- WHEN the sweep runs
- THEN the fixture is classified as refused, not silently skipped

### Requirement: A quoted parameter value is opaque

The line splitter SHALL treat a double-quoted parameter value as opaque, per RFC 5545 section 3.2: neither the `:` separating the head from the value nor the `;` separating one parameter from the next is recognised inside one.

A head carrying an unbalanced quote SHALL still parse: with no `:` outside quotes the splitter falls back to the first `:` anywhere, so a malformed line yields a line rather than an error.

#### Scenario: The RFC 5545 section 3.2.1 alternate representation
- GIVEN a line reading `DESCRIPTION;ALTREP="cid:part1.0001@example.org":Meeting notes`
- WHEN it is parsed
- THEN it carries one parameter, `ALTREP` holding the whole quoted URI, and the value reads `Meeting notes`

#### Scenario: An unbalanced quote
- GIVEN a line reading `ATTENDEE;CN="Ada:mailto:ada@example.com`
- WHEN it is parsed
- THEN it parses at the first colon anywhere and round-trips unchanged

### Requirement: A written value never breaks its line

Serializing a value SHALL NOT emit a byte that ends the line it sits on, whatever the caller wrote into it. A newline is the one such byte the escapes exist for, and every version SHALL write it escaped.

vCalendar 1.0 has no newline escape, so a newline written into a 1.0 value SHALL go out as `\n` and read back as those two characters. That is the closest versit can carry, and the alternative is a calendar its own parser refuses.

#### Scenario: A newline set on a vCalendar 1.0 property
- GIVEN a 1.0 calendar and a caller setting a value holding a newline
- WHEN the calendar is serialized
- THEN the property is still one line and the calendar parses
