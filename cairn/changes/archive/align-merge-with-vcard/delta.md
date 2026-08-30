---
cairn: delta
change: align-merge-with-vcard
---

## ADDED Requirements

### Requirement: A value is compared as written

Two values SHALL be compared on their raw nodes, component by component, rather than on what they decode to. A decoded value reads its own kind's shape, and a text value reads its first `;`-component alone, so two lines saying different things past that point decode alike and the difference is never seen.

Where the two sides escape by different rules, only identical bytes SHALL count as the same value, there being no shared decoding to compare through.

#### Scenario: An edit past the first semicolon

- GIVEN a base and a side whose text value differs only after its first `;`
- WHEN they are diffed
- THEN the change is reported and the merged calendar carries it

### Requirement: A list is a multiset

A list value SHALL be diffed and replayed as a multiset. One item leaving a list that held it twice SHALL be reported as one removal, and SHALL take one item rather than every item equal to it.

#### Scenario: One of two equal items leaves

- GIVEN a base holding `a,a,b` and a side holding `a,b`
- WHEN the removal is reported and replayed
- THEN the merged list holds `a,b`

### Requirement: A replay target follows its property

Where the baseline side added a property, the position a replayed action lands on SHALL account for that addition as well as for the baseline side's removals. The merged calendar is the baseline side's own tree, so a line it inserted moves every base-derived line at or after it.

#### Scenario: An insertion above an edit

- GIVEN a baseline side that inserted a property above two it kept
- WHEN the other side's edit of the second one is replayed
- THEN the edit lands on that property, and the one above it is untouched

### Requirement: A restored property comes back once

Where the baseline side removed a property the other side edited, the property SHALL be restored once however many actions the other side made on it. The restored line is the other side's own, bytes and all, so every one of its actions is already in it.

#### Scenario: Two edits to a removed property

- GIVEN a baseline side that removed a property, and another side that changed both its value and one of its parameters
- WHEN the actions are replayed
- THEN the merged component holds that property once

### Requirement: Retyping a value contests it

A change to the `VALUE` parameter SHALL collide with a value-level action on the other side, and a whole-value change SHALL collide with the other side's item edits. `VALUE` declares how the value is read, so items written under the old type cannot stand beside the new one (RFC 5545 section 3.8.5.2).

#### Scenario: A retype against an addition

- GIVEN one side adding an `RDATE` item and the other retyping the property to `PERIOD`
- WHEN they are merged
- THEN the collision is reported rather than both landing

## REMOVED Requirements

### Requirement: Organiser authority

Removed. The merge no longer takes a calendar address for a side, and no change is refused for want of authority. The field named a fixed side rather than a role, which forced the one caller using it to place its local calendar opposite to every other caller in the ecosystem.
