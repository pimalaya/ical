---
cairn: change
id: collision-preference
status: landed
created: 2026-08-29
---

# One field decides three things, and a caller needs them apart

## Why

`IcalMerge` names its two versions `left` and `right`, and that naming carries more weight than it looks. The merged calendar starts as a clone of the left one and the right side's actions are replayed onto it, so `left` is at once the side whose untouched bytes survive, the side that wins a collision, and the side nothing is refused of. `right` is the side whose actions are replayed, the side that loses a collision, and the only side `right_speaks_for` can speak for: `judge` runs over the right operations alone, the left ones being in the merged bytes before judging starts.

Three properties, one knob. A caller that wants two of them on opposite sides cannot say so.

That caller exists. tCal merges a local edit against a remote body for a person about to decide between them. It wants the local edit judged, because the local edit is the attendee's and an attendee may not move a meeting someone else organises (RFC 5546 3.2). It also wants the local edit to win a collision it cannot show, because a value silently dropped should be the one nobody typed here. Today it can have either, and it chose authority, so local sits on the right and loses. Its sibling tCard merges the same shape of divergence with local on the left, vCard having no organiser and so no reason to move it, and local wins there. Two tools built as a pair, from one plan, disagree on which side survives a collision their projection cannot render, and neither author chose that.

The coupling worth breaking is the one between the bytes and the outcome. Which side supplies the baseline is a fidelity question: it decides whose folds, whose parameter casing and whose property order come out unchanged, and it is answered by whichever side the caller would rather not churn. Which side wins a collision is a policy question about two people disagreeing, and it is answered by what the caller knows about them. They are not the same question and there is no reason one should settle the other.

The coupling between authority and replay is different, and it stays. A permission check belongs on the change being applied, not on the baseline it is applied to, so judging the replayed side is right rather than accidental. What is missing is not a second judge but a way to be judged without thereby losing.

## What

- A collision preference on `IcalMerge`, naming which side's value the merged calendar carries when both sides changed one property to different things. It defaults to the left side, which is what every merge does today.
- It changes the outcome of that case and nothing else. A property only one side touched is still taken from that side, an untouched line still comes out byte for byte, and the report still names both actions whichever way the preference falls.
- The removal-against-update rule does not move: an update beats a removal whichever side it came from, because keeping data beats losing it silently, and that is not a preference the caller gets to invert. The preference decides only where both sides wrote a value.
- Organiser authority stays on the replayed side and keeps its current spelling. Refusing a baseline-side action would mean splicing the base's bytes back over a line already in the merged calendar, which is a different and much larger change, and no caller has asked for it.

## What this does not do

A merge where both sides need judging, two attendee copies of one meeting reconciled against the organiser's last-known body, is still out of reach: only the replayed side is judged. Naming it here because the preference makes it look closer than it is.
