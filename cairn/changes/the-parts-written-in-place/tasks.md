---
cairn: tasks
change: the-parts-written-in-place
---

Stopped on 2026-11-03 at 12:15 with steps 1 to 3 of the proposal in the working tree, uncommitted. Pick up from the first unchecked task.

## Done

- [x] Baseline copy of HEAD, the cycle harness built from one directory with shuffled layouts, the differential crate refreshed (see the proposal's Measuring)
- [x] The user's yes on the ASCII proof, the crate's first `unsafe` (2026-11-03)
- [x] Step 1, the ASCII proof, kept for its cycles
- [x] Step 2, the common line written into its slot, its parameters in place, the block scans, kept for its cycles
- [x] Steps 1 and 2 through the differential: corpus and 35,160 edited inputs on four seeds, all agreeing
- [x] Steps 1 to 3 through the differential: corpus and 11,720 edited inputs on four other seeds, all agreeing; the suite under the default and no features; clippy `--all-targets --all-features -D warnings`; `cargo build --no-default-features`; `cargo fmt`

## Next session, in order

- [ ] Move the measured binaries off tmpfs (ext4, three copies each, runs rotated) and check that two copies of one binary agree within about 1% before trusting any delta
- [ ] Confirm steps 1 and 2 against the baseline on ext4 (they gained 12% to 23% on tmpfs, above its noise)
- [ ] Settle step 3 (`put`) and the reverted `resize_with` in `push_line`, each alone, over layout 0 and shuffled layouts 1 to 3 on a quiet machine (load average under 1, nothing on the measuring core or its sibling); keep each only for its cycles
- [x] Miri over the `tree::` unit tests (102 passed, `cargo miri test --lib tree::` from the nightly of ~/code/pimalaya/quoted-printable-codec/fuzz/shell.nix)
- [ ] Miri over the remaining unit tests and the corpus tests, the `unsafe` in `tree::codec::unescape::text` being new
- [ ] Profile after step 3: card and corpus `parse`, decode alone, `to_bytes` alone, `to_string`
- [ ] Decode into the property's slot (the handoff's lever 3, vcard's lever 2): `IcalProp::empty`, `IcalLine::decode_into`, the decoded list grown by an empty property, `decode_value_kind_into` with each arm storing its own value, a `plain` read for an unsplit value with no backslash, the unescape slow paths out of line
- [ ] The kind lookups by length and first byte, never copying the name (vcard's lever 3), with vcard's test of every spelling recased, one byte off, cut and grown
- [ ] The splits by byte (vcard's `PlainPieces`) where `str::split` or `split_once` with a char confirm each hit with `bcmp`
- [ ] The handoff's levers 4 to 8: the cold walks, the value node's walks, `to_string` of a parsed tree, the small writes, the structural scanner
- [ ] The bench rows owed (the handoff's list)
- [ ] Before landing: the handoff's "Before landing a pass" list (suite under default, all and no features; clippy; docs; the bare build; Miri; both fuzz targets ten minutes; `cargo fmt`; the differential with a planted break caught; the final table of every workload against the baseline over four layouts; criterion; benches/README.md; CHANGELOG; the log entry; this change archived; the handoff rewritten with the Measuring lessons), then ask the user before committing
