---
cairn: change
change: rfc5545-contract-fixes
---

# Delta

## ADDED Requirements

### Requirement: The contracts follow RFC 5545
A property's allowed values and parameters SHALL be those RFC 5545 states: `DTSTART`, `DTEND`, `DUE` and `RECURRENCE-ID` SHALL take `DATE` and `DATE-TIME` and a `TZID`; `EXDATE` and `RDATE` SHALL take a `TZID`; `ORGANIZER` SHALL take `CN`, `DIR`, `SENT-BY`, `LANGUAGE` and the RFC 6638 and 7986 parameters `ATTENDEE` takes.

### Requirement: An encoded line is folded
An encoded content line longer than 75 octets SHALL be folded (RFC 5545 3.1), never inside a UTF-8 sequence. A decoded line SHALL keep its recorded layout.

### Requirement: A calendar address is a URI
A `CAL-ADDRESS` value SHALL be encoded as a URI, without text escaping.

### Requirement: A duration follows its grammar
A duration SHALL read as a length only when it follows RFC 5545 3.3.6; `P1H` SHALL read as none.
