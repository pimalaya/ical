---
cairn: change
id: the-lines-built-in-place
status: landed
created: 2026-11-03
---

# The lines built in place

The first speed pass over the parser, the decoded model and the serializers, the twin of vcard-rs's first two passes (vcard cairn/log/2026-11-02-the-lines-built-in-place.md and 2026-11-03-the-card-written-straight.md), with no behaviour change.

## Why

The tree layer is still the shape vcard-rs had before its passes, and profiles the same way. Over tests/corpus (293 files, 3.4 MB, 120k logical lines, 2,881 calendars through `parse_many`) a strict `parse` pass takes 58M user-space cycles, about 485 a line, where vcard-rs now spends about 330 on lines carrying folded photos. In `IcalLine::parse`, 57% of the samples sit on one 16-byte stack load (a store-forwarding wait on a moved multi-word value), memmove takes 17.7% of the pass and the allocator about 17%.

Decoding allocates a `String` for every unknown name it looks up (`X-` properties, parameters and components, each name parsed twice), validates every value through `String::from_utf8_lossy` and copies owned values once more; `to_bytes` copies a folded line twice through a fresh buffer; `to_string` of a decoded calendar encodes it into a tree first and converts every line lossily on its own.

## What

Port what vcard-rs measured, each change kept only for its cycles:

- Lines filled in place: a component grows its item list by a constant placeholder line and the tokeniser fills it (`IcalLine::take_into`), the line's fields written over without a drop; a nested `BEGIN` line is tokenised once rather than once by the parent and again by the child.
- A folded line built once: with the value colon on its first physical line, the head borrows the input and only the value is joined, sized first, into the buffer the value node keeps; the folds read once.
- The parameters in one walk, static line endings, the escaping mode stamped only when it is not the default.
- Decoding: kind lookups by exact match on an upper-cased stack copy with no allocation, each name looked up once; UTF-8 validated strictly before any lossy walk; the components walked once.
- `to_bytes` sized up front and a folded line laid out in place.
- Whatever else a profile names on the way, and the decoded calendar's `to_string` written without the intermediate tree if the pass gets there.

## Rules

No new dependency, no `unsafe` (the crate has none; vcard-rs's ASCII proof needed the user's yes and so would this one), no API break. Every parser or model change goes through a differential run against a copy of the baseline (scratchpad crate `ical-old`): the corpus and random edits of it through `parse`, `parse_many` and `parse_recovering`, comparing trees and errors, bytes, text, the decoded model, its encodes and every cursor read, before and after an edit of component 2 on every line. Cycles are counted by `perf stat` on an aligned build, interleaved medians and minimums, as vcard cairn/changes/the-structure-scanned-once/proposal.md describes.

## Consequence

None outside speed and allocation counts: every requirement in spec/parsing.md and spec/decoded-model.md holds byte for byte.
