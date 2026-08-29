---
cairn: change
id: removal-meets-what-it-destroys
status: landed
created: 2026-08-29
---

# A removal meets everything it takes away

## Why

`collides` matches two actions by the field they occupy, and it is narrower than what a removal actually destroys. Two shapes escape it.

A whole-property removal occupies the property field; a parameter change occupies one parameter field and a list item occupies the item field. None of those pairs collide, so an attendee accepting an invitation while the other side drops the attendee line passes the removal by in silence: the answer is gone and nothing is reported. The mirror is as bad in the report even where the outcome is right: the removal is undone by the updating side and the caller is not told.

A component removal collides only with actions addressed to that component itself, because the whole table sits behind `if left.path() != right.path()`. An action addressed to something nested inside the removed component has a longer path and never meets the removal. One replica deletes an event while the other adds a reminder to it or answers inside it, and the merge reports a clean result with the reminder gone. The one-level case already behaves correctly, so this is a defect about depth alone.

## What

`collides` is widened twice, and the outcome rule is unchanged: an update beats a removal whichever side it came from, and every collision is reported.

A whole-property removal collides with any action on that property, its parameters and its list items included. The surviving line is then the updating side's, whole, which is what a value collision against a removal already does.

A component removal collides with any action addressed to that component or to anything nested inside it, at any depth, unless that action is itself a removal: two sides taking overlapping things away have not disagreed, and reporting that pair would be noise.

## Judgement call the owner should review

The outcome is asymmetric between the two directions, and deliberately so, because only the right side's actions are replayed. Where the right side removes a component the left side worked inside, the removal is refused and the whole subtree survives with the left side's work in it. Where the left side removes it and the right side worked inside, the right side's action is reported and does not land, since the merged calendar is built from the left side and there is nothing there to land on. That asymmetry is what the one-level case already does today; this change makes depth stop mattering rather than inventing a new rule for it.
