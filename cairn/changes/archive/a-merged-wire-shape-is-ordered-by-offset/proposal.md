---
cairn: change
id: a-merged-wire-shape-is-ordered-by-offset
status: landed
created: 2026-08-29
---

# A merged wire shape is ordered by offset

## Why

`IcalWire::prepend` concatenates the tokeniser's pieces before the line splitter's on the stated assumption that the two lists are already in wire order. They are not. A `QUOTED-PRINTABLE` value ending on two `=` gives the tokeniser a soft break recorded past the last logical byte and the splitter a dangling `=` recorded before it, so the concatenation emits the soft break first.

The line then serializes as `x=\r\n=` where the input read `x==`, and the reparse of those bytes joins the following line into the value. A round-trip that is meant to be byte-faithful loses content.

vcard-rs carries the same wire shape and hit the case through a checked-in fuzz regression, and fixed it with a stable sort by offset.

## What

Sort the merged part list by offset, stably, so a piece the tokeniser and the splitter both recorded at one offset keeps the tokeniser's first. Replace the comment asserting the false invariant with the reason the sort is there, and pin the case with a unit test.
