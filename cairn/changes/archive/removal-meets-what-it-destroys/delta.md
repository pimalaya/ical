---
cairn: delta
change: removal-meets-what-it-destroys
---

## ADDED Requirements

### Requirement: A removal meets what it takes away

A removal SHALL collide with every action the other side took on what it removes, not only with an action occupying the same field.

A whole-property removal SHALL collide with a change to that property's value, to any of its parameters and to any item of its list value. A component removal SHALL collide with any action addressed to that component or to anything nested inside it, at any depth, unless that action is itself a removal, since two sides taking overlapping things away have not disagreed.

The outcome is the one a collision already has: an update beats a removal whichever side it came from, the surviving line or component is the updating side's whole, and the collision is reported.

#### Scenario: An answer against a dropped attendee

- GIVEN one version answering an invitation by changing its `PARTSTAT` and the other removing the `ATTENDEE` line
- WHEN they are merged
- THEN the answered line survives and the collision is reported

#### Scenario: A reminder inside a deleted event

- GIVEN one version changing the `TRIGGER` of an alarm and the other removing the event holding it
- WHEN they are merged
- THEN the collision is reported rather than the alarm disappearing in silence
