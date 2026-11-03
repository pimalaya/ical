---
cairn: log
change: the-parts-written-in-place
landed: 2026-11-03
---

# The parts written in place

The second speed pass over the parser, the decoded model and the serializers, with no behaviour change and no API change, taken from the handoff [the-structure-scanned-once](../changes/the-structure-scanned-once/proposal.md): vcard-rs's third pass ported and the handoff's levers 1, 3, 6 and 7. It ran in two sessions on 2026-11-03. The first, 11:25 to 12:15, landed the ASCII proof, the common line written into its slot and the short `to_bytes` pieces, committed as 6545f7e; the second, from 17:20, after [the-simd-codecs](2026-11-03-the-simd-codecs.md) (583be79), landed the rest and the measurements below. The next pass starts from the rewritten handoff.

## Results

User-space cycles a pass (`perf stat -e cycles:u`, the difference of two pass counts, 4 and 24 over tests/corpus, 20 and 120 over the criterion calendar parsed 300 times), each binary built from one directory with `-align-all-functions=6 -align-loops=64` and linked in four layouts (natural order and lld `--shuffle-sections` seeds 1 to 3), three copies of each on ext4, runs interleaved and rotated over the copies on core 12 with nothing on it or its sibling. Before is the tree before the pass (89512ea) and after the working tree, layout 0 medians of 7 runs; the delta is the mean of the four layouts' median deltas, each layout's within 2.3 points of it on every row that parses first.

| workload | before | after | delta | first session | second session |
| --- | ---: | ---: | ---: | ---: | ---: |
| corpus `parse` | 34,409k | 24,224k | -29.4% | -26.6% | -4.1% |
| corpus `parse` + `decode` | 61,085k | 43,676k | -28.6% | -21.6% | -8.8% |
| corpus `parse` + `to_bytes` | 41,437k | 30,456k | -26.9% | -24.3% | -3.1% |
| corpus `decode` of trees parsed beforehand | 36,041k | 29,193k | -16.9% | -8.8% | -8.6% |
| corpus `to_bytes` of trees parsed beforehand | 21,362k | 19,803k | -10.8% | -3.7% | -4.9% |
| corpus `to_string` of the decoded calendars | 12,088k | 11,387k | -6.7% | -1.6% | -5.3% |
| corpus `encode` of the decoded calendars | 67,430k | 54,214k | -20.1% | +0.2% | -20.5% |
| corpus `to_string` of the parsed trees | 48,683k | 25,169k | -49.0% | +5.7% | -51.5% |
| card `parse` | 1,105k | 778k | -28.8% | -23.4% | -5.6% |
| card `parse` + `decode` | 1,969k | 1,503k | -23.5% | -17.4% | -7.2% |
| card `parse` + `to_bytes` | 1,397k | 1,024k | -26.0% | -21.9% | -5.1% |
| card `decode` | 829k | 633k | -25.3% | -11.6% | -14.4% |
| card `to_bytes` | 374k | 353k | -7.7% | -7.0% | -1.5% |
| card `to_string` of the decoded calendar | 473k | 457k | -3.7% | +0.0% | -3.9% |
| card `encode` | 2,743k | 2,060k | -24.6% | -1.4% | -23.2% |
| card `to_string` of the parsed tree | 2,054k | 737k | -63.6% | +10.0% | -67.2% |

The first session's column is 583be79 against 89512ea in the same runs (583be79 also carries the SIMD codecs, which no row exercises); the second session's comes from a run of its own at 11 runs, 583be79 against the working tree. The first session made `to_string` of a parsed tree slower, `put` growing a fresh buffer for every line; the second writes the tree through one buffer.

Instructions a corpus pass: `parse` 127.5M to 98.9M, `parse` + `decode` 203.4M to 161.7M, `parse` + `to_bytes` 153.6M to 124.9M, `to_string` of the decoded calendars 47.9M to 43.5M, `encode` 250.1M to 196.8M, `to_string` of the parsed trees 163.7M to 49.4M.

Allocations of the second session (a counting allocator; one pass over the corpus, one calendar): `to_string` of the parsed trees 118,897 allocations and 205,167 reallocations to 5,765 and 22,554 over the corpus (12.0 MB to 5.1 MB), 24 and 43 to 2 and 9 for the card; `encode` 499,950 to 373,942 (97 to 72); `parse` 89,246 to 89,254 with 1,015 to 802 reallocations, an `END` line no longer taking a slot of its component's list; `decode` (41,796), `to_bytes` (2,881) and the decoded calendar's `to_string` (6,005) unchanged.

