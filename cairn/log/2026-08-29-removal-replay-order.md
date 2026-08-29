---
cairn: log
change: removal-replay-order
landed: 2026-08-29
---

# Two removals from one group, and only one of them happened

A removal carries the position its target held in the base, and the replay resolved that position against the merged calendar as it stood after the removals before it. Replaying two removals from one group of same-named properties in diff order therefore took the first one out and then named the second by a position that no longer held it. Three attendees, all three removed on one side and the other side untouched, came out as one attendee left standing, with three removals reported and no conflict. Removing the first two was worse than incomplete: the third went and the second stayed, so the merged calendar kept someone the removing side had removed and dropped someone it had kept. Three `VALARM`s did the same, a `VALARM` being addressed by its position for want of a `UID`.

What makes this the plainest kind of defect is that only one side had changed anything. `merge(base, base, right)` has to be `right`, there being nothing to reconcile, and the report agreed with that reading: it named every removal and raised no conflict. The bytes simply did not follow.

The fix separates judging from applying. The judge still runs over the right side's actions in diff order, so the report and the order of its conflicts are byte for byte what they were, and the actions it lets through are then sorted before they are replayed: removals last, highest position first, everything else stable in diff order. Each removal then names in the merged calendar what it named in the base. Four lines of ordering and one loop split in two.

The spec gains "A side's own actions all land", which says the outcome rather than the mechanism, with the two scenarios that fail without it: every member of a group removed, and the first members of a group removed. The second matters more than it looks. A test asserting only that one attendee survived would have passed the broken merge, so it asserts which one.

What this does not touch is deliberate and is written up in the campaign's findings under ical-positional-index-applied-to-the-wrong-line.md: a position is a poor name for a property, and a base position resolved against a calendar the *left* side renumbered still names the wrong thing, so a reply can still land on another attendee. Ordering the replay fixes the case where the replayed side is the only one that touched the group. The rest wants an identity rather than a position, which is a design decision rather than a repair.

Found by a property test asserting that a side which changed nothing yields the other side.

Capabilities moved: merge, new requirement "A side's own actions all land".
