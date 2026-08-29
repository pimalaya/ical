---
cairn: change
id: removal-replay-order
status: landed
created: 2026-08-29
---

# Two removals from one group, and only one of them happens

## Why

A right-side removal carries the position its target held in the base, and the replay resolves that position against the merged calendar. Replaying two removals from one group of same-named properties or sibling components in the order the diff produced them therefore takes the first one out and then names the second by a position that no longer holds it.

Three attendees, all three removed on one side, the other side untouched:

```text
base:   ATTENDEE;CN=Ada  ATTENDEE;CN=Bob  ATTENDEE;CN=Cyd
right:  none
merged: ATTENDEE;CN=Bob
```

The merge reports three removals and no conflict, and Bob is still there. Removing the first two is worse than incomplete, it is wrong: Cyd goes and Bob stays, so the merged calendar keeps someone the right side removed and drops someone it kept. Three `VALARM`s behave the same way.

`merge(base, base, right)` has to be `right`: only one side changed anything, so there is nothing to reconcile. It is not, and nothing in the report says so.

## What

- The replay applies removals last, highest position first, so each one still names in the merged calendar what it named in the base. Everything else keeps the order the diff produced.
- Judging is unchanged and still runs in diff order, so the report and the order of its conflicts are exactly what they were.

## What this does not do

The wider problem stays: a position is a poor name for a property or a `UID`-less component, and a base position resolved against a calendar the *left* side renumbered still names the wrong thing. Ordering the replay fixes the case where the replayed side is the only one that touched the group, which is the case that loses data with nobody disagreeing about anything. The rest wants an identity rather than a position, which is a design decision.
