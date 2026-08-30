---
cairn: spec
capability: merge
status: current
---

# Three-way merge

Reconciling two divergent edits of a calendar against their common base, in the syntax layer, so the merged calendar is bytes rather than a re-encoding.

The merge is arranged as the steps it performs: addressing a calendar, diffing a side against the base, deciding when two sides performed one act, judging whether a right-side act lands, and replaying the ones that do.

### Requirement: Three-way merge against a stored base

Two divergent versions of a calendar SHALL reconcile against their common base, never by last-writer-wins. A field only one side touched SHALL be taken from that side. Every action each side took relative to the base, and every collision between them, SHALL be reported to the caller.

The merged calendar SHALL keep the untouched bytes of the left side: it starts as the left calendar and the right side's actions are replayed onto it line by line, so a line neither side edited comes out exactly as it went in, folds included. That is a statement about bytes alone: which side supplies the baseline does not decide which side wins a collision, which the caller states separately.

#### Scenario: Two edits to different properties

- GIVEN a base event, a left version with a new summary and a right version with a new location
- WHEN they are merged
- THEN both edits survive, nothing is reported, and a folded line neither side touched is still folded

#### Scenario: Two edits to the same property

- GIVEN both versions setting a different summary
- WHEN they are merged
- THEN the left side's summary is kept and the collision is reported rather than one side silently winning

#### Scenario: A removal against an update

- GIVEN one version removing a property and the other changing it
- WHEN they are merged
- THEN the changed property survives, whichever side changed it, and the collision is still reported

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

### Requirement: Ours wins, and the collision is still reported

The left side SHALL be `ours` and the right side `theirs`, in git's sense. The merged calendar SHALL be built from the left side's bytes, and where both sides changed one property to different things it SHALL carry the left side's value. Neither is a caller's to choose.

One side answering both questions is the point rather than a shortcut. A caller reaches for a merge holding a version it is merging into, and that version is both the one it would rather not churn and the one it means to keep. A caller wanting the other value has the collision in the report and can put it to somebody, which is a better answer than a flag that silently picks.

The rule SHALL decide only the case where both sides wrote a value. A property one side alone touched is still taken from that side, an untouched line still comes out byte for byte, an update still beats a removal whichever side it came from, and the report still names both actions.

#### Scenario: Both sides write a value

- GIVEN both versions setting a different summary
- WHEN they are merged
- THEN the merged calendar carries the left side's summary and the collision is reported

#### Scenario: The rule does not reach an uncontested property

- GIVEN a property only the right side changed
- WHEN they are merged
- THEN the right side's change survives and nothing is reported for it

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

### Requirement: Matching normalises, writing is exact

An identity SHALL be compared normalised and written back exactly. The comparison lowercases, so a URI scheme (RFC 3986 section 3.1) and the host of a calendar address meet whichever case they were written in. What goes back on the wire is the bytes the side that wrote them wrote, never a normalised form the merge chose.

The two halves are one rule. Comparing raw bytes misses a match that is there; writing the normalised form loses the byte fidelity the whole crate is for.

A case difference in a value is still a change, since only matching normalises: a side that rewrote the case of a scheme rewrote the value, and that change lands like any other.

#### Scenario: One calendar address in two cases

- GIVEN a base `ATTENDEE:MAILTO:Ada@Example.com`, a side adding a `CN` and a side that lowercased the address and answered
- WHEN they are merged
- THEN the component holds one attendee, carrying the answer

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

Two sides that made the same act SHALL NOT be reported as diverging, and the merged calendar SHALL carry that act once. A collision is two people disagreeing, and two identical acts are not that.

Two sides SHALL be held to have made one act only where they wrote the same bytes. An unescape is not injective, since `\N` and `\n` both read as a line break (RFC 5545 section 3.3.11), so two acts that decode alike may say different things on the wire, and reading those as one act drops the difference without a word. What is weighed is what the act itself wrote: the component or the line an addition put there, the value a change wrote, the item a list gained, the parameter a side wrote. An act that only takes something away wrote no bytes, and what it names lives in the base both sides share, so the act itself settles it.

The one exception SHALL be a parameter the specification gives no order: `DELEGATED-FROM` and `DELEGATED-TO` (sections 3.2.4 and 3.2.5), `MEMBER` (section 3.2.11) and `FEATURE` (RFC 7986 section 6.3) hold lists rather than sequences, so two sides writing one list in two orders SHALL be one act, its values compared as a set both decoded and raw.

