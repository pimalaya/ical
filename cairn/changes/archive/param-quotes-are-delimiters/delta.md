---
cairn: delta
change: param-quotes-are-delimiters
---

## MODIFIED Requirements

### Requirement: A parameter value is encoded by RFC 6868, not by the text escapes

A parameter value SHALL be decoded and encoded by RFC 6868 section 3.1: `^n` reads as a newline, `^^` as a caret, `^'` as a double quote, and any other caret sequence, a trailing lone caret included, stays exactly as written, which section 3.1 requires rather than merely permits. A backslash SHALL be content in both directions, since RFC 5545 section 3.2 gives a parameter value no escapes at all and RFC 6868 section 3.2 forbids adding the backslash ones.

RFC 6868 updates RFC 5545 and no earlier specification, so the rules SHALL apply to iCalendar 2.0 alone. A vCalendar 1.0 parameter carries its caret literally, and a parameter node SHALL therefore carry the escaping mode of the calendar it was parsed from, stamped once `VERSION` is known, as a value node already does.

The double quotes RFC 5545 section 3.1 wraps a `param-value` in SHALL be delimiters rather than content: decoding a parameter SHALL strip a balanced surrounding pair before resolving the carets, and encoding one SHALL wrap the encoded text in a pair when it carries a `,`, a `;` or a `:`, the delimiters a bare `paramtext` may not hold. A double quote cannot reach that test, the caret encoding having already spelled it `^'`.

The `quoted-string` production is iCalendar 2.0's, so `Escaper` SHALL answer for it separately from the caret encoding: a vCalendar 1.0 parameter has no quoting and its double quote is content.

An unbalanced quote SHALL be content, so a value the wire left open decodes as it stands rather than losing a delimiter it never closed.

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
- WHEN it is decoded
- THEN it reads `cid:part1.0001@example.org`, and encoding it again puts the quotes back, the value carrying a `:`

#### Scenario: A quoted value needing no quotes
- GIVEN `PARTSTAT="ACCEPTED"`
- WHEN it is decoded and encoded again
- THEN it reads `ACCEPTED` and comes back as `PARTSTAT=ACCEPTED`, the quotes having nothing to protect

#### Scenario: A double quote inside a 2.0 parameter
- GIVEN a decoded `CN` reading `say "hi", then go`
- WHEN it is encoded
- THEN it comes back as `CN="say ^'hi^', then go"`, the quote encoded and the pair added for the comma

#### Scenario: A quote a vCalendar 1.0 calendar wrote
- GIVEN `X-FOO="bar"` in a 1.0 calendar
- WHEN it is decoded
- THEN it reads `"bar"`, the version having no quoting for the pair to delimit

#### Scenario: An unbalanced quote
- GIVEN `PARTSTAT="ACCEPTED` in a 2.0 calendar
- WHEN it is decoded
- THEN it reads `"ACCEPTED`, the pair being unbalanced

#### Scenario: A parameter one side merely re-quoted
- GIVEN a base carrying `PARTSTAT=NEEDS-ACTION` and a side carrying `PARTSTAT="NEEDS-ACTION"`
- WHEN the two are merged against that base
- THEN no change is reported and no conflict is raised
