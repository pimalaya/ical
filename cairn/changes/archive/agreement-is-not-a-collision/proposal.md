---
cairn: change
id: agreement-is-not-a-collision
status: landed
created: 2026-08-29
---

# Two sides that did the same thing have not disagreed

## Why

`collides` compares the field two actions occupy and never the values they carry, so two sides that independently made the identical edit are reported as diverging. `merge(base, x, x)` reports a conflict for every change `x` made, and the conflict names the same action on both sides, which is a self-contradiction rather than a judgement call.

Convergent edits are the normal outcome when two replicas apply one server change, so this is the common path. A caller that prompts a person on any conflict prompts them for a decision both sides already made the same way, and a caller that falls back to keep-both keeps two copies of one value.

The spec is already on that side: the scenario says "both versions setting a different summary", and the requirement speaks of two people disagreeing.

## What

Two actions that are equal do not collide. The right side's action then applies as an uncontested one, which is a no-op on a calendar the left side already made the same way, and nothing is reported.

This also settles the removal arm, whose arithmetic treated two identical removals as a refusal that happened to produce the right bytes only because the merged calendar came from the side that had removed it.
