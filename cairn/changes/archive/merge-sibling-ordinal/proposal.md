---
cairn: change
id: merge-sibling-ordinal
status: landed
created: 2026-08-29
---

# A component's position is counted twice, two different ways

## Why

"A component carrying no `UID` SHALL be matched by its position among its same-named siblings" is one sentence, and the merge implements it in four places. Three of them count per name: `walk`, which builds the paths every op carries; `find_mut`, which resolves a path in the merged calendar; and the retain that removes a component. The fourth, `find`, counts over every child instead:

```rust
components(held)
    .enumerate()
    .find(|(ordinal, child)| {
        component_name(child) == step.name && key(child, *ordinal) == step.key
    })
```

`find` is how `apply` reads the right side's source, both for a whole component it adds and for the line a property action lands on. So a path naming `DAYLIGHT` ordinal 0 is offered `key(child, 1)`, because a `STANDARD` came first, and matches nothing. `apply_to_line` returns early, the change is not made, and `judge` never saw a collision, so nothing is reported either.

A `VTIMEZONE` defining both observances is in nearly every calendar that carries a zone, and its `DAYLIGHT` could not be merged at all. The same held for any second child of a different name: a `VALARM` after a `VLOCATION`, a `PARTICIPANT` after a `VALARM`. A second child of the *same* name was fine, the two counts coinciding there, which is why the suite never saw it: its only multi-child fixture is one `VALARM` on an event that had none.

Found by a property test asserting that a side which changed nothing yields the other side.

## What

- `find` counts ordinals per name, as its three siblings already do.
- One reproduction is pinned in the suite: a right-side change to a component that is not the first child of its parent.

## What this does not do

The wider question of a position being a poor identity is untouched. A base index resolved against a calendar whose left side inserted or removed a same-named sibling still names the wrong thing, which is a design question about what a replayed action should carry, not a counting error.
