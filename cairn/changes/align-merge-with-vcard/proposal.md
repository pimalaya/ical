---
cairn: change
id: align-merge-with-vcard
status: landed
created: 2026-08-30
---

# Align the merge with vcard-rs

## Why

The two crates are deliberate twins and state one merge contract, but they share almost no implementation, and this side had drifted into five defects a green suite was hiding. Every one of them is a rule vcard-rs already implements correctly, which is what makes this an alignment rather than a redesign.

Separately, `right_speaks_for` named a side rather than a role. The one caller that used it therefore had to put its local calendar on the merge's right and ask for the right side to be preferred, while every other caller in the ecosystem puts local on the left. One field forced one convention apart from the rest.

## What

Port vcard-rs's rules for value comparison, list diffing, replay addressing and collision granularity, and remove the authority apparatus so a single side convention holds everywhere.

Done when a value differing past its first `;` is seen, a list behaves as a multiset on both the diff and the replay, a replay target survives an insertion on the baseline side, a restored property comes back once however many actions it carries, a retyped `VALUE` contests the other side's items, and `IcalMerge` carries only base, left, right and prefer.

## Consequence

Organiser authority is gone: an attendee's change to a property the organiser owns is no longer refused. The capability is worth having back, and the way back is a field naming its own side rather than a fixed one, which would let it return without moving anybody's sides again.
