---
cairn: tasks
change: the-structure-scanned-once
---

The handoff stays active across passes; each pass lands under its own change id and checks its own tasks there (the first: [the-lines-built-in-place](../archive/the-lines-built-in-place/tasks.md), the second: [the-parts-written-in-place](../archive/the-parts-written-in-place/tasks.md)). What is left for the next one:

- [ ] Baseline copy, harness and differential crate refreshed from harness/ against the tree after the-parts-written-in-place, a planted break caught
- [ ] Lever 1: one scan a line (the line feed and the head's marks together), then the structural scanner, each measured on the card first
- [ ] Lever 2: a smaller item (API break: ask the user first)
- [ ] Lever 3: the component box written field by field (`unsafe`: ask the user first)
- [ ] Levers 4 to 8: the value kind from a table, the kind lookups confirmed by word compares, list values by block scans, the unknown value's walk, the small writes
- [ ] Bench rows: the corpus through `parse_many` (rivals' coverage checked first); a criterion group for the content decoders with the detected token
- [ ] Each lever kept only for its cycles over four layouts, the dropped ones listed with their numbers
- [ ] The checks of "Before landing a pass"
- [ ] benches/README.md numbers, CHANGELOG, a log under the pass's own change id, this proposal rewritten, commit after the user's yes
