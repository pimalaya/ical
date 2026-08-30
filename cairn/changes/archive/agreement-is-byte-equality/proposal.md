---
cairn: change
id: agreement-is-byte-equality
status: landed
created: 2026-08-30
---

# Agreement is byte equality

## Why

One defect shape has been fixed three times in two days: comparing the decoded form of something whose decode is not injective. It was fixed for values (`value_eq`), for URIs and for parameters (`param_eq`). Action-level agreement was the fourth site and still compared decoded actions.

`Merger::agrees` required raw byte equality for a property or a component added, and nothing at all for everything else. So two sides that wrote different bytes decoding alike, `SUMMARY:a\nb` against `SUMMARY:a\Nb`, produced equal actions, the right side's act was skipped as already made, and the divergence was never reported.

Separately, the crate had no order-insensitivity anywhere, while vcard-rs compares the items of `TYPE` and `PID` as sets. iCalendar has unordered list parameters too, and nothing said so.

## What

Make agreement byte equality at the granularity of the act itself: the component or the line an addition put there, the value a change wrote, the item a list gained, the parameter a side wrote. An act that only takes something away wrote no bytes, and what it names lives in the base both sides share, so the act settles it.

The one exception is a parameter the specification gives no order: `DELEGATED-FROM` and `DELEGATED-TO` (RFC 5545 sections 3.2.4 and 3.2.5), `MEMBER` (section 3.2.11) and `FEATURE` (RFC 7986 section 6.3). Those compare as sets, decoded and raw alike, so writing one list in two orders stays one act.

Done when a spelling-only difference is reported rather than swallowed, an unordered list parameter written in two orders is still agreement, and both crates state one rule.

## Consequence

The merged bytes do not move: a refused agreement is judged normally, collides with the left act, and the left side keeps its value, so only the report gains an entry.

A list value is now written back only where an item really joins or leaves it, since a write-back escapes every item afresh and a replay that changes nothing would otherwise churn the left side's own spelling.
