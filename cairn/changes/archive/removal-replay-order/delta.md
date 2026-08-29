---
cairn: delta
change: removal-replay-order
---

## ADDED Requirements

### Requirement: A side's own actions all land

Where one side alone changed a calendar, the merged calendar SHALL be that side's, whatever the change was. Two removals from one group of same-named properties or sibling components SHALL both take effect, and SHALL take effect on the members the removing side removed.

The order the replay applies actions in is therefore not the order the diff produced them: a removal is addressed by the position its target held in the base, and taking one member out renumbers the ones after it, so removals are replayed last and highest position first. What is reported, and in what order, is unchanged.

#### Scenario: Every member of a group removed

- GIVEN a component holding three attendees, and a version removing all three
- WHEN it is merged against an untouched other side
- THEN the merged component holds no attendee and nothing is reported

#### Scenario: The first members of a group removed

- GIVEN the same three attendees, and a version removing the first two
- WHEN it is merged against an untouched other side
- THEN the merged component holds the third one alone

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
