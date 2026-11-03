---
cairn: tasks
change: the-structure-scanned-once
---

The handoff stays active across passes; each pass lands under its own change id and checks its own tasks there (the first: [the-lines-built-in-place](../archive/the-lines-built-in-place/tasks.md)). The second, [the-parts-written-in-place](../the-parts-written-in-place/tasks.md), is under way: read its proposal and tasks first. What is left for it:

- [x] Ask the user about lever 1, the ASCII proof (the crate's first `unsafe`): yes on 2026-11-03, in the working tree; Miri still owed
- [ ] Baseline copy, harness and differential crate refreshed from the tree after the-lines-built-in-place, a planted break caught
- [ ] Lever 2: fewer moves between the tokeniser and the line, a structural change measured on the card first
- [ ] Lever 3: decode into the property's slot
- [ ] Lever 4: the cold walks of `to_bytes` and `decode` over trees parsed long before
- [ ] Levers 5 to 7: the value node's walks, `to_string` of a parsed tree, the small writes
- [ ] Lever 8: the structural scanner's kernel bench against memchr; the scanner only if it wins
- [ ] Bench rows: the corpus through `parse_many` (rivals' coverage checked first), a folded `ATTACH` calendar, a 1.0 quoted-printable calendar
- [ ] Each lever kept only for its cycles (interleaved medians and minimums), the dropped ones listed with their numbers
- [ ] The checks of "Before landing a pass"
- [ ] benches/README.md numbers, CHANGELOG, a log under the pass's own change id, this proposal rewritten, commit after the user's yes
