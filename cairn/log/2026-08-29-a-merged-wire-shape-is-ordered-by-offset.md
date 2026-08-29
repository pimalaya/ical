---
cairn: log
change: a-merged-wire-shape-is-ordered-by-offset
landed: 2026-08-29
---

# A merged wire shape is ordered by offset

`IcalWire::prepend` put the tokeniser's pieces in front of the line splitter's and said in a comment that the two lists concatenate already ordered. They do not, and the comment is what hid it.

The tokeniser resolves a `QUOTED-PRINTABLE` soft break while it assembles the logical line, and records it at the offset it reached. The line splitter then drops a dangling `=` off the assembled value and records that at the offset the value ends on. For a value ending on two `=`, those are the same two bytes seen from opposite ends: the tokeniser's soft break sits one byte past the last logical byte and the splitter's `=` sits on it. Concatenating emits the soft break first, so a line whose input read `x==` serialized as `x=\r\n=`, and the reparse of those bytes read the `=` as a fresh soft-break marker and joined the following line into the value. `END:VCALENDAR` disappeared into a `NOTE`.

The fix is the one vcard-rs already carries: append, then a stable sort by offset. Stability is what settles the tie, since a piece the tokeniser and the splitter both recorded at one offset belongs to the tokeniser first. The comment now states why the sort is there rather than an invariant that never held.

Two tests pin it, and both fail without the sort. `orders_a_merged_shape_by_offset_rather_than_by_list` states the mechanism on a bare shape. `round_trips_a_quoted_printable_value_ending_on_two_equals` states the consequence on the input itself, a calendar that must serialize to its own bytes and reparse to them again. The second is vcard-rs's checked-in fuzz regression, transposed to a calendar: the same shape, so the same defect, found there first only because vcard-rs replays that corpus.

Capabilities moved: parsing ("Line normalisation" MODIFIED, gaining the ordering rule and its scenario).
