---
cairn: delta
change: rfc5545-contract-fixes
---

# Delta

## ADDED Requirements

### Requirement: The contracts follow RFC 5545
Into conformance. A property's contract SHALL allow every value type and every parameter its RFC gives it: `DTSTART`, `DTEND`, `DUE` and `RECURRENCE-ID` SHALL take `DATE` beside `DATE-TIME` and a `TZID`, and `RECURRENCE-ID` its `RANGE`; `EXDATE` and `RDATE` SHALL take a `TZID`; `ORGANIZER` SHALL take `CN`, `DIR`, `SENT-BY`, `LANGUAGE`, `EMAIL` and the RFC 6638 parameters `ATTENDEE` takes. The section 3.8 audit adds `ATTACH` (`BINARY`, `FMTTYPE`, `ENCODING`), `FREEBUSY` (`FBTYPE`), `TRIGGER` (`DATE-TIME`, `RELATED`), a repeatable `DESCRIPTION`, and from the extensions `IMAGE`, `CONFERENCE`, `LINK`, `RELATED-TO`, `STRUCTURED-DATA` and `STYLED-DESCRIPTION`. A contract may stay wider than its RFC (the default parameter set, one cardinality for every component).

### Requirement: An encoded line is folded
Into parsing. A content line with no recorded shape that still fits it SHALL be folded when longer than 75 octets (RFC 5545 3.1), never inside a UTF-8 sequence, with the line's own break. A parsed line SHALL keep its recorded layout byte for byte. A `QUOTED-PRINTABLE` line SHALL NOT be folded.

### Requirement: A calendar address is a URI
Into decoded-model. A `CAL-ADDRESS` value SHALL be encoded as a URI, without text escaping, on every path: the model's encode and the value cursor's `set_text` and `set_bytes`, which encode by the line's value type. Reading stays liberal.

### Requirement: Durations are validated
Into conformance. A duration value (`DURATION`, a relative `TRIGGER`, `REFRESH-INTERVAL`) or `GAP` parameter outside the RFC 5545 3.3.6 grammar SHALL be reported, with the property and the text as written. A period's duration is not checked.

## MODIFIED Requirements

### Requirement: The validation walk
In conformance. The walk also reports a duration outside its grammar.

### Requirement: Line normalisation
In parsing. An edited value still drops the recorded shape of its own line, and the line is then laid out as an encoded one is, folded at 75 octets, rather than written unfolded.

### Requirement: A written value never breaks its line
In parsing. A line break written into a URI or a calendar user address SHALL go out percent-encoded (RFC 3986 2.1), these values taking no text escape.

### Requirement: A duration and a UTC offset read as numbers
In decoded-model. `IcalDuration::seconds` SHALL read liberally what calendars in the wild write (`P1H`, lower case, `PT1H20S`, units in any order, a fraction of second, RFC 8984's `P1W2D`), returning nothing only for text naming no length; whether a duration conforms is the validator's question. `IcalDuration::from_seconds` writes the strict grammar, a minute between an hour and a second.

### Requirement: A truncating read names the component it truncates at
In parsing. The cursor's whole-value setters `set_text` and `set_bytes` SHALL encode by the line's value type.

## REMOVED Requirements
