---
cairn: delta
change: quoted-parameter-values
---

## ADDED Requirements

### Requirement: A quoted parameter value is opaque

The line splitter SHALL treat a double-quoted parameter value as opaque, per RFC 5545 section 3.2: neither the `:` separating the head from the value nor the `;` separating one parameter from the next is recognised inside one.

A head carrying an unbalanced quote SHALL still parse: with no `:` outside quotes the splitter falls back to the first `:` anywhere, so a malformed line yields a line rather than an error.

#### Scenario: The RFC 5545 section 3.2.1 alternate representation
- GIVEN a line reading `DESCRIPTION;ALTREP="cid:part1.0001@example.org":Meeting notes`
- WHEN it is parsed
- THEN it carries one parameter, `ALTREP` holding the whole quoted URI, and the value reads `Meeting notes`

#### Scenario: An unbalanced quote
- GIVEN a line reading `ATTENDEE;CN="Ada:mailto:ada@example.com`
- WHEN it is parsed
- THEN it parses at the first colon anywhere and round-trips unchanged

## MODIFIED Requirements

## REMOVED Requirements
