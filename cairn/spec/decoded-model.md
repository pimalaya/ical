---
cairn: spec
capability: decoded-model
status: current
---

# Decoded model

The version-agnostic side of the crate: pure data with no dependency on the syntax side, always available even with `parser` off. A calendar is an `Ical` (a version, the calendar-level properties, and nested `IcalComponent`s, themselves recursive). A property is an `IcalProp` of a name, parameters and one value.

### Requirement: One model, every version

The crate SHALL read and write vCalendar 1.0 (versit) and iCalendar 2.0 (RFC 5545, extended by 6638, 7529, 7953, 7986, 9073, 9074 and 9253) through a single model. The version SHALL be a decoded indicator, never a type parameter and never a separate dialect. Only the codec and the per-property spec branch on it, and only where escaping or a value's shape genuinely differ.

#### Scenario: An unrecognised version
- GIVEN a calendar whose `VERSION` is missing or unrecognised
- WHEN it is decoded
- THEN the decoded version normalises to 2.0, while the syntax tree keeps the original bytes

#### Scenario: An availability component
- GIVEN a `VAVAILABILITY` carrying an `AVAILABLE` sub-component
- WHEN the calendar is decoded and validated
- THEN both are known components and the nesting is accepted

### Requirement: A list value is a list whatever VALUE declares

A property whose model kind is a list (`RDATE`, `EXDATE`, `CATEGORIES`, `RESOURCES`) SHALL decode as one whatever its `VALUE` parameter declares. The declared kind describes each *item*, not the value as a whole.

#### Scenario: A declared item type on a list
- GIVEN `CATEGORIES;VALUE=TEXT:one,two`
- WHEN it is decoded
- THEN both items are present

### Requirement: Every property belongs to a version

Each property SHALL state the versions that define it, and validation SHALL report one written in a version that does not. The legacy vCalendar 1.0 alarm properties (`AALARM`, `DALARM`, `MALARM`, `PALARM`), along with `RNUM` and `TZ`, belong to 1.0 alone; every property an extension RFC adds belongs to iCalendar 2.0 alone, as do the RFC 5545 properties vCalendar 1.0 never had.

#### Scenario: An extension property in a vCalendar 1.0 file
- GIVEN `COLOR` (RFC 7986) in a calendar whose version is 1.0
- WHEN the calendar is validated
- THEN the property is reported as one the version does not define

#### Scenario: A property both versions share
- GIVEN `SUMMARY` in a calendar of either version
- WHEN the calendar is validated
- THEN nothing is reported about it

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

### Requirement: A parameter value is encoded by RFC 6868, not by the text escapes

A parameter value SHALL be decoded and encoded by RFC 6868 section 3.1: `^n` reads as a newline, `^^` as a caret, `^'` as a double quote, and any other caret sequence, a trailing lone caret included, stays exactly as written, which section 3.1 requires rather than merely permits. A backslash SHALL be content in both directions, since RFC 5545 section 3.2 gives a parameter value no escapes at all and RFC 6868 section 3.2 forbids adding the backslash ones.

RFC 6868 updates RFC 5545 and no earlier specification, so the rules SHALL apply to iCalendar 2.0 alone. A vCalendar 1.0 parameter carries its caret literally, and a parameter node SHALL therefore carry the escaping mode of the calendar it was parsed from, stamped once `VERSION` is known, as a value node already does.

A value the wire spelled inside its own double quotes SHALL keep that pair on the way out, only what they enclose being encoded. The decoded model holds a parameter exactly as it was written, delimiters included, so encoding the surrounding pair would strip the quoting off every quoted URI.

#### Scenario: The three sequences
- GIVEN `CN=a^nb^^c^'d` in a 2.0 calendar
- WHEN it is decoded and encoded again
- THEN it reads `a`, a newline, `b^c"d`, and comes back as `CN=a^nb^^c^'d`

#### Scenario: A caret before an ordinary letter
- GIVEN `CN=a^xb^`
- WHEN it is decoded
- THEN it reads `a^xb^`, the caret and what follows staying as written

#### Scenario: A backslash in a parameter
- GIVEN `X-PATH=C:\temp\note.txt`
- WHEN it is decoded
- THEN the value keeps both separators rather than losing them to a text escape

#### Scenario: A caret in a vCalendar 1.0 parameter
- GIVEN `LABEL=a^nb` in a 1.0 calendar
- WHEN it is decoded
- THEN it reads `a^nb`, the version predating RFC 6868

#### Scenario: A quoted parameter through a round trip
- GIVEN `ALTREP="cid:part1.0001@example.org"`
- WHEN it is decoded and encoded again
- THEN the bytes are the ones it arrived as

### Requirement: A declared VALUE decides the kind, known name or not

A property that declares its own `VALUE` SHALL decode as that kind whether or not its name is in the vocabulary (RFC 5545 3.2.20). A name outside the vocabulary has no spec to consult, but a line that says what it is has said it.

#### Scenario: A vendor property that names its type
- GIVEN `X-OFFSET;VALUE=UTC-OFFSET:-0500`
- WHEN it is decoded
- THEN the value is a UTC offset rather than an undecoded one

### Requirement: A decoded calendar can outlive its bytes

Every decoded type SHALL offer `into_owned`, replacing each borrow with an allocation. A calendar read from a buffer that is about to go away, or rebuilt from data that was never one line to begin with, needs exactly that.

#### Scenario: A calendar outliving its buffer
- GIVEN a calendar decoded from a buffer
- WHEN `into_owned` is called on it
- THEN the result borrows nothing from the buffer

### Requirement: Closed identity, open payload

Component names, property names, parameter names and value types SHALL be closed identity enums whose wire spelling is reached through `FromStr` and `Deref`. Parameters and values SHALL be open payload enums carrying an `Unknown` arm, so anything outside the model survives a decode.

#### Scenario: A vendor property
- GIVEN a property no version defines
- WHEN the calendar is decoded and re-encoded
- THEN the property survives with its name, parameters and value intact

### Requirement: Projection both ways

A syntax tree SHALL project onto the decoded model (`decode`), and the model SHALL project back to a canonical syntax tree (`encode`, `From<Ical>`).

### Requirement: Opt-in content decoding

The core SHALL transform no content. A transfer encoding (`QUOTED-PRINTABLE`, `BASE64`) and a `CHARSET` SHALL be surfaced raw, with their parameters kept. Decoding them is opt-in, one small `no_std` crate per feature: `quoted-printable`, `base64`, `encoding`.

#### Scenario: An undecoded binary attachment
- GIVEN an inline `BASE64` value and the `base64` feature off
- WHEN the calendar is decoded
- THEN the value is the raw base64 text, with its `ENCODING` parameter kept
