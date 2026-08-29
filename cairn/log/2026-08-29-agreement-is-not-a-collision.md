---
cairn: log
change: agreement-is-not-a-collision
landed: 2026-08-29
---

# Two sides that did the same thing have not disagreed

Two identical actions are no longer a collision, and an act the baseline side already performed is not replayed, since the merged calendar is that side. An addition counts as the same addition only where both sides wrote the same bytes, which is what tells two sides adding one attendee with different parameters from two sides adding the same line.

`merge(base, x, x)` now returns `x` and reports nothing under either preference, and a recurrence pair the replayed side made in full is one person's own edit rather than two people disagreeing about a series.

Capabilities moved: merge (ADDED: Agreement is not a collision).
