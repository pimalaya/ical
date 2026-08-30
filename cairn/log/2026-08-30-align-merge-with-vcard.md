---
cairn: log
change: align-merge-with-vcard
date: 2026-08-30
---

# Aligned the merge with vcard-rs

The two crates are twins that state one merge contract and share almost no implementation, and this side had drifted. An audit claimed five defects here and one in vcard-rs; a second pass told to refute them confirmed all six, and two root causes were then read in the source directly. Every one of the five is a rule vcard-rs already has, which is what made this an alignment rather than a design.

What moved, in the order the failures bite:

A value was compared decoded, and a text value decodes its first `;`-component alone, so `LOCATION:Room A;floor 2` edited to `floor 9` reported nothing and merged nothing. Values now compare on the raw nodes, component by component, falling back to bytes where the two sides escape by different rules.

A list was a set on both sides of the operation: the diff asked only whether an item was still a member, and the replay dropped every item equal to the one removed. `CATEGORIES:a,a,b` losing one `a` was therefore invisible going in and took both coming out. Both halves are now multisets, matching one for one.

A replay target was corrected for the baseline side's removals and not for its additions, although the merged calendar is that side's own tree. A line it inserted pushed everything after it down, so an edit meant for the second property landed on the first: one overwritten, one left stale, nothing reported. The translation now walks removals then additions.

A property the baseline side removed and the other side edited twice came back twice, the restore having no ledger. It now comes back once, the restored line being the other side's own bytes and already carrying every action.

A `VALUE` retyped on one side did not meet the other side's item edits, so a property could end up declaring `PERIOD` while carrying a bare `DATE-TIME` item, which RFC 5545 section 3.8.5.2 forbids. Retyping now contests the value, as a whole-value change now contests item edits.

Organiser authority went the other way. `right_speaks_for` named a side rather than a role, so the one caller using it had to put its local calendar on the merge's right and then ask for the right side to be preferred, while every other caller puts local on the left. The field, the `Authority` reason and the organiser predicates are gone, and with them the RFC 5546 section 3.2 refusal. tcal moved its local calendar to the left and dropped `merge --speaks-for` in the same breath. The capability is worth having back behind a field that names its own side, which would let it return without moving anybody's sides again.

Capabilities moved: merge.
