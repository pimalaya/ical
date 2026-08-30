---
cairn: log
change: a-spec-is-not-syntax
date: 2026-08-30
---

# A spec is not syntax

Every property and component marker carried two things at once: an RFC contract and a syntax projection. Both sat under `tree`, so both were gated on `parser`, and the gate reached much further than the parser did. `Ical::validate` reads the decoded model. `IcalPropBuilder` emits a decoded property. The jCal codec wanted the spec vtable to resolve one value kind. None of the three parses a byte, and all three were unreachable without `memchr`.

The split follows the distinction the code already made. Seventy property markers and their `IcalPropSpec` impls moved to `prop::<name>`, fourteen component markers and their `IcalComponentSpec` impls to `component::<name>`, each beside the vtable that dispatches the open kind onto it. `IcalPropCardinality` went with the property spec and `COMMON_PARAMS` with the parameters. What stayed under `tree::prop::<name>` is the `IcalPropLens` impl alone, which is the only half that knows about bytes.

`IcalComponentLens` did not move, it went. It was an empty marker trait whose whole content was its `IcalComponentSpec` supertrait; `IcalCst::component` and its siblings now bound on the spec directly, and fourteen files holding one empty impl each went with it.

The strict-out layer followed its contract out of `tree`. `tree::ical::builder` is `builder` and `tree::ical::validate` is `validator`, both at the crate root, and `tree::ical` is gone. `valid` folded into `validator`, so the proof sits with the check that mints it, as it does in vcard-rs; `IcalRecurRule::validate` still mints the same marker from the other end of the crate. `From<IcalValid<Ical>> for IcalCst` moved to the encoder, which is where a conversion to a tree belongs and what leaves the validator with no dependency on `tree` at all.

`jcal` stopped implying `parser`. `--no-default-features --features jscalendar` now builds, and pulls in `serde_json` alone: a JMAP client that never sees an iCalendar byte can depend on this crate for the conversion.

Three test modules compiled for the first time as a result, and the marker paths moved for every caller: `ical::tree::prop::summary::SUMMARY` is `ical::prop::summary::SUMMARY`, and the component markers likewise.

Capabilities moved: conformance, jcal, jscalendar.
