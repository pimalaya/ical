---
cairn: log
change: the-lines-built-in-place
landed: 2026-11-03
---

# The lines built in place

The first speed pass over the parser, the decoded model and the serializers, with no behaviour change: the twin of vcard-rs's first two passes (vcard cairn/log/2026-11-02-the-lines-built-in-place.md and 2026-11-03-the-card-written-straight.md), each of their changes ported and measured again here. The baseline is HEAD (9d6a2f9). The next pass starts from [the-structure-scanned-once](../changes/the-structure-scanned-once/proposal.md).

## Results

User-space cycles a pass (`perf stat -e cycles:u`, the difference of two pass counts, 4 and 24 over tests/corpus, 20 and 120 over the criterion calendar parsed 300 times; medians of 11 interleaved runs of each binary, both built with `-align-all-functions=6 -align-loops=64`, `taskset -c 12` on a quiet core):

| workload | before | after | delta |
| --- | ---: | ---: | ---: |
| corpus `parse` | 57,766k | 34,257k | -40.7% |
| corpus `parse` + `decode` | 94,339k | 60,926k | -35.4% |
| corpus `parse` + `to_bytes` | 68,421k | 41,665k | -39.1% |
| corpus `to_string` of the decoded calendars | 126,519k | 12,071k | -90.5% |
| corpus `encode` of the decoded calendars | 75,743k | 67,466k | -10.9% |
| corpus `decode` of calendars parsed beforehand | 40,748k | 35,476k | -12.9% |
| corpus `to_bytes` of calendars parsed beforehand | 17,261k | 21,360k | +23.7% |
| card `parse` | 2,145k | 1,097k | -48.9% |
| card `parse` + `decode` | 3,723k | 1,951k | -47.6% |
| card `parse` + `to_bytes` | 2,705k | 1,391k | -48.6% |
| card `to_string` | 5,717k | 479k | -91.6% |
| card `encode` | 3,111k | 2,743k | -11.8% |
| card `decode` of calendars parsed beforehand | 1,283k | 830k | -35.3% |
| card `to_bytes` of calendars parsed beforehand | 575k | 365k | -36.5% |

Instructions a corpus pass: `parse` 201.1M to 127.5M, `parse` + `decode` 320.8M to 203.4M, `parse` + `to_bytes` 236.9M to 153.5M, `to_string` 470.3M to 47.9M.

Criterion, medians of three interleaved runs on the same core (benches/README.md): `IcalCst::parse` 1,607 to 818 ns, parse + `decode` 2,667 to 1,528 ns, `decode` 896 to 601 ns, `encode` 2,509 to 2,154 ns, `to_string` (a row this pass adds to the bench) 4,292 to 326 ns, `to_bytes` 393 to 210 ns, the round trip 2,059 to 1,091 ns; the rivals moved within 4%. calcard now takes 95% longer than parse + `decode`, and the `ical` crate's `PropertyParser` 254% longer than `IcalCst::parse`.

The figures under What landed were taken on core 4, which another session's interleaved criterion runs were found pinned to at the end of the pass, how long before unknown: read them as interleaved pairs only. The tables were taken on core 12 with nothing else on it.

The corpus is tests/corpus: 293 `.ics` files, 3.4 MB, 132,291 physical lines of which 12,281 are fold continuations, a median logical line of 21 bytes; `parse_many` reads 2,881 calendars and 12,997 components out of it, holding 90,019 property lines (a tenth with a wire shape, a sixth with parameters). The card is the 23-line calendar of benches/parse.rs.

The one row that lost is `to_bytes` of calendars parsed long before: the corpus's trees take about 20 MB, `to_bytes` now walks a tree once for its size before writing it, and both walks miss the cache. Parsed and written in one go (`parse` + `to_bytes`, the card) the walk hits the cache and the sizing wins. Writing with no presizing measured `to_bytes` alone -15% against the sized build but card `to_bytes` +48% and `parse` + `to_bytes` +10% (see Tried and dropped).

Allocations (a counting allocator; one pass over the corpus, one calendar):

