---
cairn: log
change: a-removed-component-comes-back-for-an-edit
date: 2026-09-02
---

# A component removed under an edit comes back for it

The merge has always said that an update meeting a removal wins, whichever side it came from, because keeping data beats losing it silently, and the spec has always said the thing that survives is the updating side's whole line or component. Only the line was ever built.

So an override deleted on one side and edited on the other settled on whichever side happened to be `ours`. Left deletes, right edits: one divergence reported, the occurrence gone, the edit lost. Left edits, right deletes: one divergence, the occurrence kept with the left's edit, the deletion refused. Two engines merging the same pair in opposite directions converged on nothing, and a calendar object lost an occurrence somebody had just written to.

The judge was right all along. It marks the right side's edit applicable, because a component removal met by anything finer is the coarser act. The replay was where it went: `apply_to_line` looked the component up in the merged calendar, found `None` because the baseline side had removed it, and returned. The verdict said the update landed and nothing landed, in silence.

The replay now holds one `Restored` for both granularities. Before an action is applied, the component it lands in is looked up; where it is gone and the action writes something, the highest component the baseline side removed comes back from the side still working inside it, whole and as that side wrote it. Every further action addressed into that subtree is then let go, because the restored bytes already carry it, which is the same reasoning that made a restored line come back once rather than once per edit. An action that only takes something away brings nothing back: a component removed on one side and untouched on the other still goes, one both sides removed stays gone, and every removal-meets-removal pair still answers the same in both directions.

The test that pinned the old outcome is gone. `an_edited_override_outlives_a_deletion_from_either_side` asserts the rule instead: two components either way round, the edit in both, one divergence in both. Four neighbours came with it, for the deletion nobody edited, the deletion both sides made, a master deleted beside an edited override, which is a recurrence conflict and not a divergence, and an edit one component below the deletion.

One law in the property suite had to learn something. `contested` read only the right half of a conflict, which was enough while the right side's action was the only one that could fail to land. The removal is now the half that can lose, so it reads both halves, and the shape that found this, a `VTIMEZONE` removed on the left with an edit inside it on the right, is in the regression seeds. The field-level reference merge never spoke about this shape: `comparable` already excludes a component one side removed while the other changed something inside it.

Capabilities moved: merge.
