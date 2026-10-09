---
cairn: log
change: bookkeeping-is-settled-not-contested
date: 2026-10-09
---

# Bookkeeping is settled, not contested

Every component both sides edited conflicted on `DTSTAMP`, `LAST-MODIFIED` and `SEQUENCE`, each side having restamped what it wrote. Those record when and how often a component was written (RFC 5545 3.8.7.2 to 3.8.7.4), not what it says, and the conflicts they raised buried the ones that mattered.

`settles` (tree/merge/judge.rs) runs before the collision search: when both sides wrote one of the three on the same component, by a value change or an addition, the right side's lands only when it is the later stamp or the greater sequence (RFC 5546 2.1.4 for `SEQUENCE`), and nothing is reported either way. A sequence compares as a number, a stamp as a date-time; a value that reads as neither leaves the pair to the ordinary rules, so it is contested as before.

The property tests (tests/merge_props.rs) state the rule in their laws and their reference merge, and the corpus replay, whose edits append text that no longer reads as a date-time, checks the fallback.

vcard-rs has no twin rule for `REV` (RFC 6350 6.7.4): its merge (src/tree/merge) contests a `REV` both sides wrote like any other value. The ical merge spec says so; aligning the two needs a change in vcard-rs.

Capabilities moved: merge.
