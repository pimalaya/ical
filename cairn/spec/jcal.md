---
cairn: spec
capability: jcal
status: current
---

# jCal

The RFC 7265 JSON spelling of a calendar, behind the opt-in `jcal` feature. A component is `[name, [properties], [components]]`, a property is `[name, {params}, type, value...]`. The boundary is a raw `serde_json::Value`, never a serde implementation on a calendar type: one type has more than one JSON spelling, and serde keys one representation per type.

### Requirement: jCal codec

A decoded calendar SHALL encode to the RFC 7265 form and decode back from it. A round-trip SHALL preserve the model: unknown components, properties, parameters and value types included.

The type slot carries what `VALUE` declared, and import resolves it back through the same property spec the wire decoder uses, so a declared kind that says more than the property's default returns as a `VALUE` parameter rather than dissolving into the type it named.

#### Scenario: An unknown property
- GIVEN a calendar carrying a property no version defines, with an unknown parameter and an unrecognised type slot
- WHEN it is encoded to jCal and decoded back
- THEN the property keeps its name, its parameter and its value

#### Scenario: A declared value kind
- GIVEN `RDATE;VALUE=PERIOD:20260106T100000/20260106T120000`
- WHEN it is encoded to jCal and decoded back
- THEN the type slot reads `period` and the decoded property carries `VALUE=PERIOD` again

### Requirement: What jCal normalises

A round-trip SHALL be a fixpoint rather than byte-preserving. Three things the JSON format cannot hold are normalised, and no more: parameter order (a JSON object is unordered), recurrence rule part order (likewise, so parts come back in the order RFC 5545 3.3.10 states them), and the case of an unknown name (jCal lowercases every name, and iCalendar's convention is uppercase).

Byte fidelity belongs to the syntax tree. jCal is a projection of the decoded model.

#### Scenario: A rule written out of order
- GIVEN `RRULE:FREQ=WEEKLY;BYDAY=MO;COUNT=10`
- WHEN it is encoded to jCal and decoded back
- THEN the rule reads `FREQ=WEEKLY;COUNT=10;BYDAY=MO`, the same parts in the RFC's order

### Requirement: jCal needs no parser

The `jcal` feature SHALL NOT imply `parser`. It reads the property spec to resolve a value kind, and the spec is model rather than syntax, so a build with default features off and `jcal` on SHALL pull in `serde_json` alone.
