---
cairn: tasks
change: a-spec-is-not-syntax
---

- [x] Move each property marker and its `IcalPropSpec` impl to `prop::<name>`, leaving the lens impl in `tree::prop::<name>`
- [x] Move `IcalPropCardinality` and the property vtable to `prop`, and `COMMON_PARAMS` to `param`
- [x] Move each component marker and its `IcalComponentSpec` impl to `component::<name>`, with the vtable
- [x] Delete `IcalComponentLens` and bound the typed subtree accessors on `IcalComponentSpec`
- [x] Move `tree::ical::builder` to `builder` and `tree::ical::validate` to `validator`, folding `valid` in
- [x] Move `From<IcalValid<Ical>> for IcalCst` to the encoder, so the validator has no tree dependency
- [x] Drop `parser` from the `jcal` feature and verify the bare-core builds
- [x] Fold the spec and log the change
