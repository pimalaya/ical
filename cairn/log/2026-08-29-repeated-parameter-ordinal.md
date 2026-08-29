---
cairn: log
change: repeated-parameter-ordinal
landed: 2026-08-29
---

# A parameter written twice is two parameters

Parameters are matched by name plus their position among the same-named parameters of the line, the field two parameter actions collide at carries that position, and the replay adds, changes and removes the occurrence the action named rather than the first of that name.

A line carrying `RSVP` twice no longer reports a change nobody made, which the merge fuzz target found on an ical4j fixture before mutating anything.

Capabilities moved: merge (ADDED: Repeated parameters).
