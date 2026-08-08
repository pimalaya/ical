---
cairn: change
id: fold-preservation
status: landed
created: 2026-08-08
---

# Make the byte-faithfulness claim true

## Why

The parser unfolds continuation lines, drops blank lines and resolves QUOTED-PRINTABLE soft breaks, and restores none of them. Every real `.ics` folds at 75 octets, so a majority of real files do not come back byte-identical. The README promises "down to the exact bytes and the line endings", and the corpus test avoids catching it by using only pre-unfolded fixtures. Silence is not an option: either the claim becomes true, or it becomes precise.

## What

Take the first way out, which is the better product: record the wire shape on the line and re-emit it, so a parsed calendar serializes back to its exact input bytes, folds and blank lines included. The cost is one small record per line. The logical content the rest of the crate sees is unchanged, and an edited value re-folds rather than pretending the old fold points still apply.

vcard-rs shares the design and the same file, so the port there is a follow-up in that repository, not part of this change.

Done when the README, the src/lib.rs header and the corpus test all say the same thing, and the corpus test no longer relies on hand-unfolded fixtures.
