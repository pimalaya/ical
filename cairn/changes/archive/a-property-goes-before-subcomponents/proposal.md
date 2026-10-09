---
cairn: change
id: a-property-goes-before-subcomponents
status: landed
created: 2026-10-09
---

# A property goes before the subcomponents

## Why

`IcalCst::push` appended a property after the component's last item, so on a `VEVENT` holding a `VALARM` it landed after `END:VALARM`, where RFC 5545 3.6 does not allow a property. `push_raw` did the same, and so did the merge when it added a property the right side had. The Android app popped each pushed item back off and re-inserted it (`insert` and `place` in rust/src/calendar.rs).

## What

A property the crate adds goes after the component's properties and before its first subcomponent: a new `IcalCst::push_line` primitive does it, `push` encodes through it, `push_raw` places its property lines there and appends a stashed subcomponent's `BEGIN`..`END` lines after everything else, and the merge adds through it. A stashed subcomponent kept as raw lines counts as a subcomponent from its `BEGIN`. Nothing already in the component moves.
