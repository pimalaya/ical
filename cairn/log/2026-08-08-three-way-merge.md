---
cairn: log
change: three-way-merge
landed: 2026-08-08
---

# Three-way merge

`src/tree/merge.rs`, under the `parser` feature, since a merge that cannot keep bytes is not worth having and bytes live in the syntax tree.

`IcalMerge` holds the three calendars and who the right side speaks for; `merge` returns what each side did relative to the base, the merged calendar, and every collision. The merged calendar starts as a clone of the left one and the right side's actions are replayed onto it line by line, so a line neither side edited comes out of the merge exactly as it went in, folds and all. That is the one property the vCard implementation this was ported from also has, and it is why the merge lives in the syntax layer rather than over the decoded model.

## What the calendar case needed that the card case did not

**A tree, not a list.** A vCard is a flat list of properties; a calendar is components inside components. Every component is addressed by a path of steps from the root, and the diff runs per matched component.

**An identity iCalendar already has.** vcard-rs matches instances by `PID`, the RFC 6350 synchronisation identity. iCalendar has no `PID`; it has `UID` and `RECURRENCE-ID`, which say exactly what is needed: a component is identified by which object it is and which occurrence of it. A component with no `UID` (an alarm, a time-zone observance) falls back to its position among its same-named siblings. The consequence worth stating is that a file whose events were reordered merges as if nothing had moved, which is what a server that sorts differently from a phone will do every time.

**Two conflict rules the card case has no word for.**

A change to a series and a change to one of its instances both survive, and are reported as a pair. Neither is wrong; but a rule that moved may have moved the ground the override stood on, and only the caller knows whether that matters.

An attendee may not rewrite what the organiser owns (RFC 5546 3.2). That needs to know who is editing, which no amount of diffing can infer, so `right_speaks_for` is where the caller says it. Set, a right-side change to an organiser-owned property of a meeting someone else organises is refused and reported; unset, no claim is made and nothing is refused on that ground. An attendee's own `ATTENDEE` line, their transparency, their alarms and anything outside the vocabulary stay theirs either way.

## Two bugs the tests found

**A removal against an update was resolved backwards.** The rule is that the update survives, because keeping data beats losing it silently. The first implementation only applied it when the *right* side was the removal; when the left side removed and the right side updated, the removal won and the update was dropped. Both directions are now the same rule, the update is replayed onto a line the left side had deleted, and the collision is reported either way.

**A single text value was being truncated at its first comma.** `SUMMARY:Standup, moved` decoded to `Standup`. RFC 5545 3.3.11 says that comma should have been escaped, but there is no list for it to separate, so the strict reading loses data that the byte-faithful tree was carrying perfectly well. `IcalText` now joins the components, as URIs already did. The merge found this because it compared two lines by their *first* value and concluded a changed summary had not changed; comparing decoded values rather than raw first-value bytes is the other half of that fix.

## What is reported

Component added or removed, property added or removed, value changed, one list item added or removed, parameter added, removed or changed. List items merge as a set, so two sides editing one list never collide. Collisions carry the left action they collided with, so a caller resolving differently has both sides in hand.

`cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features` (13 binaries), `cargo build --no-default-features` and `cargo deny check` are green.

Capabilities moved: `merge` (ADDED: three-way merge against a stored base, instance identity, a series and its instances, organiser authority); `decoded-model` (ADDED: a single text value is not split on its commas).
