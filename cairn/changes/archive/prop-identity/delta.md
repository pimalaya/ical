---
cairn: delta
change: prop-identity
---

## ADDED Requirements

### Requirement: Property identity

A property SHALL be addressed by an identity where iCalendar gives it one, and by its position among its same-named siblings only where it does not.

A property that may occur more than once in a component and whose value names a thing outside the calendar SHALL be identified by that value, compared as written: `ATTENDEE` by its calendar user address, `ATTACH` by its URI or inline binary, `RELATED-TO` by the `UID` it points at, `CONFERENCE` and `IMAGE` by their URI. Every other property SHALL carry no identity, since a property that may occur only once is already named by its name, and a property whose value is the datum would turn every edit into a replacement.

Two properties carrying different identities SHALL NOT be matched with each other, whatever their positions: a different calendar address is a different person, not a renamed one. The identity SHALL be reported with the action, so a caller is told which member of a group is contested.

Where no identity exists, a position SHALL name the same property in every calendar the merge resolves it against: the position an action carries is the one its target held in the base, and it SHALL be translated through the removals the baseline side made in that group before it is resolved against the merged calendar. The line an action's new bytes are read from SHALL be addressed by the position it holds in the side that wrote it.

#### Scenario: A side that only reordered and replaced an attendee

- GIVEN a base holding Ada and Zoe, an untouched left side, and a right side holding Zoe and Bob
- WHEN they are merged
- THEN the merged calendar is the right side, byte for byte, and nothing is reported

#### Scenario: An answer to an invitation

- GIVEN a left side that replaced Ada with Bob and a right side in which Ada answers
- WHEN they are merged
- THEN Ada's answer is never recorded against Bob

#### Scenario: A removal beside a contested neighbour

- GIVEN a base holding Ada and Bob, a left side that removed Ada and accepted for Bob, and a right side that declined for Bob
- WHEN they are merged
- THEN Bob appears once and the two answers are reported as one collision

## MODIFIED Requirements

### Requirement: A side's own actions all land

Where one side alone changed a calendar, the merged calendar SHALL be that side's, whatever the change was. Two removals from one group of same-named properties or sibling components SHALL both take effect, and SHALL take effect on the members the removing side removed.

The order the replay applies actions in is therefore not the order the diff produced them: an action addressed by a position is addressed by the position its target held in the base, and taking one member out renumbers the ones after it, so removals are replayed last and highest position first. What is reported, and in what order, is unchanged.

#### Scenario: Every member of a group removed

- GIVEN a component holding three attendees, and a version removing all three
- WHEN it is merged against an untouched other side
- THEN the merged component holds no attendee and nothing is reported

#### Scenario: The first members of a group removed

- GIVEN the same three attendees, and a version removing the first two
- WHEN it is merged against an untouched other side
- THEN the merged component holds the third one alone

### Requirement: Instance identity

A component SHALL be matched across versions by `UID` plus `RECURRENCE-ID`, so an override of one instance is never confused with the series it belongs to, however the two are ordered in the file. A component carrying no `UID` SHALL be matched by its position among its same-named siblings, and that position SHALL be counted the same way wherever the merge counts it: differently-named children do not shift each other.

Inside a matched component, a property SHALL be matched by its identity where it has one, and by its position among its same-named siblings otherwise.

#### Scenario: An override beside its series

- GIVEN a series and an override sharing a `UID`
- WHEN a version edits only the override, and writes it before the series
- THEN the series is untouched and only the override merges

#### Scenario: A change to a component that is not the first child

- GIVEN a `VTIMEZONE` whose `STANDARD` is written before its `DAYLIGHT`
- WHEN one version changes a property of the `DAYLIGHT`
- THEN the change is in the merged calendar
