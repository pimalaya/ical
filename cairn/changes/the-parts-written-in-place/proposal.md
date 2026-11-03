---
cairn: change
id: the-parts-written-in-place
status: active
created: 2026-11-03
---

# The parts written in place

The second speed pass over the parser, the decoded model and the serializers, taken from the handoff [the-structure-scanned-once](../the-structure-scanned-once/proposal.md), which stays active. It ports vcard-rs's third pass (vcard cairn/log/2026-11-03-the-parts-written-in-place.md) and the handoff's lever 1. No behaviour change, no API change. The baseline is HEAD (89512ea, the tree of 924a3ae).

The session that started it stopped on a deadline (2026-11-03, 12:15) with three changes in the working tree, uncommitted. This proposal is the plan for the session that finishes it: read [tasks.md](./tasks.md) first.

## In the working tree

Each figure is the change against the build before it, cycles a pass, medians of 5 interleaved runs, `taskset -c 4`, layout 0 (see Measuring below).

1. The ASCII proof (the handoff's lever 1), which the user approved on 2026-11-03: `tree::codec::unescape::text` proves ASCII with `<[u8]>::is_ascii()` and calls `str::from_utf8_unchecked`, falling back to `str::from_utf8`, as vcard-rs's helper of the same name. Used for the head check in `IcalLine::build`, the borrowed arm of `utf8_lossy`, the resolved wire bytes (`ascii`) and the one check of the decoded calendar's text (`Display` for `Ical`). The crate's first `unsafe`, so it needs Miri before landing. Corpus `parse` -11.7%, `parse` + `decode` -13.0%, `parse` + `to_bytes` -9.6%, card `decode` alone -12.7%, card `parse` flat.
2. The common line written into its slot (vcard's lever 1). A line that is one physical line, not blank, indented, folded or soft-broken records no wire shape: `IcalLine::take_into` writes its name, value (`IcalValueNode::hold`), line ending and sealed length straight into the slot, and pushes its parameters as empty nodes filled where they land (`take_params`, `IcalParamNode::take_into`, which replaces `IcalParamNode::take`). Every other line takes the old path, now `take_wired`, whose `build` fills its parameters the same way. The line feed (`physical_line`) and the head's colon and semicolon (`head_marks`, settling a head with no quote before its colon) are found by the inlined 16-byte block scan `tree::codec::escape::position`, now `pub(crate)`; `head_marks` scans no further than the line rounded up to a whole block, so a run of colon-less lines (which `parse_recovering` reads one by one) stays linear. Against step 1: corpus `parse` -16.2%, card `parse` -22.9%, `parse` + `decode` -10.7% and -13.8%, `parse` + `to_bytes` -13.1% and -17.5%, `IcalCst::parse` (corpus) -13.1%, decode alone flat.
3. Short pieces of `to_bytes` by fixed-size moves (vcard's lever 4): `tree::leaf::put` writes a name, a parameter piece, the line ending and an unsplit value with no wire shape over sixteen zeroes by two overlapping moves, the output sized with sixteen bytes of headroom; a test checks every length from 0 to 26. Card `to_bytes` alone -6% to -9% in each of eight builds (two runs over four layouts), `parse` + `to_bytes` about -3%, corpus `to_bytes` alone -1% to -3%. Not yet settled: see tasks.

Tried and reverted: the empty line grown by `resize_with` in `push_line` (vcard kept it): card `parse` -0.5% to -1.8%, corpus `parse` -1.2% to -1.6%, but card `parse` + `decode` +2.8% to +6.9% in the runs taken; undecided under the layout noise below, measure again on its own.

Steps 1 and 2 together passed the differential crate against the baseline: the corpus and 30 rounds of edits on four seeds (35,160 edited inputs), all agreeing. Steps 1 to 3 then passed it again: the corpus and 10 rounds on four other seeds (11,720 edited inputs). The suite passes under the default features, and the tree is formatted.

The harness and the differential crate this pass used are copied in [harness/](./harness), untracked (see harness/README.txt).

## Measuring: what this session changed

- Two builds of the same code from two paths (the repository and a scratchpad copy) linked their functions in different orders, even with `--remap-path-prefix`: cargo hashes a path dependency's path into its symbol names, which reorders the generic instantiations and shifted .text by a cache line. The harness now builds every version from one directory, the tree copied into it before each build, so identical code gives an identical binary (checked with `cmp`, and section by section for a rebuilt snapshot).
- Moving .text as a whole (vcard's four layouts, by path prefixes of four lengths) keeps the functions' relative placement, so it cannot average out aliasing between hot functions: card `parse` + `decode` read +3.3% in all four such layouts for step 3, whose code that row never runs (same instruction count). Layouts 1 to 3 now shuffle the input sections with lld's `-Wl,--shuffle-sections=.text.*=<seed>`, layout 0 keeping the natural order.
- Single-layout rows moved up to 11% between builds that differ only in code the row never runs (card decode alone, 3,525k instructions in both). Decide close calls only over several shuffled layouts, on a quiet machine: the last run of the session, with the load average at 5.6, read 15% to 70% above the earlier medians and is void.
- Every binary this pass measured sat on tmpfs (the scratchpad under /tmp). The vcard-rs session found that byte-identical binaries copied to tmpfs measure 5% to 15% apart, one copy in twelve 34% slower (op-cache misses up 24%), the pages landing by chance and staying with the file, while copies on ext4 agree within 1.2% (memory note measuring-binary-placement). Steps 1 and 2 gained more than that noise; step 3 and `resize_with` did not, so measure them again with binaries on ext4 (vcard keeps three copies of each under the repository's target/ and rotates runs among them).

## Next

The levers left are the handoff's 3 to 9 and vcard's other third-pass changes, in tasks.md. Profile after step 2 (card `parse`): `take_into` and `build` held 62% of the cycles before it, mostly one 16-byte load waiting on the wire shape's stores, which the common line no longer stages.
