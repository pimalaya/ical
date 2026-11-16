---
cairn: change
id: merge-respects-cardinality
status: active
created: 2026-11-11
---

# The merge respects a property's cardinality, and matches alarms by content

## Why

Found while reviewing the Pimalaya Android app's stale-write guard (android cairn/changes/write-against-the-current-revision, 2026-11-11), probing `CalendarClient.mergeEvent`, which runs this crate's three-way merge:

- A once-only property kept twice. `merge(base DESCRIPTION:A, local DESCRIPTION:B, remote DESCRIPTION:A + DESCRIPTION:C)` resolves with no conflict and keeps both DESCRIPTION:B and DESCRIPTION:C. The remote's second line is diffed as an addition, and an addition never collides with the local change of the first line (`is_addition` differs, src/tree/merge/op.rs around 67, judged in judge.rs). RFC 5545 allows DESCRIPTION, SUMMARY, LOCATION, DTSTART, DTEND, DUE, STATUS and the like at most once per component (3.6.1, 3.6.2), so the result is invalid and silently keeps both values, which the merge rules forbid (a same-property collision is a conflict, never both values kept). Reached only when a server body already carries such a property twice.
- VALARMs without a UID matched by position. When each side drops a different alarm, the merge resolves silently and brings back the alarm the other side deleted; dropping the first alarm on one side while editing the second on the other gives both plus a conflict. Nothing is lost, but both outcomes misread the edits. The app now gives new alarms a UID (RFC 9074), but alarms from other clients, Google and older saves have none.

## What (to decide)

- The merge knows each property's cardinality per component (RFC 5545 and 7986): for an at-most-once property, every line on either side is the same instance, so two different values are a conflict, never an addition beside a change.
- VALARMs without a UID are matched by content (ACTION and TRIGGER, then DESCRIPTION) before position, the position only breaking ties.
- Tests failing first: the DESCRIPTION case above; each side dropping a different alarm; dropping one alarm while editing another.
