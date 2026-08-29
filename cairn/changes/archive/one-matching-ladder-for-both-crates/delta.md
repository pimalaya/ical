---
cairn: delta
change: one-matching-ladder-for-both-crates
---

## ADDED Requirements
### Requirement: Matching normalises, writing is exact

An identity SHALL be compared normalised and written back exactly. The comparison lowercases, so a URI scheme (RFC 3986 section 3.1) and the host of a calendar address meet whichever case they were written in. What goes back on the wire is the bytes the side that wrote them wrote, never a normalised form the merge chose.

The two halves are one rule. Comparing raw bytes misses a match that is there; writing the normalised form loses the byte fidelity the whole crate is for.

A case difference in a value is still a change, since only matching normalises: a side that rewrote the case of a scheme rewrote the value, and that change lands like any other.

#### Scenario: One calendar address in two cases
- GIVEN a base `ATTENDEE:MAILTO:Ada@Example.com`, a side adding a `CN` and a side that lowercased the address and answered
- WHEN they are merged
- THEN the component holds one attendee, carrying the answer

## MODIFIED Requirements
### Requirement: Property identity

A property SHALL be matched across versions down one ladder: an explicit synchronisation identity, then a natural identity where the format gives the property one, then equality, then position among its same-named siblings. iCalendar defines no synchronisation identity for a property, so the first rung is empty here and the second is the first one consulted.

A property that may occur more than once in a component and whose value names a thing outside the calendar SHALL be identified by that value, the whole of it and as written: `ATTENDEE` by its calendar user address, `ATTACH` by its URI or inline binary, `RELATED-TO` by the `UID` it points at, `CONFERENCE` and `IMAGE` by their URI. Every other property SHALL carry no identity, since a property that may occur only once is already named by its name, and a property whose value is the datum would turn every edit into a replacement.

An identity SHALL tell a property from its same-named siblings or it is not one: where two of them carry the same value, both fall back to their positions, and a sibling still alone with its value keeps its own identity. A property carrying an identity SHALL NOT be matched with one carrying none, since the two are told apart differently and a position on one side does not answer for an identity on the other.

Two properties carrying different identities SHALL NOT be matched with each other, whatever their positions: a different calendar address is a different person, not a renamed one. The identity SHALL be reported with the action, so a caller is told which member of a group is contested.

Where no identity exists, a position SHALL name the same property in every calendar the merge resolves it against: the position an action carries is the one its target held in the base, and it SHALL be translated through the removals the baseline side made in that group before it is resolved against the merged calendar. The line an action's new bytes are read from SHALL be addressed by the position it holds in the side that wrote it.

An addition is the exception, since it names a property the base did not hold: its position is the one it holds in the side that added it, and it SHALL NOT be compared with the position of an action that names a property the base held. Two additions of one name still meet each other.

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

#### Scenario: One calendar address on two attendees

- GIVEN a component holding one calendar address twice, edited once
- WHEN it is merged with itself against the original
- THEN the merged calendar is that edit and nothing is reported
