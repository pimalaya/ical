---
cairn: log
change: a-truncating-read-names-its-component
landed: 2026-08-30
---

# A truncating read names the component it truncates at

The value node read one `;`-component through `decode_at`, `decode_scalar_at` and `decode_joined_at`, and almost every caller passed `0`. Component zero looks like the value and is not: the read stops at the first unescaped `;`, and the scalar form stops again at the first unescaped `,`. Four defects in two days across three crates came out of that one shape, each fixed where it was found while the shape that produces them stayed, with fifteen more `..._at(0)` call sites behind it.

The readers now say what they cut. `decode`, `decode_list` and `decode_bytes` read the whole value with its separators literal; `decode_component` and `decode_component_list` read one slot and always spell out which. `decode_scalar_at` and `decode_bytes_at` are gone: both cut twice, and no honest caller wanted the second cut. The writers moved with them, `set` and `set_bytes` replacing the whole value against `set_component` and `set_component_bytes` naming their slot, because a reader and a writer at different scopes turn a read-modify-write into data loss.

Every call site was then reviewed one at a time. Most became the whole-value read, which fixed what they were quietly dropping: a description or a duration or a date cut at a `;` it was supposed to escape, a text list and a date-time list losing everything past one, a `CAL-ADDRESS` and a `PERIOD` likewise, and a `REQUEST-STATUS` description and its extra data cut at a comma that separates nothing. `GEO` and `REQUEST-STATUS` kept their component reads, now written as the deliberate act they are. `RECUR` was walking the components and rejoining them by hand, which is exactly the new whole-value read, so the loop is gone and the borrow it used to lose is kept.

The cursor moved with the node, so `text`, `bytes` and `list` read the value rather than its first slot, and their setters replace it rather than leaving a tail behind. One consequence needed answering: the wire cannot tell an empty list from a list of one empty item, and the whole-value list read answers one empty item where the component read answered none, so the merge's list replay, which reads its own writes, would have written a leading comma into the next addition. It now treats an all-empty list as empty. The cursor's accessor test asserted nothing at all and was made to assert the whole set, matching its vCard twin.

Spec updated: `decoded-model` (MODIFIED: a single value is not split on a separator it does not own), `parsing` (ADDED: a truncating read names its component; MODIFIED: round-trip fidelity, now distinguishing the two setter scopes).
