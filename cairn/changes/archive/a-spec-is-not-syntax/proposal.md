---
cairn: change
id: a-spec-is-not-syntax
status: landed
created: 2026-08-30
---

# A spec is not syntax

## Why

Every property and component the crate knows is a zero-sized marker carrying two things at once: an RFC contract (the versions it lives in, its cardinality, the value types and parameters it may take, the children it may nest) and a syntax projection (how to decode and edit one line or subtree of a byte tree).

Both lived under `tree`, so both were gated on the `parser` feature. That put the whole strict-out layer behind a parser it never touches: `Ical::validate` reads the decoded model, `IcalPropBuilder` emits a decoded property, and the jCal codec only ever wanted the spec vtable to resolve a value kind. None of the four parses anything, and all four were unreachable without `memchr`.

The gate was not the only cost. `tree::ical::builder` and `tree::ical::validate` read as a tree builder and a tree validator, which is not what they are, and the `ical` module inside `tree` named a layer rather than a thing.

## What

Split each marker in two. The marker itself, and its `IcalPropSpec` or `IcalComponentSpec` impl, move to the model side, one module per property under `prop` and one per component under `component`, beside the vtable that dispatches the open kind onto them. Only the `IcalPropLens` impl stays under `tree`, in a module of the same name.

The strict-out layer follows the contract it consults. `tree::ical::builder` becomes `builder` and `tree::ical::validate` becomes `validator`, both at the crate root, and `tree::ical` disappears. `valid` folds into `validator`, so the proof sits with the check that mints it, as it does in vcard-rs.

`IcalComponentLens` is deleted rather than moved. It was an empty marker trait whose whole content was its `IcalComponentSpec` supertrait; the typed subtree accessors bound on the spec directly, and fourteen files holding one empty impl each go with it.

`jcal` stops implying `parser`. Done when `--no-default-features --features jscalendar` builds.

## Consequence

The decoded model, the builder, the validator, the recurrence layer, the time zones and both JSON representations are all reachable without the parser. A JMAP client that never sees an iCalendar byte can depend on this crate and pull no dependency at all.
