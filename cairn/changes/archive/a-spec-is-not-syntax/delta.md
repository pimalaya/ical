---
cairn: delta
change: a-spec-is-not-syntax
---

## MODIFIED Requirements

### Requirement: The per-property and per-component contract is model, not syntax

Each property SHALL define a marker in `prop::<name>` carrying its `IcalPropSpec`, and each component a marker in `component::<name>` carrying its `IcalComponentSpec`. The vtable dispatching the open `IcalPropKind` and `IcalComponentKind` onto those static impls SHALL live beside them. Neither SHALL require the `parser` feature.

The read-and-edit lens on a property marker SHALL stay under `tree::prop::<name>`, so the contract and the projection meet on one type without the contract depending on a parser.

### Requirement: Conformance checking and strict construction need no parser

`Ical::validate` SHALL live in `validator` and `IcalPropBuilder` in `builder`, both at the crate root and both available with default features off. `IcalValid` SHALL live beside the calendar validator that mints it.

### Requirement: A JSON representation needs no parser

The `jcal` feature SHALL NOT imply `parser`. Building with default features off and `jscalendar` on SHALL pull no dependency beyond `serde_json`.

## REMOVED Requirements

### Requirement: A component is identified by a lens trait

Removed. `IcalComponentLens` carried nothing its `IcalComponentSpec` supertrait did not already supply. Typed subtree access is keyed on the component marker through the spec directly.
