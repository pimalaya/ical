---
cairn: log
change: merge-sibling-ordinal
landed: 2026-08-29
---

# A component's position was counted two different ways, and the second one was wrong

"A component carrying no `UID` SHALL be matched by its position among its same-named siblings" is implemented in four places in the merge. `walk` builds the paths, `find_mut` resolves one in the merged calendar, the `ComponentRemoved` retain drops one, and `find` reads the right side's source; the first three counted per name and `find` counted over every child. A path naming `DAYLIGHT` ordinal 0 was therefore offered `key(child, 1)`, a `STANDARD` having come first, matched nothing, and `apply_to_line` returned early. The change was neither applied nor reported: `judge` had already passed it, having found no collision, so `conflicts` stayed empty and a caller saw a clean merge that had dropped an edit.

The reach was not narrow. A `VTIMEZONE` defining both observances is in nearly every calendar carrying a zone, and its `DAYLIGHT` could not be merged at all. Any second child of a different name went the same way. A second child of the same name was safe, the global index and the per-name index coinciding there, which is exactly why the suite missed it: its only multi-child fixture adds one `VALARM` to an event that had none, and with zero or one sibling positional matching cannot go wrong.

`find` now counts per name, which is four lines and no design decision: the other three call sites were already right and the spec says which reading is meant. The spec's "Instance identity" requirement now says so out loud, that a position is counted the same way wherever the merge counts it, and carries a scenario for a change to a component that is not the first child. tests/merge.rs pins it, asserting both that the `DAYLIGHT` moved and that the `STANDARD` did not, since asserting only the first would pass a merge that wrote the change onto the wrong observance.

Found by a property test over generated calendars, asserting that a side which changed nothing yields the other side unchanged. The wider problem it sits next to is untouched and deliberately so: a replayed action carries the position its target held in the base, and resolving that against a merged calendar whose left side inserted or removed a same-named sibling names the wrong thing. That is a question about what an action should carry rather than a counting error, and it wants the owner.

Capabilities moved: merge, requirement "Instance identity".
