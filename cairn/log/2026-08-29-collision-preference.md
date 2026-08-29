---
cairn: log
change: collision-preference
landed: 2026-08-29
---

# Say which side wins a collision, apart from which side supplies the bytes

`IcalMerge` named its two versions `left` and `right`, and that one naming answered three questions at once. The merged calendar starts as a clone of the left one, so the left side was at once the side whose untouched bytes survive, the side that wins a collision, and the side nothing is refused of; the right side was the side replayed, the side that loses, and the only side `right_speaks_for` could speak for. A caller that wanted two of those on opposite sides had nothing to say.

tCal is that caller. It merges a local edit against a remote body and wants the local edit judged, an attendee may not move a meeting someone else organises (RFC 5546 3.2), which puts local on the right, which today costs it every collision. Its sibling tCard, merging the same shape of divergence with local on the left because vCard has no organiser, keeps the local value. Two tools built as a pair disagreed on which side survives, and neither author chose that.

The coupling between the bytes and the outcome is now broken. `IcalMergeSide` names the two sides and defaults to `Left`; a `prefer: IcalMergeSide` field on `IcalMerge` carries the caller's answer to the policy question, leaving `left` to answer the fidelity one alone. Every field of `IcalMerge` is public and callers build it as a struct literal, so the new field is a breaking addition, and one already released caller (tCal) adopts it when it bumps.

The boundary is the whole of the change. `judge` reads the preference in exactly one place, the branch where a right-side action collides with a left-side one, and only after the removal question is settled: if either side removed, the update still wins whichever side it came from, because keeping data beats losing it silently and that is not the caller's to invert. Where neither side removed, both sides wrote a value into one field, and the preference says whose lands. Nothing else moved. A property one side alone touched is still taken from that side, an untouched line still comes out byte for byte with its folds, list items still merge as a set, and the report shape is untouched: `IcalMergeConflict` still names the right action beside the left one on its `Divergent` reason, whichever way the preference falls.

Authority is deliberately not part of the preference. The check runs before the collision branch and reads only the replayed side, so a refused action stays refused under both preferences: a permission check belongs to the change being applied, not to the baseline it is applied to. A merge where both sides need judging, two attendee copies of one meeting reconciled against the organiser's last-known body, is still out of reach, and the preference only makes it look closer than it is. Refusing a baseline-side action would mean splicing the base's bytes back over a line already in the merged calendar, which is a different and much larger change.

Six behaviours are asserted, all of them real: the right side's value carried under the right preference, the left preference stated out loud giving byte for byte what saying nothing gives, an update beating a removal in both directions under both preferences, an uncontested property untouched by either preference, an untouched folded line surviving both, and a change Ada has no authority over refused under both. A seventh covers the case the whole change exists for: Ada, judged, answers her invitation and still wins the summary of a task nobody organises.

Capabilities moved: merge. "Three-way merge against a stored base" now says its byte statement is about bytes alone, "Organiser authority" now says a refusal does not depend on the preference, and "The winning side is chosen, not implied" is new.
