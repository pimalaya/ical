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

A component SHALL be matched across versions by `UID` plus `RECURRENCE-ID`, so an override of one instance is never confused with the series it belongs to, however the two are ordered in the file. A component carrying no `UID` SHALL be matched by its position among its same-named siblings.

#### Scenario: An override beside its series

- GIVEN a series and an override sharing a `UID`
- WHEN a version edits only the override, and writes it before the series
- THEN the series is untouched and only the override merges

### Requirement: A series and its instances

A change to a series and a change to one of its instances SHALL both survive, and SHALL be reported together: a rule that moved may have moved the ground the override stood on, and only the caller can know whether that matters.

#### Scenario: A rule change against an instance change

- GIVEN one version changing the `RRULE` and the other changing an overriding instance's start
- WHEN they are merged
- THEN both changes are in the merged calendar and the pair is reported

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
