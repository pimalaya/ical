# Harness of the speed passes

Copies of the scratchpad sources the second pass (the-parts-written-in-place) measured and checked with, kept here because the scratchpad lives on tmpfs. Untracked: copy them into a scratchpad, fix the paths at the top of each script and in each Cargo.toml (the scratchpad P, the repository R, the binaries B), and do not commit them.

- build.sh: builds the cycle harness against the working tree (`new`), the baseline copy (`old`, a copy of the tree before the pass in P/base) or any tree copy (a directory), always compiled from the one path P/lib so the same code builds the same binary; layout 0 links functions in natural order, layouts 1 to 9 shuffle the input sections with lld's seed. Three copies of each binary land on ext4 in R/target/icyc-bin.
- multi.sh: interleaved cycles and instructions a pass of several binaries on one layout, medians and minimums, runs rotated over the three copies.
- lay.sh: the same over layouts 0 to 3 of each version, the per-layout deltas against the first version and their mean. The final tables of a pass come from it.
- diff.sh: snapshots the working tree, builds the differential crate once and runs one seed a core on cores 8 to 11 and 13 to 15.
- codec.sh: the content decoders (the codec crate) before the switch to the SIMD codecs and after, with and without the detected token.
- cycles/: the cycle harness (`icyc <work> <passes>`): parse, decode, bytes, dec, tob, encode (to_string of decoded calendars), enc (encode of them), str (to_string of parsed trees), each also prefixed card- for the criterion calendar 300 times a pass.
- diff/: the differential crate against `ical-old`, a copy of the baseline's src and Cargo.toml renamed (lib `ical_old`, its bench, example and dev-dependency sections dropped).
- codec/: `icodec <work> <passes> [token]` over a folded 36 KiB inline ATTACH (b64), a soft-broken 1.0 QUOTED-PRINTABLE description (qp, cs through charset) and short ones (b64s, qps, css); its `old` feature builds against a tree from before the SIMD codecs (6545f7e).
- alloc/: allocations, reallocations and bytes of one corpus pass and of one card, a counting global allocator.
