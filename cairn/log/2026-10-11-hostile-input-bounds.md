---
cairn: log
change: hostile-input-bounds
date: 2026-10-11
---

# Hostile input bounds

Found by the Android audit (S3, findings 2, 3 and 9).

`take_component` recursed once per nested `BEGIN`, with frames large enough that about 9 KB of nested `BEGIN` lines overflowed a 1 MiB stack, the stack an Android thread gets: a SIGSEGV no JNI guard catches, on any event a CalDAV server hands over. Every later walk of the tree (decode, clone, drop, merge, serialization) recursed the same way, so parsing now bounds the depth they all see: `IcalCst::DEPTH_MAX` (16). The strict parsers refuse deeper nesting with the new `IcalParseError::TooDeep`; the recovering one keeps what lies past the bound as raw lines of the deepest component it admits, iteratively, so the bytes still round-trip.

A windowed expansion walked every occurrence since `DTSTART`: a `FREQ=SECONDLY` event started in 1970 cost minutes per call. `IcalRecurExpand::skip_to` and `IcalRecurSetExpand::skip_to` jump a rule with no `COUNT` to the period before the window on its `INTERVAL` lattice; a `COUNT` rule still walks, counted and not yielded. The recur fuzz target checks the skip against the full walk.

The merge fuzz target's idempotence oracle failed twice. Both sides retyping a value and changing it identically collided with each other's halves, since a parameter saying what the value means contests every value action of the other side: a side that made the same change of meaning no longer contests it. And a stamp both sides added was settled against the left side's other stamp before agreement was looked at, landing it twice: an act the left side performed identically is no longer settled.

## What landed

- src/tree/cst.rs, src/tree/error.rs: `DEPTH_MAX`, `TooDeep`, the bounded strict parse and the flattening recovering one.
- src/recur/expand.rs, src/recur/set.rs: `skip_to` on both walks.
- src/tree/merge/judge.rs: the two agreement fixes.
- tests/nesting.rs (on 1 MiB threads), tests/recur_skip.rs, tests/merge.rs (three cases), fuzz/fuzz_targets/recur.rs (the skip oracle).
- Capabilities moved: parsing ("Nesting is bounded", added), recurrence ("A walk skips ahead", added), merge (the meaning-parameter and bookkeeping requirements, modified).

## Not done

Merging a component of 30000 lines with itself takes 0.7 s in release (decoding and cloning it 0.02 s): a CPU cost for a hostile calendar of a few hundred kilobytes, not a crash, left for a later look.
