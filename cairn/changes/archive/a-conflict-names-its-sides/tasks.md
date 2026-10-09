---
cairn: tasks
change: a-conflict-names-its-sides
---

- [x] Rename `IcalMergeConflict::reason` to `left` and order the fields `left`, `right`.
- [x] Rewrite the struct and variant documentation around the two field names.
- [x] Update the construction site in `IcalMerge::merge`.
- [x] Update `tests/merge.rs`, `tests/merge_props.rs` and `examples/three_way_merge.rs`.
- [x] Fold the delta into `cairn/spec/merge.md` and write the log entry.
- [x] Note the break in the changelog.
