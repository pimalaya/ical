---
cairn: change
id: an-edited-line-keeps-its-folds
status: draft
created: 2026-10-18
---

# An edited line keeps the folds before its edit

## Why

Most of fold-aware targeted patching is already here: an edit rewrites only its own logical line and every other byte of the calendar survives, folds, blank lines, quoted parameters, `X-` properties and `VALARM`s included (parsing: round-trip fidelity; merge: the left side's untouched bytes), and an edited line that grew past 75 octets is folded again (parsing: an encoded line is folded, rfc5545-contract-fixes, 2026-10-09).

What is left is the edited line itself. A length change drops its whole recorded shape (`IcalWire`'s length guard), so a long `DESCRIPTION` folded by its sender at 74 octets with a tab, edited in its last word, goes out refolded from its first byte at 75 octets with a space. Every physical line of it changes, a diff of the calendar shows the whole property rewritten, and a server comparing bytes sees more change than there was.

An external engine patches the target logical line fold-aware and leaves every other byte untouched; this crate does that for lines, and this change does it inside the line.

vcard-rs has the same design one step behind (it writes an edited line unfolded): vcard cairn/changes/an-edited-line-is-folded ports the fold and this rule together, so both crates fold an edited line alike.

## What

- An edit of one value records where the logical bytes first differ. The recorded folds before that offset stay where they were; the line is refolded from the last kept fold onward.
- The refold uses the line's own folding whitespace (a tab where its first fold had one) and its own line ending, at the width of its recorded folds when they were all at one width and below 75 octets, else at 75, never inside a UTF-8 sequence.
- The length guard stays the safety net: a shape that no longer fits is never applied blindly, it is cut at the first changed byte.
- A `QUOTED-PRINTABLE` line keeps today's rule: never folded.

## Open questions

- Whether "the width of its recorded folds" is worth inferring, or 75 always.
- Whether a component writer (`set_component`) reports the first changed offset itself, or the line compares old and new bytes.
