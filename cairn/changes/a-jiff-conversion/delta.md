---
cairn: delta
change: a-jiff-conversion
---

## ADDED Requirements

### Requirement: A civil time converts to and from jiff

Behind an off-by-default `jiff` feature, `IcalRecurDateTime` SHALL convert to and from `jiff::civil::DateTime`. The outward direction SHALL be fallible, refusing a second this crate admits and jiff does not (second 60, RFC 5545 3.3.5) and a year outside jiff's range, rather than mapping either onto something else.

No conversion to a zoned jiff type SHALL be offered. Such a conversion would have to choose a disambiguation policy, which is the choice `IcalTzOffset` exists to return to the caller; the crossing to an instant stays `IcalTzOffset::instant`.

#### Scenario: A leap second refused rather than mapped
- GIVEN a civil time whose second is 60
- WHEN it is converted to a jiff civil date-time
- THEN the conversion fails, the value having no counterpart there
