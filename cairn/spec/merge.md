---
cairn: spec
capability: merge
status: current
---

# Three-way merge

Reconciling two divergent edits of a calendar against their common base, in the syntax layer, so the merged calendar is bytes rather than a re-encoding.

### Requirement: Three-way merge against a stored base

Two divergent versions of a calendar SHALL reconcile against their common base, never by last-writer-wins. A field only one side touched SHALL be taken from that side. Every action each side took relative to the base, and every collision between them, SHALL be reported to the caller.

The merged calendar SHALL keep the untouched bytes of the left side: it starts as the left calendar and the right side's actions are replayed onto it line by line, so a line neither side edited comes out exactly as it went in, folds included. That is a statement about bytes alone: which side supplies the baseline does not decide which side wins a collision, which the caller states separately.

#### Scenario: Two edits to different properties

- GIVEN a base event, a left version with a new summary and a right version with a new location
- WHEN they are merged
- THEN both edits survive, nothing is reported, and a folded line neither side touched is still folded

#### Scenario: Two edits to the same property

- GIVEN both versions setting a different summary, and a caller stating no preference
- WHEN they are merged
- THEN the left side's summary is kept and the collision is reported rather than one side silently winning

#### Scenario: A removal against an update

- GIVEN one version removing a property and the other changing it
- WHEN they are merged
- THEN the changed property survives, whichever side changed it and whatever the preference, and the collision is still reported

#### Scenario: Two edits to one list

- GIVEN both versions adding a different keyword, one of them also removing an existing one
- WHEN they are merged
- THEN both additions and the removal all apply, and nothing is reported

### Requirement: Instance identity

A component SHALL be matched across versions by `UID` plus `RECURRENCE-ID`, so an override of one instance is never confused with the series it belongs to, however the two are ordered in the file. A component carrying no `UID` SHALL be matched by its position among its same-named siblings, and that position SHALL be counted the same way wherever the merge counts it: differently-named children do not shift each other.

Inside a matched component, a property SHALL be matched by its identity where it has one, and by its position among its same-named siblings otherwise.

A `UID` is the only identity iCalendar gives a component, so a group of same-named siblings that carry none, several `VALARM`s of one event or the observances of one time zone, is matched by position alone. A side that removes one of them therefore pairs the survivors with the base by position, and the merge describes that as a change to the first rather than as the removal it was. Every action still lands or is reported, and no value is lost, but the lines involved come out as neither side wrote them.

A calendar holding two components at one path, one `UID` written twice with no `RECURRENCE-ID` telling them apart, which RFC 5545 3.8.4.7 does not allow, is beyond what any addressing can tell apart. The merge pairs them in the order they are written and reports what it cannot settle, but an action addressed to the second may land on the first.

#### Scenario: An override beside its series

- GIVEN a series and an override sharing a `UID`
- WHEN a version edits only the override, and writes it before the series
- THEN the series is untouched and only the override merges

#### Scenario: A change to a component that is not the first child

- GIVEN a `VTIMEZONE` whose `STANDARD` is written before its `DAYLIGHT`
- WHEN one version changes a property of the `DAYLIGHT`
- THEN the change is in the merged calendar

### Requirement: A series and its instances

A change to what defines a series and a change to one of its instances SHALL both survive, and SHALL be reported together: a rule that moved may have moved the ground the override stood on, and only the caller can know whether that matters.

What defines the series is its `DTSTART`, `DTEND`, `DURATION`, `RRULE`, `RDATE` and `EXDATE`, and the series component itself. A change to anything else the series carries cannot have moved an occurrence, and SHALL NOT be reported against one.

#### Scenario: A rule change against an instance change

- GIVEN one version changing the `RRULE` and the other changing an overriding instance's start
- WHEN they are merged
- THEN both changes are in the merged calendar and the pair is reported

#### Scenario: A description change against an instance change

- GIVEN one version changing the series' `LOCATION` and the other changing an overriding instance's summary
- WHEN they are merged
- THEN both changes are in the merged calendar and nothing is reported

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

### Requirement: Organiser authority

Where the caller says which calendar address the right side edits as, a right-side change to a property only the organiser may set SHALL be refused and reported (RFC 5546 3.2). An attendee owns their own `ATTENDEE` line, the transparency they show, their alarms and anything outside the vocabulary; everything describing the meeting is the organiser's.

Where the caller says nothing, no claim is made and nothing is refused on this ground.

Authority SHALL stay on the replayed side, and a refusal SHALL NOT depend on the collision preference: a permission check belongs to the change being applied rather than to the baseline it is applied to. A caller therefore puts the edit it speaks for on the replayed side, and states its preference separately when that edit should also win.

#### Scenario: An attendee moving a meeting

- GIVEN a right side speaking for an attendee, changing the start of a meeting someone else organises
- WHEN it is merged, under either preference
- THEN the start does not change and the refusal is reported

#### Scenario: An attendee answering an invitation

- GIVEN the same right side changing its own `PARTSTAT`
- WHEN it is merged
- THEN the answer applies and nothing is reported

### Requirement: The winning side is chosen, not implied

A caller SHALL be able to say which side's value the merged calendar carries when both sides changed one property to different things, independently of which side supplies the baseline bytes. Where the caller says nothing, the left side wins, which is what a merge has always done.

The two are different questions. Which side is the baseline decides whose folding, whose parameter casing and whose property order survive untouched, and is answered by whichever version the caller would rather not churn. Which side wins a collision is a statement about two people disagreeing, and is answered by what the caller knows about them. Deciding the second by the first makes a byte-fidelity choice settle a data-loss one.

