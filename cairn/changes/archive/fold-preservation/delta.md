---
cairn: delta
change: fold-preservation
---

## MODIFIED Requirements
### Requirement: Round-trip fidelity

The parser SHALL preserve every byte of a parsed calendar, including its folds, its blank lines and its QUOTED-PRINTABLE soft breaks, so serializing an unedited calendar reproduces the input exactly, and editing one property leaves every other byte intact.

#### Scenario: A folded real-world calendar
- GIVEN a calendar folded at 75 octets, with blank lines between components
- WHEN it is parsed and serialized
- THEN the output is byte-identical to the input

#### Scenario: One edited property
- GIVEN a parsed, folded calendar
- WHEN one property value is set through its lens cursor
- THEN every line but the edited one keeps its original wire shape

### Requirement: Line normalisation

The parser SHALL resolve the wire shape of a line into logical content for every layer above it, and SHALL restore that shape on output: folded continuation lines, blank lines before a content line, and QUOTED-PRINTABLE soft line breaks. A missing final line break stays absent on output.

An edited value SHALL drop the recorded shape of its own line rather than re-apply fold points that no longer match its bytes.

#### Scenario: A folded line
- GIVEN `NOTE:foo\r\n bar\r\n`
- WHEN it is parsed
- THEN the line holds the logical value `foobar` and serializes folded exactly as it arrived
