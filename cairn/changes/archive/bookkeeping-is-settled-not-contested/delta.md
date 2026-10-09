---
cairn: delta
change: bookkeeping-is-settled-not-contested
---

## ADDED Requirements

### Requirement: Bookkeeping is settled, not contested
Into merge. Two sides writing `DTSTAMP`, `LAST-MODIFIED` or `SEQUENCE` is no collision: the later stamp and the greater sequence stand, whichever side wrote them, and nothing is reported. A value that does not read is contested as any other. vcard-rs's twin is `REV`, not stated there yet.

## MODIFIED Requirements

## REMOVED Requirements
