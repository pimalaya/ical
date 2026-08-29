---
cairn: change
id: one-matching-ladder-for-both-crates
status: landed
created: 2026-08-29
---

# One matching ladder, the same in both crates

## Why

vcard-rs is adopting the property identity this crate landed today, and the two are meant to be one algorithm with two tables rather than two designs. Two things here do not yet say that.

The ladder is written the wrong way round. Equality is consulted before identity, which is not what the shared ladder says and, where the two disagree, not what should win: an identity is what a rung is for, and equality is the rung under it.

The comparison is on raw bytes. `MAILTO:Ada@Example.com` and `mailto:ada@example.com` are one calendar address, and RFC 3986 section 3.1 makes a URI scheme case-insensitive, so a side that rewrote either misses its own attendee and the merge invents a second one.

## What

Consult the identity rung before the equality rung, and record in the code that the rung above them both, an explicit synchronisation identity, is empty for iCalendar and is why the ladder starts at its second.

Compare an identity normalised and write the line back exactly. State that discipline in the spec, where it holds for both crates.
