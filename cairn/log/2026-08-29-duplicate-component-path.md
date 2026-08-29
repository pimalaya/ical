---
cairn: log
change: duplicate-component-path
landed: 2026-08-29
---

# Two components at one path are two components

The diff now matches each side component at most once, so a calendar holding one `UID` twice pairs its duplicates with the other side's duplicates instead of comparing both against the same one. Merging such a calendar with itself no longer reports a collision per duplicate naming a value nobody wrote.

The replay still addresses such a component by its path alone, so an action about the second of two may land on the first, which the spec now says out loud.

Capabilities moved: merge (ADDED: Two components at one path).
