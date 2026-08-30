---
cairn: delta
change: agreement-is-byte-equality
---

## ADDED Requirements

### Requirement: A list value is written back only when it changes

A list value SHALL be written back only where the replayed item really joins or leaves it. Writing a list back escapes every item afresh, so a replay that changes nothing would spell the baseline side's own items the canonical way and churn bytes nobody edited.

#### Scenario: An item both sides added

- GIVEN a base holding `CATEGORIES:a`, and two sides that both added `b`
- WHEN they are merged
- THEN the merged list holds `a,b` with the baseline side's own bytes

## MODIFIED Requirements

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

## REMOVED Requirements
