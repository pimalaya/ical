---
cairn: delta
change: missing-standards
---

## MODIFIED Requirements
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
