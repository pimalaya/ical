---
cairn: delta
change: a-removed-component-comes-back-for-an-edit
---

## ADDED Requirements

### Requirement: A restored component comes back once
*Folds into merge.md.*

Where the baseline side removed a component the other side wrote something into, the component SHALL be restored once however many actions the other side made inside it, and SHALL come back as that side wrote it, whole. The restored subtree is the other side's own, bytes and all, so every one of its actions is already in it, and every action it carries SHALL then be let go rather than replayed onto it.

The component restored SHALL be the highest one the baseline side removed, not the one holding the edited line, so an edit nested any number of components below the removal brings back the component that actually went.

#### Scenario: An edit under a deleted component

- GIVEN one version deleting an event and the other changing the `TRIGGER` of the alarm inside it
- WHEN they are merged
- THEN the event comes back once, holding the alarm and the changed trigger, and the collision is reported

## MODIFIED Requirements

### Requirement: A removal meets what it takes away

A removal SHALL collide with every action the other side took on what it removes, not only with an action occupying the same field.

A whole-property removal SHALL collide with a change to that property's value, to any of its parameters and to any item of its list value. A component removal SHALL collide with any action addressed to that component or to anything nested inside it, at any depth, unless that action is itself a removal, since two sides taking overlapping things away have not disagreed.

The outcome is the one a collision already has, with granularity settling which side removes: a side that drops one parameter of a property keeps the property, so against a side that removed the property whole it is the one preserving data. The update beats the removal whichever side it came from, the surviving line or component is the updating side's whole, and the collision is reported.

Which of the two sides is the baseline SHALL NOT decide it. What the baseline side removed comes back for the other side's act, at both granularities, so merging one version into the other and merging it back the other way agree on what survives.

Only an act that writes something SHALL bring anything back, since an act that only takes something away has nothing to keep. A component one side removed and the other left alone therefore goes, and so does one both sides removed.

#### Scenario: An answer against a dropped attendee

- GIVEN one version answering an invitation by changing its `PARTSTAT` and the other removing the `ATTENDEE` line
- WHEN they are merged
- THEN the answered line survives and the collision is reported

#### Scenario: A reminder inside a deleted event

- GIVEN one version changing the `TRIGGER` of an alarm and the other removing the event holding it
- WHEN they are merged
- THEN the event survives carrying the changed trigger, whichever side removed it, and the collision is reported

#### Scenario: An occurrence deleted on one side and edited on the other

- GIVEN one version deleting an overriding occurrence and the other changing its `LOCATION`
- WHEN they are merged, and merged again with the two sides swapped
- THEN both merges hold the occurrence with the changed location, and both report one divergence

#### Scenario: An occurrence deleted with nobody editing it

- GIVEN one version deleting an overriding occurrence and the other leaving it alone, or both versions deleting it
- WHEN they are merged
- THEN the occurrence is gone and nothing is reported

## REMOVED Requirements

None.
