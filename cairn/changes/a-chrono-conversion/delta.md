---
cairn: delta
change: a-chrono-conversion
---

## ADDED Requirements

### Requirement: A civil time converts to and from chrono

Behind an off-by-default `chrono` feature, `IcalRecurDateTime` SHALL convert to and from `chrono::NaiveDateTime`. The outward direction SHALL be fallible, refusing a second this crate admits and chrono has no second field for (second 60, which chrono spells as a nanosecond overflow) rather than mapping it.

No conversion to `chrono::DateTime<Tz>` SHALL be offered, for the reason its jiff sibling gives: a zoned conversion must choose a disambiguation policy, and the crossing to an instant stays `IcalTzOffset::instant`.

#### Scenario: The naive type is the target
- GIVEN a caller holding an occurrence and wanting a chrono value
- WHEN it converts
- THEN it gets a `NaiveDateTime`, and reaches an instant only through a zone and `IcalTzOffset::instant`
