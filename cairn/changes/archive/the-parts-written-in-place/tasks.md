---
cairn: tasks
change: the-parts-written-in-place
---

The first part ran on 2026-11-03 from 11:25 to 12:15 and was committed as 6545f7e; the second ran the same day from 17:20, after [the-simd-codecs](../../../log/2026-11-03-the-simd-codecs.md), and landed the rest. Numbers in the [log](../../../log/2026-11-03-the-parts-written-in-place.md).

## First part (6545f7e)

- [x] Baseline copy of HEAD, the cycle harness built from one directory with shuffled layouts, the differential crate refreshed
- [x] The user's yes on the ASCII proof, the crate's first `unsafe` (2026-11-03)
- [x] Step 1, the ASCII proof, kept for its cycles
- [x] Step 2, the common line written into its slot, its parameters in place, the block scans, kept for its cycles
- [x] Step 3, short `to_bytes` pieces by fixed-size moves (`put`)
- [x] Steps 1 to 3 through the differential, the suite, clippy, the bare build, `cargo fmt`; Miri over the `tree::` unit tests

## Second part

- [x] Binaries measured from ext4 (target/icyc-bin), three copies each, runs rotated; an A/A check of two byte-identical builds within 2% (the memory-bound rows aside)
- [x] The whole pass against the tree before it (89512ea) over layout 0 and shuffled layouts 1 to 3, steps 1 to 3 included
- [x] Components filled in place: the box first, `BEGIN:` and `END:` lines straight into the envelope, the item list grown by `resize_with`, the box placeholder a constant
- [x] Decoding into slots: properties, components and parameters grown by constant empty ones and filled where they lie, a one-text value read inline when plain, `prop_spec` handing out a reference
- [x] Model encoding into slots, canonical names borrowed rather than allocated
- [x] The parsed tree written as text through one buffer
- [x] Block scans for a URI's percent escapes and the unquoted colon of a head
- [x] Dropped with their numbers: the kind lookup by length and two bytes, `charset` keeping the UTF-8 buffer, the parameter decoder inlined into `extend`
- [x] `put` and `resize_with` in `push_line` settled over four layouts
- [x] The codec integration measured: the content decoders before the switch and after, with and without the detected token
- [x] The differential on the final tree, with a planted break caught
- [x] Miri over the unit tests of the modules the pass touches and the parser paths reaching the `unsafe` in `tree::codec::unescape::text` (142 passed); a run over every unit test stalled on a recurrence arithmetic loop that reaches no `unsafe`
- [x] The parse and merge fuzz targets ten minutes each
- [x] Suite under default, all and no features; clippy; docs; the bare build; thumbv6m-none-eabi and riscv32imc-unknown-none-elf; `cargo fmt`
- [x] benches/README.md, CHANGELOG, the log, the handoff rewritten
- [ ] Commit, once the user says yes
