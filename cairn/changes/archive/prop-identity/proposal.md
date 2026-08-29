---
cairn: change
id: prop-identity
status: landed
created: 2026-08-29
---

# Address a property by what it names, not by where it sits

## Why

`IcalPropPath::index` is documented as "the position among the component's properties of that name". Every action the diff produces counts that position in the base. The replay then resolves it against the merged calendar, which is a clone of the left side, and reads the right side's new bytes back out of the right calendar at the same position. Three calendars, one number, three different meanings.

The left side is free to have taken a same-named property out before that position, and the right side is free to have written its own list in a different order. When either happens the number names a different line in each calendar, and the replay lands on somebody else. Merging a base of `ATTENDEE Ada, ATTENDEE Zoe` with an untouched left and a right of `ATTENDEE Zoe, ATTENDEE Bob` produces `ATTENDEE;CN=Bob:mailto:zoe@example.com` next to a duplicated Zoe, with no conflict reported: a person who exists in no version of the calendar. An attendee answering an invitation has their `PARTSTAT` written onto a different attendee. Neither depends on the preference.

The pairing that feeds the replay is wrong in the same way. `diff_component` pairs same-named properties by exact equality and then by position, so a base attendee nobody kept is paired with a side attendee nobody had, and the diff reports a rename where a person left and another arrived.

## What

A property is addressed by an identity where iCalendar gives it one, and by its position only where it does not.

**The identity rule.** A property that may occur more than once in a component and whose value names a thing outside the calendar is identified by that value. RFC 5545 gives four such properties, and RFC 7986 two more: `ATTENDEE` by its calendar user address (3.8.4.1), `ATTACH` by its URI or its inline binary (3.8.1.1), `RELATED-TO` by the `UID` it points at (3.8.4.5), `REQUEST-STATUS` excluded because its value is the datum rather than a reference, `CONFERENCE` and `IMAGE` by their URI (RFC 7986 5.11, 5.10). Every other property has no identity: either it may occur only once, so its name already identifies it and position zero is not a guess, or its value is the thing being edited, so keying on it would turn every edit into a replacement.

The identity is the raw value as written, compared exactly. A calendar address or a URI carries no escaping to normalise, and folding a `mailto:` address case-insensitively would merge two mailboxes RFC 5322 keeps apart.

The identity travels on the path, so it reaches the report: `IcalPropPath` gains `identity`, `None` where the property has none. A caller reading a conflict is told which attendee is contested rather than which slot of a list.

**What the identity is used for.** The diff pairs a base property with a side property by exact equality, then by identity, and only then by position, and a positional pairing is refused where the two carry identities that differ: a different calendar address is a different person, never a renamed one. The replay resolves its target by identity where the path carries one, so an action about Ada never lands on Bob, and reads its source line out of the side that produced it by that side's own address rather than by the base's.

**Where no identity exists.** Position remains, and is made to mean what it says. A replayed action carries both the position its target held in the base and the position the source line holds in the side that wrote it. The first is translated through the left side's own removals before it is resolved against the merged calendar, since taking a member out of a group renumbers the ones after it, and the second is used as written against the side it was measured in. A base position the left side removed resolves to nothing, which is the removal-against-an-update case the replay already restores.

Component matching is untouched: a `UID` with a `RECURRENCE-ID` after it is already an identity, and a component carrying neither is still matched by its position among its same-named siblings.

## Judgement calls the owner should review

Changing the calendar address of an `ATTENDEE` is now reported as a removal and an addition rather than as a value change, and the merged calendar carries both people where two sides changed one address differently. That follows from the rule rather than from convenience: the address is the person, and two sides that invited two different people have not disagreed about one field.

The identity is the *raw* value rather than the decoded one, and is compared byte for byte.