Authority is what forces the split rather than mere tidiness. Only the replayed side is judged, so a caller wanting its own edit refused where it exceeds an attendee's authority has to put that edit on the replayed side, and without a preference stated apart from the baseline that would be the same act as making it lose every collision. A caller would then be choosing between refusing what a person may not change and keeping what they did change, which are not alternatives.

The preference SHALL decide only the case where both sides wrote a value. A property one side alone touched is still taken from that side, an untouched line still comes out byte for byte, and the report still names both actions whichever way the preference falls.

#### Scenario: The right side is preferred

- GIVEN both versions setting a different summary, and a caller preferring the right side
- WHEN they are merged
- THEN the merged calendar carries the right side's summary and the collision is reported as it always was

#### Scenario: The preference does not reach an uncontested property

- GIVEN a property only the left side changed, and a caller preferring the right side
- WHEN they are merged
- THEN the left side's change survives and nothing is reported for it

#### Scenario: Being judged no longer costs the collision

- GIVEN a right side speaking for an attendee, changing its own `PARTSTAT` and also setting a summary the left side set differently, with the right side preferred
- WHEN they are merged
- THEN the answer applies, the summary is the right side's, and only the summary is reported

### Requirement: Property identity

A property SHALL be addressed by an identity where iCalendar gives it one, and by its position among its same-named siblings only where it does not.

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

### Requirement: A removal meets what it takes away

A removal SHALL collide with every action the other side took on what it removes, not only with an action occupying the same field.

A whole-property removal SHALL collide with a change to that property's value, to any of its parameters and to any item of its list value. A component removal SHALL collide with any action addressed to that component or to anything nested inside it, at any depth, unless that action is itself a removal, since two sides taking overlapping things away have not disagreed.

The outcome is the one a collision already has, with granularity settling which side removes: a side that drops one parameter of a property keeps the property, so against a side that removed the property whole it is the one preserving data. The update beats the removal whichever side it came from, the surviving line or component is the updating side's whole, and the collision is reported.

#### Scenario: An answer against a dropped attendee

- GIVEN one version answering an invitation by changing its `PARTSTAT` and the other removing the `ATTENDEE` line
- WHEN they are merged
- THEN the answered line survives and the collision is reported

#### Scenario: A reminder inside a deleted event

- GIVEN one version changing the `TRIGGER` of an alarm and the other removing the event holding it
- WHEN they are merged
- THEN the collision is reported rather than the alarm disappearing in silence

### Requirement: Agreement is not a collision

Two sides that made the same change SHALL NOT be reported as diverging, and the merged calendar SHALL carry that change once. A collision is two people disagreeing, and two identical actions are not that. An addition is the same addition only where both sides wrote the same bytes, since an addition names where it lands and what it says but not how it is spelt.

Merging two identical sides SHALL therefore return those bytes and report nothing, under either preference.

#### Scenario: A side merged with itself

- GIVEN a base, and two sides holding the same edits of it
- WHEN they are merged
- THEN the merged calendar carries those edits and nothing is reported

### Requirement: An addition that wins replaces the one it beat

Where both sides added a property or a component the base lacked and the merge keeps the replayed side's, the addition it beat SHALL be replaced where it stood rather than left beside it. The merged calendar SHALL never hold more members of a group than the side that wrote the most, so a property RFC 5545 allows once is never emitted twice and `validate` never refuses what the merge produced, and a position addressing the members of a group SHALL NOT be renumbered by the replacement.

#### Scenario: Both sides setting a location the base lacked

- GIVEN a base with no `LOCATION` and two sides adding a different one, with the right side preferred
- WHEN they are merged
- THEN the merged event holds the right side's `LOCATION` alone and the collision is reported

### Requirement: Repeated parameters

A property carrying one parameter name more than once SHALL have each occurrence matched with the occurrence at the same position on the other side, and a parameter action SHALL address the occurrence it named rather than the first of that name. Two actions on two different occurrences SHALL NOT collide.

#### Scenario: A line carrying one name twice

- GIVEN a property written with the same parameter name twice
- WHEN a calendar is merged with itself against itself
- THEN no action and no collision is reported

### Requirement: The merged calendar can always be read back

Whatever the three calendars hold, the merged calendar SHALL parse, and SHALL reparse to the same bytes. A merge never emits a calendar its own parser refuses, and never emits one that loses content on the next read.

`BEGIN` and `END` are the component envelope rather than properties, whichever side carries them. A bare, envelope-less record, which the parser accepts so a lone fragment round-trips, SHALL therefore contribute its properties alone: no side is reported as adding or removing a structural line, and none is ever copied into the merged calendar.

A line copied out of one side SHALL carry a line ending. The last line of a truncated download has none, and copied into the middle of a calendar it would swallow the line after it. The untouched bytes of the baseline side are not affected: only what the replay copies is terminated.

#### Scenario: A bare record as one side

- GIVEN a well-formed base and left side, and a right side that is an envelope-less fragment holding a `BEGIN` line
- WHEN they are merged
- THEN the merged calendar parses and reparses to itself

### Requirement: Two components at one path

Where a calendar holds several components at one path, a `UID` written twice with no `RECURRENCE-ID` telling them apart, each component of one side SHALL be matched with at most one component of the other. Comparing two of them with the same one would report the difference between the duplicates as a change a side made.

The replay addresses such a component by its path alone, so an action about the second of two may land on the first. What no addressing can tell apart the merge does not claim to, and it reports what it cannot settle rather than guessing.

#### Scenario: A calendar holding one UID twice

- GIVEN a calendar with two events sharing a `UID`, edited once
- WHEN it is merged with itself against the original
- THEN the merged calendar is that edit and nothing is reported
