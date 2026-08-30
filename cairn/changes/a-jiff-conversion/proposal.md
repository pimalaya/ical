---
cairn: change
id: a-jiff-conversion
status: active
created: 2026-08-30
---

# A jiff conversion

## Why

A caller holding an occurrence holds an `IcalRecurDateTime`, a civil date and time this crate defines because it will not take a dependency to name one. Everyone else in the ecosystem already has a type for that, and converting by hand is six field reads nobody should write twice.

jiff is the closer of the two candidates, and not by a small margin: its model is ours. `TimeZone::to_zoned` yields a result the caller must disambiguate through an explicit policy rather than one the library picks, which is `IcalTzOffset::One` / `Gap` / `Fold` under other names. Converting between two libraries that agree a local time may name zero, one or two instants is honest in a way converting to a type that must name exactly one is not.

## What

An off-by-default `jiff` feature, four impls and nothing else:

- `TryFrom<IcalRecurDateTime> for jiff::civil::DateTime`
- `From<jiff::civil::DateTime> for IcalRecurDateTime`

and the same pair over `IcalDateTime`, the wire value, which first needs `IcalDateTime::civil` to read its raw text into a civil time. That bridge is the entry point for every conversion here and lands with this change.

Outward is fallible on purpose. This crate admits second 60, the wire spelling it and RFC 5545 3.3.5 permitting it, and jiff does not; jiff caps a year at four digits where the field is an `i32`. Both are refusals rather than mappings, so nothing is silently rewritten.

**The feature deliberately offers no zoned conversion.** There is no `IcalRecurDateTime -> jiff::Zoned`, because such an impl would have to pick a disambiguation policy, and that choice is the one this crate exists to hand back. A caller reaching for an instant goes through `IcalTzOffset::instant`, where the gap is the RFC's `None` rather than an argument to a function. The API shape carries the rule so the documentation does not have to.

## Consequence

The first optional dependency in the model layer rather than behind `parser`, so whether the bare core stays `no_std` is a property to verify rather than assume.

jiff is pre-1.0. A breaking release forces a release here for a feature few will enable, which is the cost of the convenience and is bounded by there being four impls to fix.
