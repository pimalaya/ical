---
cairn: change
id: identity-must-distinguish
status: landed
created: 2026-08-29
---

# An identity that does not distinguish is no identity

## Why

Two refinements the merge fuzz target found on the identity rule, both of which made a calendar collide with itself.

The identity was read as the first value of the line rather than the whole one, so an `ATTENDEE` carrying a list of calendar addresses was identified by the text up to its first comma. Two different lines then shared one identity and were matched with each other.

Nothing checked that an identity was unique among the siblings it is meant to tell apart. A component holding one calendar address twice, which RFC 5545 does not forbid since the two lines may differ in everything else, gave both lines one identity, and the merge treated them as one property.

## What

The identity is the whole raw value of the line, as written.

An identity shared with a same-named sibling is dropped, and that line falls back to its position, which is what the merge does for every property iCalendar gives no identity to. A sibling still alone with its value keeps its own identity, so one repeated address does not cost the rest of the group theirs.

Since one side may repeat a value where the other does not, a property carrying an identity is never matched with one carrying none: a position on one side does not answer for an identity on the other. Nor is an addition, which names a property the base did not hold and is numbered in the side that added it, matched with an action naming a property the base held: one side adding a property and the other editing a different one that happens to sit at the same position is not a disagreement.
