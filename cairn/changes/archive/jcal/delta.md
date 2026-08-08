---
cairn: delta
change: jcal
---

## ADDED Requirements
### Requirement: jCal codec

Behind the opt-in `jcal` feature, a decoded calendar SHALL encode to the RFC 7265 JSON form and decode back from it, crossing the boundary as a raw JSON value rather than through serde implementations on any calendar type. A round-trip through jCal SHALL preserve the model, unknown components, properties, parameters and value types included.

#### Scenario: An unknown property
- GIVEN a calendar carrying a property no version defines
- WHEN it is encoded to jCal and decoded back
- THEN the property keeps its name, parameters and value
