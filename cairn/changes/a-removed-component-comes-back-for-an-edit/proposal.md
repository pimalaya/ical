---
cairn: change
id: a-removed-component-comes-back-for-an-edit
status: landed
created: 2026-09-02
---

# A component removed under an edit comes back for it

## Why

The merge states one rule about a removal meeting an update: the update wins whichever side it came from, because keeping data beats losing it silently. The spec already says the surviving thing is the updating side's whole "line or component". Only the line half was ever built.

An override deleted on one side and edited on the other therefore settled on whichever side happened to be the left one. Left deletes and right edits: one `Divergent(ComponentRemoved)` is reported, the occurrence goes, and the right side's edit is lost. Left edits and right deletes: one conflict, the occurrence stays with the left's edit, and the deletion is refused. Merging A into B and merging B into A disagreed about what survives, which for a synchronisation engine means a calendar that converges on nothing.

The judge was not the problem. It marks the right side's edit applicable, exactly as the rule says, because `Op::scraps` answers true for a component removal met by anything finer. The replay was: `apply_to_line` looks the component up in the merged calendar with `at_mut`, gets `None` because the left side removed it, and returns. The verdict says the update landed; nothing landed. The property-level twin of this case has had an explicit restore path since the merge was written, and the component-level one had none.

## What

- Give the replay a component-level restore, the twin of the line-level one: where the component an action lands in is gone because the baseline side removed it, put it back from the side that is still working inside it.
- Put it back whole and once, as the right side wrote it, and let the actions it already carries go rather than replaying them onto it a second time. That covers an edit nested any number of components deep, since the whole subtree comes back with it.
- Bring a component back only for an act that writes something. An act that only takes something away has nothing to keep, so a component removed on one side and untouched on the other still goes, one removed on both sides stays gone, and the two directions of every removal-meets-removal pair keep agreeing.
- Hold the line-level and component-level bookkeeping in one `Restored`, so there is one answer to "has this already come back".
