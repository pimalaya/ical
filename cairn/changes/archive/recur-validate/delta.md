---
cairn: delta
change: recur-validate
---

## ADDED Requirements
### Requirement: Recurrence rules are validated

A decoded recurrence rule SHALL be checkable against RFC 5545 3.3.10, reporting every `BY` part the rule's frequency forbids, a `BYDAY` ordinal outside `MONTHLY` and `YEARLY`, a `BYDAY` ordinal at `YEARLY` beside `BYWEEKNO`, and `UNTIL` together with `COUNT`. Calendar validation SHALL reach the rules carried by `RRULE` and `EXRULE`.

Expansion stays liberal: a part validation reports is still ignored rather than refused when the rule is expanded.

#### Scenario: BYWEEKNO at a monthly frequency
- GIVEN `FREQ=MONTHLY;BYWEEKNO=3`
- WHEN the rule is validated
- THEN a forbidden-part problem is reported for `BYWEEKNO`
- AND expanding the same rule still ignores the part
