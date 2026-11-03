---
cairn: change
id: the-parts-written-in-place
status: landed
created: 2026-11-03
---

# The parts written in place

The second speed pass over the parser, the decoded model and the serializers, taken from the handoff [the-structure-scanned-once](../../the-structure-scanned-once/proposal.md), which stays active. It ports vcard-rs's third pass (vcard cairn/log/2026-11-03-the-parts-written-in-place.md) and the handoff's levers 1, 3, 6 and 7. No behaviour change, no API change.

## Why

After the first pass a parsed line was still built in parts and moved, and the profile showed the cost: 62% of the card's `parse` in `IcalLine::take_into` and `IcalLine::build`, mostly one 16-byte load waiting on the stores of the wire shape just built, and 47.5% of decoding alone in `unescape_with`, 98% of it on the load of `str::from_utf8`'s result. The same wait sat everywhere a multi-word value was built aside and moved: a nested component returned and boxed, a decoded property returned and pushed, a parameter decoded and pushed, a spec returned by value to read one of its pointers.

## What

Write every part where it stays, each change kept only for its cycles:

- The ASCII proof (the user's yes on 2026-11-03, the crate's first `unsafe`): `<[u8]>::is_ascii()` then `str::from_utf8_unchecked`, falling back to `str::from_utf8`.
- The common line written into its slot field by field, its parameters where they land, the line feed and the head's marks found by inlined block scans; short `to_bytes` pieces by fixed-size moves.
- A component filled in place: its box allocated first, a line starting `BEGIN:` or `END:` taken straight into the envelope, the item list grown field by field.
- Decoding into slots: properties, components and parameters grown by constant empty ones and filled where they lie, a one-text value read inline when it holds no escape, the property spec handed out by reference.
- Encoding the model into slots, a known name borrowed in its canonical spelling rather than allocated.
- The parsed tree written as text through one buffer; a URI's percent escapes and a head's unquoted colon found by block scans.

The first two items landed as 6545f7e before the session stopped on its deadline (2026-11-03, 12:15); the rest landed in a second session the same day, measured against the tree after [the-simd-codecs](../../../log/2026-11-03-the-simd-codecs.md) (583be79).

## Consequence

None outside speed and allocation counts: every requirement in spec/parsing.md and spec/decoded-model.md holds byte for byte, which the differential run checks.
