---
cairn: delta
change: recurrence-set
---

## ADDED Requirements
### Requirement: The recurrence set of a component

A component's recurrence set SHALL be the union of its `DTSTART`, every `RRULE` expansion and every `RDATE`, minus every `EXDATE` and every `EXRULE` expansion, yielded in chronological order with duplicates collapsed.

#### Scenario: A rule plus an extra date minus an exception
- GIVEN a `VEVENT` carrying one `RRULE`, one `RDATE` and one `EXDATE` falling on a rule occurrence
- WHEN its recurrence set is expanded
- THEN the extra date appears in order and the excepted occurrence does not

### Requirement: Set expansion stays lazy

The recurrence set SHALL be produced by a lazy merge of sorted streams, materialising no occurrence list, so an unbounded rule can be taken from without running to its end.

#### Scenario: An unbounded rule
- GIVEN a `VEVENT` whose `RRULE` has neither `UNTIL` nor `COUNT`
- WHEN the first ten occurrences are taken
- THEN only bounded work is done

### Requirement: Overrides replace instances

A `RECURRENCE-ID` override SHALL replace the instance it names. With `RANGE=THISANDFUTURE` it SHALL replace that instance and every later one, shifting them by the same offset the override applies to its own start.

#### Scenario: A single moved instance
- GIVEN an override whose `RECURRENCE-ID` names the third occurrence and whose `DTSTART` is an hour later
- WHEN the set is expanded
- THEN the third occurrence is the overridden one and the others are unchanged

#### Scenario: A moved tail
- GIVEN the same override carrying `RANGE=THISANDFUTURE`
- WHEN the set is expanded
- THEN the third and every later occurrence are shifted by an hour