| step | corpus before | corpus after | card before | card after |
| --- | ---: | ---: | ---: | ---: |
| `parse` | 126,529 (24,902 reallocations) | 89,245 (1,005) | 15 (3) | 11 (0) |
| `decode` | 49,584 (13,282) | 41,795 (11) | 10 (2) | 9 (0) |
| `to_bytes` | 11,974 (18,195) | 2,881 (33) | 1 (7) | 1 (0) |
| `to_string` | 711,073 (215,032) | 6,005 (709) | 137 (48) | 2 (0) |

The bytes a corpus `parse` allocates grew from 33.7 MB to 48.9 MB: a component's item list starts with 16 slots of 168 bytes (2.7 KB), where it grew from 4 by doubling, and the corpus holds many small components (alarms, time zone rules). A parsed tree keeps those spare slots until it is dropped, as a vcard-rs card keeps its spare lines. Shrinking each list to fit its items cost 6 to 8% on every parse row and 8 slots below depth 2 saved 6% of the bytes for 1.7% more cycles; neither was kept.

## What landed, each kept for its cycles

Each figure is the change against the build before it: interleaved medians or minimums of 5 to 11 runs, read on the minimums where another session's load moved the medians.

- Lines filled in place, from vcard-rs's first pass, as one batch: `IcalLine::take_into` fills a constant placeholder line the component's item list grows by (`IcalLine::take` keeps its signature over it), its fields written over without a drop; a folded line with its value colon on its first physical line borrows its head and joins only its value, sized first by one scan of the continuations, each continuation copied once by the offset of the fold after it; the parameters taken in one walk (`IcalParamNode::take`); static line endings; the dangling `=` of a `QUOTED-PRINTABLE` value inserted into the ordered wire shape rather than sorted in; the escaping mode stamped only for a calendar not in the default mode. A nested `BEGIN` line is tokenised once, where the parent read it and the child read it again, and `IcalCst::parse` hands its first line on rather than reading it twice. `to_bytes` lays a wire shape out in place and writes an unsplit value straight from its node (`lay_out_value`), copying a long folded value once, and sizes its output from each line's sealed length. Corpus `parse` -43%, `parse` + `to_bytes` -38%, `parse` + `decode` -26%, card -30% to -49% (5 runs, under another session's load, minimums).
- The item list sized at 16 (8 measured `parse` +9.2%, card +7.1%, `parse` + `to_bytes` +11.2%).
- The kind lookups with no allocation: `IcalPropKind`, `IcalParamKind`, `IcalValueKind` and `IcalComponentKind` gain a `from_name` matching an upper-cased stack copy, which their `FromStr` calls, so an unknown name (every `X-` property, parameter and component) no longer allocates its error, twice for a property; `IcalLine::decode` looks its name up once for the value kind and the property name, and the `VALUE` parameter is found by its name rather than by parsing every parameter's. Decoded values are checked strictly for UTF-8 first, walked lossily only on failure and an owned value kept rather than copied. Decode alone -17% (card) and -6% (corpus), parse and decode -21% and -6%.
- The canonical spelling matched first: copying a name into the stack buffer called `memcpy`, whose stores the upper-casing loop then read back four bytes at a time before they landed (86% of `from_name`'s samples on one load). The name is matched as it is first, and copied and upper-cased only when it holds a lower-case letter. Property lookup: decode alone -18% (card) and -7% (corpus); the three others: -1.5% to -2.5%.
- The decoded lists sized to the items they will hold, and `IcalLine::raw_value_str` (which names every decoded component) strict before lossy: decode alone -5.9%, parse and decode -6.2% and -5.5%.
- The decoded calendar written straight to text, from vcard-rs's second pass: `Display` for `Ical` encoded the calendar into a syntax tree, a leaf allocation per value, and printed every line through a buffer of its own and a lossy walk. It now writes the bytes from the model. Each value type states its wire shape once, in an `emit` writing one URI, one raw value or a run of components into an `IcalValueOut`, which `Codec::encode` points at a node and `Display` at bytes; a parameter's name and values come from one `wire` match `encode` reads too; lines fold at 75 octets by the rule `to_bytes` applies to a line with no recorded shape (`fold_long`). `to_string` -88% (corpus) and -90% (card), `encode` -14% and -17%.
- The value escapes found sixteen bytes at a time, a block's hits gathered into a bit mask (`escape_into`, which `escape_with` now builds on, so the node and the text cannot escape differently), and the parameter escapes tested on bytes rather than `char` patterns: `to_string` a further -23% and -18%.

## Tried and dropped

- `to_bytes` sized component by component, each reserving its own lines as it starts so the second read hits the cache: `to_bytes` alone -1.4% (corpus), card +11.1%, `parse` + `to_bytes` +4.5% and +7.6%.
- `to_bytes` with no presizing, as before the pass: `to_bytes` alone -15.3% (corpus), card +47.6%, `parse` + `to_bytes` +10.1% and +26.4%.
- Item lists of 8: see above. 8 below depth 2 only: `parse` +1.7%, card +1.7%, 6% fewer bytes.
- `shrink_to_fit` on each component's items once read: `parse` +7.2%, card +8.1%, `parse` + `decode` +4.7% and +8.5%.
- Reserved, measured on a scratchpad copy only: vcard-rs's ASCII proof (`str::from_utf8_unchecked` after `<[u8]>::is_ascii()`) in place of `str::from_utf8` for the head and the decoded values, whose result comes back through memory and stalls the load after it (98% of `unescape_with`'s samples). Corpus `parse` -12.5%, `parse` + `decode` -13.0%, card decode alone -10.5%, card `parse` + `decode` -4.0%, card `parse` flat. It would be the crate's first `unsafe` and waits on the user's yes.

