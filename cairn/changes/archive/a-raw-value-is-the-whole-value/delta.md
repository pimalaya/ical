---
cairn: delta
change: a-raw-value-is-the-whole-value
---

## ADDED Requirements

### Requirement: A raw value is the whole value
Into parsing. `IcalLine::raw_value` and `raw_value_str` return the whole value as written on the wire, still escaped, every `;` and `,` in place.

## MODIFIED Requirements

## REMOVED Requirements
