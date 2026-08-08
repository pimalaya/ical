---
cairn: change
id: recur-validate
status: landed
created: 2026-08-08
---

# IcalRecurRule::validate

## Why

"Strict on the way out" covers properties and components but not rules. The expander deliberately ignores parts RFC 5545 forbids at a given frequency, which is right for a liberal parser but leaves a caller with no principled way to learn that a rule is malformed.

## What

A `validate` on the decoded rule, mirroring the component-level one and minting the same kind of proof. RFC 5545 3.3.10 constrains which `BY` parts may appear at which frequency, forbids a `BYDAY` ordinal outside `MONTHLY` and `YEARLY`, forbids it at `YEARLY` when `BYWEEKNO` is present, and makes `UNTIL` and `COUNT` exclusive (that last one is already refused at parse).

Done when every constraint of 3.3.10 is reported, and `validate` on the calendar reaches the rules of its `RRULE` and `EXRULE` properties.