Criterion, medians of three interleaved runs on core 12, the bench executables on ext4 (benches/README.md), 583be79 against the working tree: `IcalCst::parse` 620 to 539 ns, parse + `decode` 1,223 to 1,037 ns, `decode` 524 to 422 ns, `encode` 2,133 to 1,565 ns, `to_string` of the decoded calendar 345 to 313 ns, `to_bytes` 184 to 181 ns, the round trip 796 to 741 ns; the rivals moved within 3%. calcard now takes 183% longer than parse + `decode`, and the `ical` crate's `PropertyParser` 406% longer than `IcalCst::parse`.

## What landed, each kept for its cycles

The first session, against the build before each step, layout 0 on tmpfs (see the proposal's measuring notes; the four-layout column above confirms them on ext4):

- The ASCII proof (approved by the user on 2026-11-03, the crate's first `unsafe`): `tree::codec::unescape::text` proves ASCII with `<[u8]>::is_ascii()` and calls `str::from_utf8_unchecked`, falling back to `str::from_utf8`, whose result comes back through memory and stalled the load after it. Corpus `parse` -11.7%, `parse` + `decode` -13.0%, card `decode` -12.7%.
- The common line written into its slot field by field, its parameters where they land, the line feed and the head's marks found by inlined 16-byte block scans: corpus `parse` -16.2%, card `parse` -22.9%.
- Short `to_bytes` pieces by fixed-size moves (`tree::leaf::put`), settled this session over four layouts: replacing it by `extend_from_slice` gives back 2.8 points of corpus `parse` + `to_bytes`, 3.9 of the card's and 7.1 of card `to_bytes`.

The second session, against the build before each step, layout 0, interleaved medians of 9 runs:

- A component filled in place: `IcalCst::take_body` fills the component it is given; a line starting `BEGIN:` or `END:` in any case is named so whatever follows, so it is taken straight into a new child's box or this component's envelope; the box is allocated before its body, from a constant placeholder; any other `BEGIN` or `END` (a blank line before it, a parameter, a fold) still goes through a slot and is moved out. The item list grows by `resize_with`, which stores the placeholder's fields where a pushed constant over 128 bytes is copied by a `memcpy` call (the constant `push` gives back about one point on every parse row over four layouts). Corpus `parse` -2.5%, card `parse` -3.9%, card `parse` + `to_bytes` -6.0%.
- Decoding into slots: a calendar's and a component's property and component lists grow by constant empty ones (`IcalProp::empty`, `IcalComponent::empty`) filled where they lie (`IcalLine::decode_into`, `IcalCst::decode_into`), each value kind storing its own value, a kind holding one text read inline when its value is unsplit, UTF-8 and free of backslashes (`IcalValueNode::plain`), the rest out of line. Corpus `parse` + `decode` -4.4%, card `decode` -6.8%. Then each parameter decoded into a slot of its list, each kind's arm storing its own parameter, and `prop_spec` handing out a reference to a constant spec rather than six fields through memory to read one pointer: card `decode` a further -4.8%. Then `plain`'s backslash found by the inlined block scan rather than memchr's call: card `decode` a further -4.2%, corpus `parse` + `decode` -2.4%.
- Encoding into slots: a known property, parameter or component name is borrowed in its canonical spelling (`wire_name`, the table each kind's `Deref` reads) rather than allocated, and the version written from its `wire_name`: `encode` -18.0% (corpus) and -21.0% (card). Each property's line encoded into a slot of the item list and each nested component into its box: a further -1.6% and -1.8%.
- Block scans for a URI's percent escapes (`percent_into`) and a head's unquoted colon (`unquoted_colon`, which `fold_long`'s check for `QUOTED-PRINTABLE` and the folded-line paths run): `to_string` of the decoded calendars -4.5% (corpus) and -3.7% (card), corpus `parse` -1.1%.
- The parsed tree written as text through one buffer: `Display` for `IcalCst` lays each line out in a buffer it reuses and writes it to the formatter, still lossily line by line as `Display` for `IcalLine` does; it allocated a buffer a line and went through `write!`. Against 583be79, the harness gaining the row then: `to_string` of the parsed trees -51.7% (corpus) and -67.4% (card).

## Tried and dropped

- The kind lookup by length, first byte and the byte before last, which tell the 71 property names apart, confirmed by `eq_ignore_ascii_case` (vcard-rs's lever 3): card `decode` +4.2% where the build before was -14.2%, instructions +22%. The canonical spelling matched by fixed-size compares first, as today, costs less than a case-insensitive walk of the candidate.
- `charset` keeping its UTF-8 buffer through `String::from_utf8` rather than encoding_rs's borrow and copy: 4.0k to 7.0k cycles on 4.4 KB of accented text, core's validator walking non-ASCII byte by byte.
- `IcalParamNode::decode` forced inline into the parameter list's `extend`: flat, the parameter still assembled on the stack and copied. The per-arm slots that replaced it removed that copy but stayed flat until `prop_spec` stopped returning its spec through memory: the wait had moved there.
- The placeholder line pushed without `const`: LLVM hoists it to the stack and still copies it by `memcpy`.

## The content codecs, measured

The question this session started from was whether the move to the SIMD codecs was done properly. The calls match vcard-rs's (`QuotedPrintableCodec::default().decode_vec`, `Base64Codec::default().decode_vec`, the token read when the codec is built); tests/codecs.rs checks the detected token decodes the corpus's values as the baseline does; the crate builds for thumbv6m-none-eabi and riscv32imc-unknown-none-elf with `parser,base64,quoted-printable`, which the-simd-codecs could not check (a throwaway shell carried both targets' standard library). Cycles a pass of harness/codec, medians of 7 interleaved runs on core 12, before the switch (6545f7e: base64 0.23, quoted_printable 0.5), after it with no token installed and after it with `SimdCpu::install_detected()`:

| decoding | before | no token | detected token |
| --- | ---: | ---: | ---: |
| a folded inline `ATTACH` of 36 KiB, `decode_base64` | 47.3k | 41.1k (-13%) | 2.0k (-96%) |
| a soft-broken 1.0 `QUOTED-PRINTABLE` description of 4.4 KB, `quoted_printable` | 29.9k | 7.1k (-76%) | 3.2k (-89%) |
| the same through `charset` (UTF-8) | 31.1k | 8.1k (-74%) | 4.2k (-87%) |
| 300 inline payloads of 48 bytes | 43.5k | 34.9k (-20%) | 22.7k (-48%) |
| 300 one-paragraph `QUOTED-PRINTABLE` values | 460.7k | 104.1k (-77%) | 65.1k (-86%) |

Both decoders drop the blanks ending a line or the value, as RFC 2045 6.7 asks, before and after the switch: `quoted_printable`'s documentation now says so rather than that decoding loses no byte.

## Verification

The differential crate (harness/diff) against a copy of 583be79 (`ical-old`), over the corpus and random edits of it, comparing trees, bytes, text, decoded models, encodes and every cursor read before and after an edit, as the handoff describes: during the session the corpus and 11,720 edited inputs on four seeds after the parse and decode changes; on the final tree the corpus and 51,275 edited inputs on seven seeds, all agreeing. Planted breaks failed it on the first corpus pass: the old copy decoding `DURATION` as text (ical4j/invalid_CalendarDataFile.ics) and the new `plain` reading a value holding a backslash as plain (ical4j/hcalendar_example1.ics). The first session ran it on its own steps (the corpus and 46,880 edited inputs).

The suite under default, all and no features; clippy `--all-targets --all-features -D warnings`; docs with `-Dwarnings` under all features, `rustdoc::bare_urls` allowed; `cargo build --no-default-features` depends on nothing; thumbv6m-none-eabi and riscv32imc-unknown-none-elf build with `parser,base64,quoted-printable`; `cargo fmt`. Two checks fail on the baseline as on this tree, for code this pass does not touch: clippy under no features (unused `year` in src/recur/scale.rs without `rscale`) and docs on the bare URLs of src/tzdb/windows.rs.

Miri (the nightly of ~/code/pimalaya/quoted-printable-codec/fuzz/shell.nix) over the unit tests of `tree::`, `prop::`, `param::`, `component::`, `value::`, `version::` and `validator::`, the parser, decoder and encoder paths the ASCII proof's `unsafe` and this pass touch: 142 passed, no undefined behaviour. The first session ran the 102 `tree::` tests. A run over every unit test stalled on `recur::civil::tests::day_counts_round_trip`, a calendar-arithmetic loop hours long under Miri that reaches no `unsafe`.

The parse fuzz target ran 601 seconds from its saved corpus (374,720 runs, coverage 6,222 to 6,436) and the merge target 601 seconds (21,990 runs, coverage 6,254 to 6,636), with no crash. The merge target saved one slow unit, three calendars of 621 KB: a release build merges it in 0.87 to 0.90 seconds on 583be79 and 0.87 to 0.92 on this tree, the merge's own cost on a large calendar.

Capabilities moved: none (parsing, decoding and serializing give the same bytes and values as before).
