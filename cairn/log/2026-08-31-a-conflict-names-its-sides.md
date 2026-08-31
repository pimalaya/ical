---
cairn: log
change: a-conflict-names-its-sides
date: 2026-08-31
---

# A conflict names one side and describes the other

`IcalMergeConflict` was `{ right, reason }` where vcard-rs's twin is `{ left, right }`. The two carry the same pair of actions, so the difference was in the naming alone: one side named after itself, the other described from the outside by the enum wrapping it.

The field is now `left`, and it comes first. `IcalMergeReason` stays as its type, because the enum is not the accident here: iCalendar has a second conflict kind, a change to a series meeting a change to one of its instances, and that kind has to be said somewhere. What the enum should not also have to say is whose action it is carrying, and the `Divergent` documentation ended on "beside the right side's on the conflict itself" precisely because the field name would not say it.

The consumer shows the gain. tCal read `sides.read(&conflict.right, &conflict.reason)`, right before left, because the field order said so; it now reads `sides.read(&conflict.left, &conflict.right)`, and `Sides::read` takes its two parameters in the same order.

This is a break for anybody matching on the field, which is every caller of the merge, and the fix is the rename. Nine test assertions, one example and one consumer moved with it, and no law changed: the same conflicts are reported, in the same order, carrying the same two actions.

Capabilities moved: merge.
