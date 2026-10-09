---
cairn: change
id: a-conflict-names-its-sides
status: landed
created: 2026-08-31
---

# A conflict names one side and describes the other

## Why

vcard-rs and ical-rs state one merge contract, and a caller reading both should not have to change idiom halfway down the stack. `VcardMergeConflict` is `{ left, right }`: two actions, one per side, named after the sides themselves. `IcalMergeConflict` was `{ right, reason }`, which names one side and then describes the other from the outside.

The two carry the same information. ical-rs holds one extra conflict kind, `Recurrence`, so the left action arrives inside an enum saying which kind this is, and that enum earns its place. The field name did not: `reason` says why the right side lost without saying whose action it is holding, and the variant documentation had to make up the difference, ending on "beside the right side's on the conflict itself" to tell a reader where the other half was.

A consumer feels it directly. tCal read `sides.read(&conflict.right, &conflict.reason)`, right before left, because the field order said so.

## What

- Rename `IcalMergeConflict::reason` to `left`, and put it first, so the pair reads `{ left, right }` as vcard-rs's does.
- Keep `IcalMergeReason` as the field's type: the extra kind is real, and the enum is where it is said.
- Drop the sentence in `Divergent` that pointed at the other half of the pair, the field names now saying it.
