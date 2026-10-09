---
cairn: delta
change: a-property-goes-before-subcomponents
---

## ADDED Requirements

### Requirement: A property goes before the subcomponents
Into parsing. `push`, `push_line`, the property lines of `push_raw` and the merge's additions go after a component's properties and before its first subcomponent, a stashed one included; `push_raw` appends a stashed subcomponent's lines; nothing parsed moves.

## MODIFIED Requirements

## REMOVED Requirements
