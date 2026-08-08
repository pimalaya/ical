---
cairn: change
id: recurrence-set
status: landed
created: 2026-08-08
---

# The recurrence set, not one rule

## Why

`recur` expands a single `RRULE` today. What a client needs is the set a component denotes. This is the biggest functional hole in the crate: no caller can answer "when does this event actually happen?" without reimplementing the merge itself.

## What

Expand `DTSTART` plus every `RRULE`, plus every `RDATE` (including its period form), minus every `EXDATE`, minus every `EXRULE` (deprecated but still on the wire), with `RECURRENCE-ID` overrides replacing an instance and `RANGE=THISANDFUTURE` replacing an instance and everything after it. The existing iterator is the right substrate: merge several sorted streams lazily rather than materialising anything.

Done when a `VEVENT` or `VTODO` with any mix of those properties yields its occurrences in order, overrides applied, with the same laziness guarantee the single-rule iterator gives, and when the RFC 5545 examples that combine `RRULE` with `RDATE` and `EXDATE` are covered as tests.