Merging two identical sides SHALL therefore return those bytes and report nothing.

#### Scenario: A side merged with itself

- GIVEN a base, and two sides holding the same edits of it
- WHEN they are merged
- THEN the merged calendar carries those edits and nothing is reported

#### Scenario: One value spelled two ways

- GIVEN a base holding `SUMMARY:a`, a left side holding `SUMMARY:b\nc` and a right side holding `SUMMARY:b\Nc`
- WHEN they are merged
- THEN the divergence is reported and the merged calendar is the left side's bytes

#### Scenario: One unordered list parameter in two orders

- GIVEN a base whose `ATTENDEE` carries no `DELEGATED-TO`, and two sides adding the same two addresses in two orders
- WHEN they are merged
- THEN nothing is reported and the merged calendar is the left side's bytes

### Requirement: A list value is written back only when it changes

A list value SHALL be written back only where the replayed item really joins or leaves it. Writing a list back escapes every item afresh, so a replay that changes nothing would spell the baseline side's own items the canonical way and churn bytes nobody edited.

#### Scenario: An item both sides added

- GIVEN a base holding `CATEGORIES:a`, and two sides that both added `b`
- WHEN they are merged
- THEN the merged list holds `a,b` with the baseline side's own bytes

### Requirement: An addition that loses does not join the one that beat it

Where both sides added a property or a component the base lacked, the merged calendar SHALL hold the left side's alone and report the collision. The right side's addition SHALL NOT be written beside it, so the merged calendar never holds more members of a group than the side that wrote the most: a property RFC 5545 allows once is never emitted twice, and `validate` never refuses what the merge produced.

#### Scenario: Both sides setting a location the base lacked

- GIVEN a base with no `LOCATION` and two sides adding a different one
- WHEN they are merged
- THEN the merged event holds the left side's `LOCATION` alone and the collision is reported

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

What the replay writes into a line SHALL be the bytes the side that wrote them wrote, never a re-encoding of their decoded form. The two are not the same string: a decoded parameter holds a real line break where the wire holds `^n`, and a re-encoding writes the canonical RFC 6868 spelling of a value the side spelled another way. The decoded form is what is reported; it is not what is written.

#### Scenario: A bare record as one side

- GIVEN a well-formed base and left side, and a right side that is an envelope-less fragment holding a `BEGIN` line
- WHEN they are merged
- THEN the merged calendar parses and reparses to itself

#### Scenario: A parameter holding an encoded newline

- GIVEN a right side that changed a parameter whose value holds a `^n`
- WHEN they are merged
- THEN the merged line carries the parameter as the right side wrote it and the calendar parses

### Requirement: Two components at one path

Where a calendar holds several components at one path, a `UID` written twice with no `RECURRENCE-ID` telling them apart, each component of one side SHALL be matched with at most one component of the other. Comparing two of them with the same one would report the difference between the duplicates as a change a side made.

The replay addresses such a component by its path alone, so an action about the second of two may land on the first. What no addressing can tell apart the merge does not claim to, and it reports what it cannot settle rather than guessing.

#### Scenario: A calendar holding one UID twice

- GIVEN a calendar with two events sharing a `UID`, edited once
- WHEN it is merged with itself against the original
- THEN the merged calendar is that edit and nothing is reported

### Requirement: A value is compared as written

Two values SHALL be compared on their raw nodes, component by component, rather than on what they decode to. A decoded value reads its own kind's shape, and a text value reads its first `;`-component alone, so two lines saying different things past that point decode alike and the difference is never seen.

Two parameters SHALL be compared the same way, on their raw nodes and value by value, for the same reason: a single-valued parameter decodes its first value alone, so two parameters differing past their first `,` decode alike and the edit is never reported.

Where the two sides escape by different rules, only identical bytes SHALL count as the same value or the same parameter, there being no shared decoding to compare through.

#### Scenario: An edit past the first semicolon

- GIVEN a base and a side whose text value differs only after its first `;`
- WHEN they are diffed
- THEN the change is reported and the merged calendar carries it

#### Scenario: An edit past the first comma of a parameter

- GIVEN a base holding `ATTENDEE;CN=Ada,Lovelace` and a side that changed it to `CN=Ada,Byron`
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
