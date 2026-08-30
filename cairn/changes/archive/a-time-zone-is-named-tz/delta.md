---
cairn: delta
change: a-time-zone-is-named-tz
---

## MODIFIED Requirements

### Requirement: The time-zone layer is scoped by tz

The module SHALL be `tz` and its public types SHALL be `IcalTz`, `IcalTzObservance` and `IcalTzOffset`, matching the `TZ` prefix the RFC gives every property the layer reads.

`IcalTzOffset` SHALL be documented as the answer to a resolution (one offset, a gap or a fold) rather than as an offset value, which is `IcalUtcOffset`.