## Verification

A differential crate against a copy of the baseline (`ical-old`) over the corpus and random edits of it (a byte removed, replaced or inserted from an alphabet of line breaks, folds, separators, quotes, backslash and caret escapes, `QUOTED-PRINTABLE` soft breaks, envelope lines, versions, non-ASCII bytes and whole structured lines; chunks copied from elsewhere; files over 4 KB cut to a random 4 KB window), through `parse`, `parse_many` and `parse_recovering`, comparing the trees and errors (`Debug`), `to_bytes`, `to_string`, the version, the decoded model, its `to_string`, its `encode` (`Debug` and bytes), every property's and component's `encode` under both escapers, and every cursor read on every line (`raw_value`, `raw_value_str`, component count and components 0 to 2, `decode_bytes`, `text`, `bytes`, `list`, components 0, 1, 2 and 7, `quoted_printable`, `charset`), then all of it again after setting component 2 of every line. From the writer on it also salts every string of each decoded model (separators, a backslash, line breaks, carets, quotes, non-ASCII, `=\r\n` at the front, the back or both ends, unknown names included) and compares its text and encodes under the model's version and the other one. Runs over the pass: 46,880, 70,320, 23,440, 43,950 and, on the final tree, 21,975 edited inputs, each after the corpus, all agreeing. Planted breaks failed it on the first corpus pass: the old copy's parameter splitter ignoring quotes (ical4j/invalid_tmeher.ics) and the new writer leaving a URI unencoded (ical4j/invalid_CalendarDataFile.ics).

The suite under default, all and no features; clippy `--all-targets --all-features -D warnings`; docs with `-Dwarnings` under all features, `rustdoc::bare_urls` allowed; `cargo build --no-default-features` depends on nothing; `cargo fmt`. Three checks fail on the baseline as on this tree, for code this pass does not touch: clippy under no features (unused `year` in src/recur/scale.rs without `rscale`), docs under all features (bare URLs in src/tzdb/windows.rs) and docs under the default features (the `jcal` link in src/lib.rs without `jcal`).

The parse fuzz target ran 601 seconds from a copy of tests/corpus (489,931 runs) and the merge target 614 seconds from the seeds fuzz/README.md describes (22,271 runs), with no crash. The merge target saved two slow units, 640 and 700 KB taking 27 and 39 seconds under the fuzz build: a release build merges them in 1.46 and 0.89 seconds on the baseline and 1.39 and 0.82 seconds on this tree, so the cost is the merge's own on a large calendar, for a change of its own.

Capabilities moved: none (parsing, decoding and serializing give the same bytes and values as before).
