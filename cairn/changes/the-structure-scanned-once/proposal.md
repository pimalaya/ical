---
cairn: change
id: the-structure-scanned-once
status: active
created: 2026-11-03
---

# The structure scanned once

The handoff for the next speed pass over the parser, the decoded model and the serializers, rewritten at the end of [the-parts-written-in-place](../archive/the-parts-written-in-place/proposal.md) (2026-11-03) so a fresh session can take the lead on its own. It is the twin of vcard cairn/changes/the-structure-scanned-once/proposal.md, which carries the same method. Rewrite it at the end of each pass that takes it up: where things stand, where the time goes, the levers left, what was tried and dropped, the traps. Each pass lands under its own change id and log entry; this one stays active.

## Read first

- This repo's AGENTS.md, cairn/spec/parsing.md and cairn/spec/decoded-model.md: round-trip fidelity, the serialization fixpoint, line normalisation and the fold of an encoded line at 75 octets are the requirements a speed change must not move.
- cairn/log/2026-11-03-the-lines-built-in-place.md and cairn/log/2026-11-03-the-parts-written-in-place.md: what the two passes changed, each change with its cycles, and what they dropped.
- ~/code/pimalaya/vcard/cairn: its passes and its handoff changes/the-structure-scanned-once/proposal.md, whose levers and traps apply here almost word for word; ~/code/pimalaya/mime/cairn/changes/the-scratch-area/proposal.md for the method they come from.
- cairn/changes/a-folded-line-is-stripped-once-assembled: a draft fix in the join paths of `IcalLine::take_wired` (a whitespace-only line's continuation leaves a space in front of the name). A speed pass keeps today's reading until that change lands, and a rewrite of the join paths carries its rule after.
- cairn/changes/an-edited-line-keeps-its-folds: a draft that will rewrite how an edited line is folded. A speed change in src/tree/wire.rs or `IcalLine::write_bytes` must keep today's output byte for byte, and `lay_out_value` (the one-copy layout of an unsplit value) must follow whatever it changes in `lay_out`.
- harness/README.txt in this folder: the cycle, codec, allocation and differential harnesses and their scripts, untracked.

## Goal and rules

Make parsing, decoding and serializing faster with no behaviour change:

- A change is kept only for its cycles, measured as below; a change that is flat goes, even if it looks cleaner to a profiler. Smaller source is a goal too.
- Every parser or model change gets the differential run below.
- No new dependency for the core. `cargo build --no-default-features` must stay dependency-free.
- The crate's one `unsafe` is the ASCII proof in `tree::codec::unescape::text`, approved by the user on 2026-11-03. Any other `unsafe` needs the user's yes first, and every `unsafe` runs under Miri before landing.
- An API break needs the user's yes first.
- Each pass ends with: benches/README.md numbers, CHANGELOG, a log entry under the pass's own change id, this proposal rewritten as the next handoff, and a commit `perf: <what>` only once the user says yes to "commit now?" (no git write before that, no AI trailer in the message).

## Where things stand

After the second pass, cycles a pass on a quiet core (layout 0): corpus `parse` 24,140k, `parse` + `decode` 43,582k, `parse` + `to_bytes` 30,594k, `decode` of trees parsed beforehand 29,463k, `to_bytes` of them 20,250k, `to_string` of the decoded calendars 11,274k, `encode` of them 54,113k, `to_string` of the parsed trees 24,520k; the card parsed 300 times 789k, 1,496k, 1,031k, 631k, 354k, 459k, 2,061k and 729k. Instructions a corpus pass: `parse` 98.9M, `parse` + `decode` 161.7M, `parse` + `to_bytes` 124.9M, `to_string` of the decoded calendars 43.5M.

Allocations a corpus pass: `parse` 89,254 (802 reallocations, 49.9 MB), `decode` 41,796, `to_bytes` 2,881, `to_string` of the decoded calendars 6,005, of the parsed trees 5,765, `encode` 373,942; the card 12, 10, 1, 2, 2 and 72. A parsed line is 168 bytes, an item 168, a component 384; a decoded property is 120, a value 72, a parameter 48.

The corpus is tests/corpus: 293 `.ics` files, 3.4 MB, 132,291 physical lines of which 12,281 are fold continuations, a median logical line of 21 bytes (90th percentile 44); 2,881 calendars and 12,997 components through `parse_many`, a median of 6 items a component, 90,019 property lines. The card is the 23-line calendar of benches/parse.rs. Look at both before keeping a change: wins split differently between them, and the corpus rows that walk trees parsed beforehand (`dec`, `tob`) are memory-bound.

The content decoders run on base64-codec and quoted-printable-codec since the-simd-codecs; harness/codec measures them (see the log of the second pass): with the detected token installed, a 36 KiB inline attachment decodes in 2.0k cycles where base64 0.23 took 47.3k.

## Where the time goes

`perf record -c 50000 -e cycles:u` on the harness after the second pass:

- `parse` (card): `IcalLine::take_into` 21%, `IcalCst::take_body` 10%, memmove 10% (the 384-byte component placeholder copied into each box by a `memcpy` call, the calendar returned by value), `physical_line` 7%, `take_params` 7%, the allocator about 15%, the tree's drop about 15%.
- `parse` (corpus): `take_into` 20%, the allocator about 25% (`_int_malloc` 11.5%, `unlink_chunk` 3%, `malloc_consolidate` 2.4%: a component's item list of 16 slots, 2.7 KB, is past glibc's tcache limit of 1,032 bytes, so every one takes the large-chunk path and consolidates the fast bins), `physical_line` 9%, `take_body` 6.5%, `take_params` 4.5%.
- `decode` alone (card): `IcalLine::decode_into` 28%, spread over the value kind's jump table, the spec's indirect `value` call and the line's loads; `IcalCst::decode_into` 9.5%; `unescape_bytes` 6.5% and `decode_list` 3.4% (list values); `Vec::resize_with` 5.5%, not inlined; `IcalPropKind::from_upper` 4%.
- `to_bytes` alone (card, in cache): `IcalLine::write_bytes` 48%, `IcalCst::size_hint` 18%, `IcalParamNode::write_bytes` 13%. Out of cache (corpus) `size_hint` takes 20%, its walk missing first.
- `to_string` of the decoded calendars (corpus): `escaped_at` 14%, memmove 13% (short `extend_from_slice` calls), `IcalProp::write_text` 11%, `escape_into` 5.5%, `fold_long` 4.9%, `IcalValue::emit` 4.7%.
- `encode` of the decoded calendars (card): the allocator about 45% (`malloc_consolidate` 13%: an item list of seven lines or more passes 1 KB), memmove 16%.

## Levers, in order

1. One scan a line: `physical_line` finds the line feed and `head_marks` the colon, semicolon and quote over the same first blocks; one pass gathering both masks would read the head once. Then a structural scanner keeping a 64-byte mask of line feeds across lines, the next line's end read from it (portable first; a SIMD kernel on simd-cpu only if a kernel bench on the corpus's line lengths beats the portable one, and its `unsafe` needs a yes). If the parser must ask for positions, it asks through an I/O-free coroutine, never a trait.
2. A smaller item (an API break: the user's yes first). `IcalLine` is 168 bytes (name 24, parameters 24, value 56, line ending 24, wire shape 40): the line ending as a one-byte enum and the wire shape behind a box would bring it near 112, so a component's first nine items fit glibc's tcache and its list skips the large-chunk path, and every placeholder line falls under LLVM's 128-byte inline copy. The parse rows spend a quarter of their cycles in the allocator.
3. The component box written field by field (`Box::new_uninit` and a write per field, `unsafe`: a yes first): the 384-byte placeholder is copied by a `memcpy` call per component.
4. Decoding: the value kind of a property and a version from a table, rather than an indirect call to its spec's `value`; `Vec::resize_with` stays out of line in `IcalCst::decode_into` (try `push` of a constant `IcalProp`, 120 bytes and so copied inline).
5. The kind lookups: the canonical spelling matched first stays (see Tried and dropped); a candidate picked by length and two bytes and confirmed by fixed-size word compares, rather than `eq_ignore_ascii_case`, might still win.
6. List values: `split_on` calls `memchr2` once a piece (`decode_list` over `CATEGORIES`, `RDATE`, `EXDATE`); block scans over the pieces, as vcard-rs's `PlainPieces`.
7. The unknown value: its components are read one walk from the start each (`(0..component_count()).map(decode_component_list)`, quadratic in its `;`); 2.3% of the corpus's lines are `X-` properties, mostly of one component, so this weighs little.
8. The small writes: `escape_param` is a byte loop over every parameter value, and `write_text` writes short pieces through `extend_from_slice` calls (`put` serves `to_bytes` the same way).
9. `to_bytes` of a tree parsed long before: a size the parser records would skip the walk, but needs a field on `IcalCst` (an API break); per-component reserves and no presizing both lost on the in-cache rows.

## The bench rows owed

- The corpus through `parse_many`, for ical-rs and for every rival that accepts each file. Whether the rivals parse all 293 files is unchecked: run each over the corpus first and state in the row how many calendars it took.
- The folded `ATTACH` calendar and the vCalendar 1.0 `QUOTED-PRINTABLE` calendar are measured by harness/codec, not by criterion: a criterion group would need the detected token installed in the bench (simd-cpu is already a dev-dependency).

## Tried and dropped

From the first pass (numbers in its log): `to_bytes` sized component by component, and with no presizing at all; item lists of 8, and of 8 below depth 2; `shrink_to_fit` on each component's items once read.

From the second pass (numbers in its log): the kind lookup by length and two bytes with an `eq_ignore_ascii_case` check (+22% instructions on decode alone); `charset` keeping its UTF-8 buffer through `String::from_utf8` (core's validator walks accented text byte by byte, slower than encoding_rs's walk and the copy it saves); `IcalParamNode::decode` inlined into the list's `extend` (the parameter was still staged on the stack); a placeholder line pushed without `const` (LLVM hoists it to the stack and still copies it by `memcpy`); `push` of the constant placeholder line rather than `resize_with`; `put` replaced by `extend_from_slice`.

## Traps

- Other sessions measure on this machine too, and pin to core 4. Measure on core 12 (CPUs 12 and 28, the second CCD) and check `ps -eo pcpu,psr,args --sort=-pcpu` before every run: any process on the measuring core or its sibling voids the run. Never run the differential, Miri or the fuzzers while measuring, nor builds on the second CCD; run them on cores 8 to 11 and 13 to 15 otherwise.
- Measure binaries from ext4 (the repository's target/icyc-bin), three copies of each, runs rotated: byte-identical binaries on the tmpfs scratchpad measured 5% to 15% apart. Two copies of one binary agree within 2% on ext4 except the memory-bound rows.
- Build every version from one directory (build.sh's lib/): cargo hashes a path dependency's path into symbol names, which reorders functions. Decide over layout 0 and shuffled layouts 1 to 3 (lld `--shuffle-sections` seeds): a single layout moved rows by up to 11% for code they never run.
- One binary measured twice in single runs can move 5%: only medians of interleaved runs decide, and minimums beside them when the load moved.
- The corpus rows over trees parsed beforehand (`dec`, `tob`) are memory-bound: decide on the card and on the rows that parse first. They also move with where the parser placed the tree: allocating a component's box before its item list made `to_bytes` of parsed trees 4.9% faster for the same instructions.
- A failed-forward site removed often moves the wait to the next load (removing the parameter's stack copy was flat until the spec stopped coming back through memory): keep a change only for its cycles, not for fewer instructions or a lower stall count.
- A value over 128 bytes, a constant included, is copied by a `memcpy` call, LLVM's inline limit on the baseline target: `IcalLine` and `IcalItem` (168 bytes) and `IcalCst` (384). `resize_with` with a closure writes the placeholder's fields instead; `IcalProp` (120) and `IcalParamNode` (56) are copied inline.
- A block-scan predicate must join `==` tests with `|`: `matches!` compiles to a branch per byte.
- core's `str::from_utf8` walks non-ASCII byte by byte: encoding_rs validates accented text faster than core does, copy included.
- perf is user space only here (`perf_event_paranoid` 2): `cycles:u`, `instructions:u` and branch stacks (`perf record -b`, which names the callers of `memmove`) work, call graphs do not.
- `nix develop` gives rustc 1.95 (the flake's fenix), `nix-shell shell.nix` 1.99.1 (fenix monthly): the harness builds with the second, the suite with the first.
- Edit code with the Edit tool, not `perl` or `sed` substitutions: a `s{}{}` with braces in the replacement fails, a `|` delimiter turns `\|` into an alternation. Never recover a file with git: stop and report.
- A planted break restored by an exact-string edit can match a second, now identical, line: check the restore by grepping both.
- The merge fuzz target saves slow units on calendars of several hundred kilobytes (the merge takes about a second on one in a release build, the baseline alike): not a speed regression of a parser pass. A `pgrep -f` waiter matches its own command line: wait on a background task's notice instead.
- Never edit src/ while a background build or fuzz run may still compile it: the differential and codec builds compile snapshots for that reason.
- `cargo doc` with `RUSTDOCFLAGS='--cfg docsrs'` fails on stable; check docs with `RUSTDOCFLAGS=-Dwarnings` under all features, where the bare URLs of src/tzdb/windows.rs fail on the baseline too (allow `rustdoc::bare_urls` to see the rest).

## How to measure

The scripts and crates are in harness/ (untracked; README.txt there). The cycle harness `icyc <work> <passes>` depends on the tree through a path dependency on the scratchpad's lib/, which build.sh fills from the working tree, the baseline copy or any tree copy before an aligned release build (`-C llvm-args=-align-all-functions=6 -C llvm-args=-align-loops=64`, `codegen-units = 1`, `debug = 1`), linked in layout 0 or a shuffled one, three copies landing on ext4. `<work>` is one of `parse`, `decode` (parse then `decode`), `bytes` (parse then `to_bytes`), `dec` (`decode` of calendars parsed beforehand), `tob` (`to_bytes` of them), `str` (`to_string` of them), `encode` (`to_string` of calendars decoded beforehand), `enc` (`encode` of them), or any of them prefixed `card-` for the criterion calendar parsed 300 times a pass.

Count cycles at two pass counts and take the difference, which removes the corpus loading: 4 and 24 passes over the corpus, 20 and 120 over the card. multi.sh interleaves binaries on one layout (7 runs, 11 for a close call), lay.sh interleaves versions over layouts 0 to 3 and prints each layout's median delta and their mean: the final table of a pass comes from lay.sh at 11 runs.

harness/alloc counts allocations, reallocations and bytes of one corpus pass and one calendar through a counting global allocator; harness/codec times the content decoders (codec.sh).

To name a stalling copy, sample cycles on the debug-info build and annotate the symbol as `perf report` prints it (`perf annotate --stdio -s '<ical::tree::line::IcalLine>::take_into'`); the hot instruction is the one after the waiting load. `perf record -b` and `perf report --sort symbol_from,symbol_to` name the callers of `memmove`, and `objdump -d --disassemble=<symbol>` the size of each `memcpy` call.

For criterion, build the baseline copy's bench and the working tree's in their own target directories (`cargo bench --bench parse --no-run`), run the two executables alternately with `--bench --output-format bencher` on the measuring core, three times each, and take the medians.

## The differential run

For any parser or model change: copy the previous src/ and Cargo.toml into a scratch crate renamed `ical-old` (lib name `ical_old`, its `[features]` and `[dependencies]` kept, the `[[bench]]`, `[[example]]` and `[dev-dependencies]` sections dropped). For the corpus and 25 to 40 rounds of random edits of it per seed, several seeds on separate cores (a byte removed, replaced or inserted from an alphabet of CR, LF, space, tab, `:`, `;`, `,`, `"`, `=`, `\`, `^`, `ENCODING=QUOTED-PRINTABLE`, `QUOTED-PRINTABLE`, `CHARSET=`, `BEGIN` and `END` lines of `VCALENDAR`, `VEVENT`, `VALARM` and an `X-` component, `VERSION:1.0`, `VERSION:2.0`, non-ASCII bytes, `\r\n `, `\r\n\t`, `\n `, `=\r\n`, `==\r\n`, the escapes `\n`, `\N`, `\,`, `\;`, `\\`, the carets `^n`, `^'`, `^^`, `;VALUE=URI`, `;VALUE=DATE`, `;VALUE=TEXT`, `;TZID=`, quoted parameters holding `,`, `:` and `;`, whole `GEO`, `REQUEST-STATUS`, `RRULE`, `X-`, `ATTENDEE`, `CATEGORIES`, `RDATE` periods, `ATTACH` base64, a soft-broken `DESCRIPTION` and a folded `SUMMARY`; chunks copied from elsewhere; files over 4 KB cut to a random 4 KB window), through `parse`, `parse_many` and `parse_recovering`, each version lists its observations and the two lists must be equal:

- the trees and errors (`Debug`), `to_bytes`, `to_string` and `version`;
- the decoded model (`Debug`), its `to_string`, its `encode` (`Debug` and bytes) and every property's and component's `encode` under both escapers;
- the same on the decoded model salted: every string a model can hold (unknown names one in five or seven) gets a salt (separators, a backslash, line breaks, carets, quotes, non-ASCII, `=\r\n`) at the front, the back or both ends, picked by a running count so both versions salt alike, read under its own version and the other one;
- every cursor read on every line (`raw_value`, `raw_value_str`, `component_count`, `decode_component` 0 to 2, `decode_bytes`, then through `IcalValueCursor`: `text`, `bytes`, `list`, `component` 0, 1, 2 and 7, `quoted_printable`, `charset`);
- then, after `set_component(2, &["x,y", "z;"])` on every line of both, the trees, bytes, text, model checks and every cursor read again.

Save a failing input before panicking, and print the first differing bytes of the first differing observation. Plant a break in the old copy, in the code the change touches, and in the new code for a path the old one lacks, and see the run fail before trusting it.

## Before landing a pass

`cargo test` under default, all and no features; clippy `--all-targets --all-features -- -D warnings`; `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features`; `cargo build --no-default-features` depending on nothing; Miri over the unit tests when `unsafe` changed or a pass ran without it (`nix-shell ~/code/pimalaya/quoted-printable-codec/fuzz/shell.nix --run "cargo miri test --lib"`, its nightly carrying Miri); the parse and merge fuzz targets for ten minutes each (`nix-shell fuzz/shell.nix --run "cd fuzz && cargo fuzz run -O parse -- -max_total_time=600"` on cores away from the measuring one), seeded as fuzz/README.md says; `cargo fmt`; the differential run; the cycle harness on every workload against the pass's baseline over four layouts; a criterion run interleaved with the baseline's; benches/README.md numbers; CHANGELOG; a log entry; this proposal rewritten; then ask the user before committing.
