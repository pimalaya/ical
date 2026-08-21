---
cairn: change
id: module-paths
status: landed
created: 2026-08-21
---

# Put the lens, spec, node and cursor types on their real module paths

## Why

Four modules under tree/ declared their contents privately and re-exported them into the parent with a doc-inlined glob: tree/param (lens, node), tree/prop (cardinality, lens, spec), tree/value (cursor, node) and tree/component (lens, spec). The type then had two names, the real one nobody could write and the flattened one everybody did, and the module it lived in was invisible on docs.rs. Guideline naming-005 asks the opposite: types live next to the code that owns them, never behind a private module plus a doc-inlined re-export. vcard-rs, this crate's twin, resolved the same thing in its 0.2.0 and now reads `tree::prop::lens::VcardPropLens`; ical-rs still read `tree::prop::IcalPropLens`.

recur.rs carried a smaller version of the same: `pub use self::validate::{IcalRecurPart, IcalRecurRuleProblem}` gave two types a second path out of an already-public module.

## What

The six submodules become `pub mod`, the re-exports go, and every path in the crate, the tests, the examples and the benches names the module that owns the type. The public API changes shape without changing behaviour: `tree::prop::IcalPropLens` is now `tree::prop::lens::IcalPropLens`, and likewise for `IcalPropSpec`, `IcalPropCardinality`, `IcalParamLens`, `IcalParamNode`, `IcalValueCursor`, `IcalValueNode`, `IcalComponentLens`, `IcalComponentSpec`, `IcalRecurPart` and `IcalRecurRuleProblem`.

The fifteen value-codec modules under tree/value stay private: they hold nothing but `Codec` impls, so there is no type for a caller to reach.
