---
cairn: delta
change: jscalendar
---

## ADDED Requirements
### Requirement: JSCalendar conversion

Behind the opt-in `jscalendar` feature, a decoded calendar SHALL convert to and from the RFC 8984 JSON data model. Everything the RFC maps SHALL survive both directions unchanged. Anything the mapping cannot express SHALL be carried in an escape hatch, in jCal syntax, rather than dropped.

#### Scenario: A property outside the mapping
- GIVEN an event carrying a property RFC 8984 does not map
- WHEN it is converted to JSCalendar and back
- THEN the property returns from the escape hatch intact

## MODIFIED Requirements

### Requirement: A declared VALUE decides the kind, known name or not

A property that declares its own `VALUE` SHALL decode as that kind whether or not its name is in the vocabulary (RFC 5545 3.2.20).

#### Scenario: A vendor property that names its type
- GIVEN `X-OFFSET;VALUE=UTC-OFFSET:-0500`
- WHEN it is decoded
- THEN the value is a UTC offset rather than an undecoded one

### Requirement: A decoded calendar can outlive its bytes

Every decoded type SHALL offer `into_owned`, replacing each borrow with an allocation.

#### Scenario: A calendar outliving its buffer
- GIVEN a calendar decoded from a buffer
- WHEN `into_owned` is called on it
- THEN the result borrows nothing from the buffer
