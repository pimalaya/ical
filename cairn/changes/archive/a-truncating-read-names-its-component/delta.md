---
cairn: delta
change: a-truncating-read-names-its-component
---

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: A single value is not split on a separator it does not own

A property whose value is one text, one URI or one scalar SHALL keep an unescaped `,` and an unescaped `;` as data. RFC 5545 3.3.11 says a text should have escaped either, 3.3.13 gives a URI no escaping at all, and there is no list and no structure for them to separate, so truncating the value at one would apply strictness to the wrong end of Postel's law.

Only the kinds the specification structures with `;` (`GEO`, `REQUEST-STATUS`, and the rule parts of `RECUR`) read component by component, and each of their components keeps the commas inside it.

#### Scenario: An unescaped comma in a summary
- GIVEN `SUMMARY:Standup, moved`
- WHEN it is decoded
- THEN the summary reads `Standup, moved`

#### Scenario: An unescaped semicolon in a description
- GIVEN `DESCRIPTION:a;b`
- WHEN it is decoded
- THEN the description reads `a;b` rather than stopping at the semicolon

#### Scenario: A comma inside a structured component
- GIVEN `REQUEST-STATUS:2.0;ok;rcpt,two`
- WHEN it is decoded
- THEN the extra data reads `rcpt,two` rather than stopping at the comma

### Requirement: Round-trip fidelity

The parser SHALL preserve every byte of a parsed calendar, its folds, its blank lines and its QUOTED-PRINTABLE soft breaks included, so serializing an unedited calendar reproduces the input exactly, and editing one property leaves every other byte intact.

Everything survives byte for byte: line endings, parameter order, casing, whitespace inside a value, unknown vocabulary, and the wire layout itself.

A file holding several calendars round-trips through the multi-calendar entry point, whose output concatenates to the input. The single-calendar entry point reads the first calendar and stops, so anything after it is not part of what it returns.

An edit through a cursor that names a component rewrites that component alone. An edit through one that names none replaces the whole value, being the inverse of the reader that names none.

#### Scenario: An unedited calendar
- GIVEN any calendar, however it is laid out
- WHEN it is parsed and serialized
- THEN the output is byte-identical to the input

#### Scenario: A folded real-world calendar
- GIVEN a calendar folded at 75 octets, with blank lines between components
- WHEN it is parsed and serialized
- THEN the output is byte-identical to the input

#### Scenario: One edited property
- GIVEN a parsed, folded calendar
- WHEN one property value is set through its lens cursor
- THEN every line but the edited one keeps its original wire shape

## REMOVED Requirements
