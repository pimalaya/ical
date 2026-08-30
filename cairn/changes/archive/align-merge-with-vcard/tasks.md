---
cairn: tasks
change: align-merge-with-vcard
---

- [x] Compare values on the raw nodes, component by component, rather than decoded
- [x] Diff list items as a multiset, and remove only the first equal item on replay
- [x] Correct a replay target for the baseline side's additions as well as its removals
- [x] Restore a property the baseline side removed once, however many actions the other side made on it
- [x] Collide a retyped `VALUE`, and a whole-value change, with the other side's item edits
- [x] Remove `right_speaks_for`, the `Authority` reason and the organiser predicates
- [x] Fold the spec and log the change
