---
cairn: log
change: identity-must-distinguish
landed: 2026-08-29
---

# An identity that does not distinguish is no identity

The identity is read off the whole raw value of a line rather than off its first value, so an `ATTENDEE` carrying a list of calendar addresses is no longer identified by the text up to its first comma. An identity a same-named sibling shares is dropped, and both of those lines fall back to their positions while the rest of the group keep theirs.

A property carrying an identity is never matched with one carrying none either, since one side may repeat a value where the other does not, and an addition, numbered in the side that added it, is never matched with an action naming a property the base held.

All three were found by the merge fuzz target, each as a calendar that collided with itself or changed under a merge with itself.

Capabilities moved: merge (MODIFIED: Property identity).
