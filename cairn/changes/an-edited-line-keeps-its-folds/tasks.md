---
cairn: tasks
change: an-edited-line-keeps-its-folds
---

- [ ] src/tree/wire.rs: cut a shape at the first changed offset instead of dropping it; refold the tail with the line's own whitespace and ending
- [ ] The value writers report, or the line computes, the first changed offset
- [ ] Tests: the delta's scenario; an edit in the first physical line; a shrinking edit; a two-octet character at the cut; a `QUOTED-PRINTABLE` line untouched; the merge fuzz target's fixpoint oracle still holds
- [ ] Same rule and tests as vcard cairn/changes/an-edited-line-is-folded, cross-checked
- [ ] Fold parsing.md, log, CHANGELOG
