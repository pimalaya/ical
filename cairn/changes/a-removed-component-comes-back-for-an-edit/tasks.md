---
cairn: tasks
change: a-removed-component-comes-back-for-an-edit
---

- [x] Give `replay` a `Restored` holding both the lines and the components already put back.
- [x] Restore the highest component the baseline side removed out from under an act that writes something, from the side that wrote it.
- [x] Let go of every action a restored component already carries, at any depth.
- [x] Rewrite `a_deleted_override_and_an_edited_one_settle_on_the_left_side` as `an_edited_override_outlives_a_deletion_from_either_side`.
- [x] Cover the neighbours: a deletion nobody edited, a deletion both sides made, a master deleted beside an edited override, and an edit nested one component below the deletion.
- [x] Read both halves of a conflict in the property suite's `contested`, since the removal is now the half that can lose.
- [x] Fold the delta into `cairn/spec/merge.md` and write the log entry.
- [x] Note the fix in `CHANGELOG.md`.
