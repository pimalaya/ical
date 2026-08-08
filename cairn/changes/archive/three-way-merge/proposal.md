---
cairn: change
id: three-way-merge
status: landed
created: 2026-08-08
---

# Three-way merge

## Why

vcard-rs has tree/merge.rs, and io-offline and cardamum depend on it. Calendula needs exactly the same for events, and the mobile conflict rules are already settled: merge against the stored base rather than last-writer-wins, and keep both over silent loss.

## What

Port the vCard implementation, replacing `PID` matching with `UID` plus `RECURRENCE-ID` as the instance identity, and add the calendar-specific axes: recurrence (a change to the rule is not the same as a change to one instance) and organiser authority (an attendee may not rewrite what the organiser owns).

Done when two divergent edits of a calendar reconcile with every action and conflict reported, and the merged calendar keeps the untouched bytes of the left side.
