---
cairn: log
change: fold-preservation
landed: 2026-08-08
---

# Make the byte-faithfulness claim true

The README promised a round-trip "down to the exact bytes and the line endings" while the parser was quietly unfolding continuation lines, dropping blank lines and resolving QUOTED-PRINTABLE soft breaks without restoring any of them. Of the 190 real-world fixtures, 72 came back byte-identical. They all do now, and the claim is no longer a paraphrase.

The mechanism is one new type, `tree::wire::IcalWire`: a list of byte offsets into a line's *logical* bytes (its name, its parameters and its value, exactly as the serializer lays them out) with the piece of wire that sat at each offset. Three pieces cover everything the tokeniser resolves: a fold (the break and the single whitespace that marked the continuation, kept apart so `\n\t` does not come back as `\r\n `), a soft break, and a run of bytes taken verbatim (the blank lines before a line, the whitespace of a dangling continuation, a trailing `=` with nothing to continue). Re-inserting them in order reproduces the input.

The offsets are checked, not trusted. The logical length is sealed alongside them, and a shape whose length no longer matches the line is dropped rather than applied: an edit that changes a value's length moves every byte after it, so the old fold points would land in the wrong places. An edited line therefore goes out unfolded, which RFC 5545 3.1 permits (it recommends 75 octets, it does not require them). An edit that keeps the length keeps the shape, since every offset still indexes what it did. The guard is on the length rather than on a mutation flag deliberately: the fields of a line are public, so a mutation flag would be bypassed by a direct write, and a length check cannot be.

Two things sat above the line and needed the same treatment. Blank lines *before* the first content line were being eaten by a `trim_leading_eol` at the calendar level, which is now gone, since the tokeniser records what it skips. Blank lines *after* the last one had nowhere to live, so `IcalCst` gained a `trailing` field, set only when nothing but whitespace follows the calendar. That also fixed the multi-calendar case: `parse_many` no longer trims between calendars, so concatenating what it yields reproduces the whole file. `parse` still reads the first calendar and stops, which is what it is for, and its documentation now says so plainly.

`IcalItem::Component` is boxed now. The extra field pushed the size difference between the two variants of that enum to 208 bytes, over clippy's 200-byte threshold. Boxing the recursive variant is the fix that stays fixed, rather than shaving bytes off the shape until the lint goes quiet.

The corpus tells the story: normalised went to zero in all three vendored directories (libical 29 identical, ical4j 102, ical.js 46 out of 46). One libical fixture moved from parsed to refused, which is not a regression but an honesty gain: the sweep now reads the whole file rather than its first calendar, so a file whose *second* calendar is malformed is refused rather than silently half-read.

The README, the `src/lib.rs` header and the corpus test now say the same thing. vcard-rs shares this design and the same file; the port there is a follow-up in that repository.

Capabilities moved: `parsing` (MODIFIED: round-trip fidelity, line normalisation).
