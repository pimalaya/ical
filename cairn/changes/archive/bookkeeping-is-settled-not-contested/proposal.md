---
cairn: change
id: bookkeeping-is-settled-not-contested
status: landed
created: 2026-10-09
---

# Bookkeeping is settled, not contested

## Why

From the Android calendar's conflict work: every edit made on both sides conflicted on `DTSTAMP`, `LAST-MODIFIED` and `SEQUENCE`, since each side restamps what it writes. These are bookkeeping, not content: a conflict on them is noise the user can do nothing with, and it hid the conflicts that mattered.

## What

Settle them in the merge without a conflict: the later `DTSTAMP` and `LAST-MODIFIED` (RFC 5545 3.8.7.2, 3.8.7.3), the greater `SEQUENCE` (3.8.7.4, RFC 5546 2.1.4), whichever side wrote it, a value that does not read falling back to the ordinary rules. Check vcard-rs for the twin rule on `REV` and state the contract in the spec.
