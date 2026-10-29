---
cairn: delta
change: itip-trust-first
---

## ADDED Requirements

### Requirement: Trust is checked before freshness

An iTIP decision SHALL refuse a `REQUEST`, `ADD` or `CANCEL` whose authenticated sender, as the caller supplies it, is not the component's `ORGANIZER`, and a `REPLY` whose sender is not the `ATTENDEE` it answers for, before comparing any revision. A refusal SHALL name the check that failed.

#### Scenario: A forged request with a higher sequence
- GIVEN a held event at `SEQUENCE:1` and a `REQUEST` for its `UID` at `SEQUENCE:9` from a sender other than its `ORGANIZER`
- WHEN the decision is made
- THEN the request is refused as untrusted, and the sequence is never compared

### Requirement: A revision names its instance

A message SHALL apply when its `(SEQUENCE, DTSTAMP)` is greater than the one held for its instance, the `UID` and the `RECURRENCE-ID`, and a `REPLY`'s revision SHALL be compared per `(instance, attendee)`.

#### Scenario: Two attendees replying out of order
- GIVEN a held event with replies recorded from two attendees
- WHEN the first attendee's older reply arrives after the second's newer one
- THEN the second attendee's answer is kept and only the first's own revision is compared
