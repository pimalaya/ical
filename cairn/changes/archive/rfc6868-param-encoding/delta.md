---
cairn: delta
change: rfc6868-param-encoding
---

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: A value is compared as written

Two values SHALL be compared on their raw nodes, component by component, rather than on what they decode to. A decoded value reads its own kind's shape, and a text value reads its first `;`-component alone, so two lines saying different things past that point decode alike and the difference is never seen.

Two parameters SHALL be compared the same way, on their raw nodes and value by value, for the same reason: a single-valued parameter decodes its first value alone, so two parameters differing past their first `,` decode alike and the edit is never reported. Where the two nodes carry different escaping modes they share no decoding to compare through, and only identical bytes are then certainly the same parameter.

#### Scenario: An edit past the first semicolon
- GIVEN a base holding `LOCATION:Room A;floor 2` and a side that changed it to `Room A;floor 9`
- WHEN they are merged
- THEN the change is reported and lands

#### Scenario: An edit past the first comma of a parameter
- GIVEN a base holding `ATTENDEE;CN=Ada,Lovelace` and a side that changed it to `CN=Ada,Byron`
- WHEN they are merged
- THEN the change is reported and lands
