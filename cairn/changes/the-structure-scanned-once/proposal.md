---
cairn: change
id: the-structure-scanned-once
status: active
created: 2026-11-03
---

# The structure scanned once

The handoff for the next speed pass over the parser, the decoded model and the serializers, written at the end of [the-lines-built-in-place](../archive/the-lines-built-in-place/proposal.md) (2026-11-03) so a fresh session can take the lead on its own. It is the twin of vcard cairn/changes/the-structure-scanned-once/proposal.md, which carries the same method one pass ahead. Rewrite it at the end of each pass that takes it up: where things stand, where the time goes, the levers left, what was tried and dropped, the traps. Each pass lands under its own change id and log entry; this one stays active.

## Read first

- This repo's AGENTS.md, cairn/spec/parsing.md and cairn/spec/decoded-model.md: round-trip fidelity, the serialization fixpoint, line normalisation and the fold of an encoded line at 75 octets are the requirements a speed change must not move.
- cairn/log/2026-11-03-the-lines-built-in-place.md: what the first pass changed, each change with its cycles, and what it dropped.
- ~/code/pimalaya/vcard/cairn: its two passes (log/2026-11-02-the-lines-built-in-place.md, log/2026-11-03-the-card-written-straight.md) and its handoff changes/the-structure-scanned-once/proposal.md, whose levers and traps apply here almost word for word; ~/code/pimalaya/mime/cairn/changes/the-scratch-area/proposal.md for the method they come from.
- cairn/changes/a-folded-line-is-stripped-once-assembled: a draft fix in the join paths of `IcalLine::take_into` (a whitespace-only line's continuation leaves a space in front of the name). A speed pass keeps today's reading until that change lands, and a rewrite of the join paths carries its rule after.
- cairn/changes/an-edited-line-keeps-its-folds: a draft that will rewrite how an edited line is folded. A speed change in src/tree/wire.rs or `IcalLine::write_bytes` must keep today's output byte for byte, and `lay_out_value` (the one-copy layout of an unsplit value) must follow whatever it changes in `lay_out`.

## Goal and rules

Make parsing, decoding and serializing faster with no behaviour change:

- A change is kept only for its cycles, measured as below; a change that is flat goes, even if it looks cleaner to a profiler. Smaller source is a goal too.
- Every parser or model change gets the differential run below.
- No new dependency for the core. `cargo build --no-default-features` must stay dependency-free.
- The crate has no `unsafe`. The first one (lever 1) needs the user's yes, and every `unsafe` runs under Miri before landing.
- An API break needs the user's yes first.
- Each pass ends with: benches/README.md numbers, CHANGELOG, a log entry under the pass's own change id, this proposal rewritten as the next handoff, and a commit `perf: <what>` only once the user says yes to "commit now?" (no git write before that, no AI trailer in the message).

## Where things stand

After the first pass, cycles a pass on a quiet core: corpus `parse` 34,257k, `parse` + `decode` 60,926k, `parse` + `to_bytes` 41,665k, `to_string` of the decoded calendars 12,071k, `encode` of them 67,466k, `decode` of trees parsed beforehand 35,476k, `to_bytes` of them 21,360k; the card parsed 300 times 1,097k, 1,951k, 1,391k, 479k, 2,743k, 830k and 365k. Instructions a corpus pass: `parse` 127.5M, `parse` + `decode` 203.4M, `parse` + `to_bytes` 153.5M, `to_string` 47.9M.

Allocations a corpus pass: `parse` 89,245 (1,005 reallocations, 48.9 MB), `decode` 41,795, `to_bytes` 2,881, `to_string` 6,005; the card 11, 9, 1 and 2. A parsed line is 168 bytes, an item 168, a component 384; a decoded property is 120, a value 72.

The corpus is tests/corpus: 293 `.ics` files, 3.4 MB, 132,291 physical lines of which 12,281 are fold continuations, a median logical line of 21 bytes (90th percentile 44); 2,881 calendars and 12,997 components through `parse_many`, a median of 6 items a component, 90,019 property lines. The card is the 23-line calendar of benches/parse.rs. Look at both before keeping a change: the first pass's wins split differently between them, and the corpus rows that walk trees parsed beforehand (`dec`, `tob`) are memory-bound.

## Where the time goes

`perf record -c 200000 -e cycles:u` on the harness after the first pass:

- `parse` (card): `IcalLine::take_into` 34%, `IcalLine::build` 26%, memmove 14.5%, the value nodes' drop 4%, the parameter lists' drop 3%. 88% of `take_into`'s samples wait on one 16-byte load copying the freshly built wire shape into `build`'s argument; half of `build`'s on moving it into the slot. The same wait vcard-rs documents: a multi-word value written by 8-byte stores, then moved by 16-byte loads before they land.
- `parse` (corpus): `take_into` 26%, `build` 23%, `_int_malloc` 7.6%, memmove 7.2%, `from_utf8` 3.5% (the head), memchr's `find_sse2` 3.2% (one call a physical line).
- `decode` alone (card): `unescape_with` 47.5%, 98% of it on the load of `str::from_utf8`'s result, which comes back through memory (lever 1); `IcalLine::decode` 14.6%; `decode_component` 6% (the component's name).
- `to_string` (corpus): memmove 12.8%, `escaped_at` 11.8% (the sixteen-byte scan), `IcalProp::write_text` 11.2%, `escape_into` 5.1%, `percent_into` 4.9% (a byte loop over every URI), `from_utf8` 4.9% (the one check of the finished text), `IcalValue::emit` 4.3%, `fold_long` and `head_is_quoted_printable` 5.6% (the head scan for `QUOTED-PRINTABLE` on every line past 75 octets), `IcalPropKind`'s `Deref` 3.7%.
- `to_bytes` alone (corpus): `IcalLine::write_bytes` 29%, memmove 25%, `IcalCst::size_hint` 21% (a walk of trees out of cache), `IcalParamNode::write_bytes` 8%.

## Levers, in order

1. The ASCII proof (reserved: the user's yes first). `str::from_utf8` returns its `&str` through memory and the 16-byte reload waits on its two 8-byte stores; vcard-rs replaced it with `<[u8]>::is_ascii()` and `str::from_utf8_unchecked`, falling back to `from_utf8` otherwise (vcard `tree::codec::unescape::text`, its one `unsafe`, Miri-checked). Measured here on a scratchpad copy, applied to the head check in `IcalLine::build` and the decoded values in `utf8_lossy`: corpus `parse` -12.5%, `parse` + `decode` -13.0%, card decode alone -10.5%, card `parse` + `decode` -4.0%, card `parse` flat. Safe alternatives were flat in vcard-rs: an `#[inline(never)]` helper returning `Option<&str>` in registers moved the wait to the next copy of the returned `Cow`.
2. Fewer moves between the tokeniser and the line (vcard lever 1). Every local cut was flat in vcard-rs, the wait moving on: recording the wire shape straight into the slot (parse -0.7%, card +0.9%), the parameters pushed first and filled in place (+0.8%), `#[inline(always)]` on `build` (+6.3%). Try a structural one: the tokeniser recording a line as plain spans (offsets into the input, a few `u32`s held in registers) and one function writing the public `IcalLine` field by field, with no intermediate value larger than two words. Measure the card first.
3. Decode into the property's slot (vcard lever 2): the 120-byte `IcalProp` returned by value and pushed. The decoded lists are now sized; grow them by a constant placeholder and fill it.
4. The cold walks: `to_bytes` of a tree parsed long before is +22% since the first pass (it walks the tree once for its size before writing it, and the item lists now hold up to 16 slots, spreading the tree out), and decode alone of such trees gained 13% where the card gained 35%. A size the parser could record would need a field on `IcalCst`, an API break; per-component reserves and no presizing both lost on the in-cache rows (see Tried and dropped).
5. The value node: an unknown value's components are read one walk from the start each (`(0..component_count()).map(decode_component_list)`, quadratic in its `;`), and `decode_list` checks every item for UTF-8 on its own. vcard-rs walks the components once and checks a value once (`Components`, `Component::Text`, `unescape_text`); 2.3% of the corpus's lines are `X-` properties, mostly of one component, so this weighs little here.
6. `to_string` of a parsed tree: `Display` for `IcalLine` allocates a buffer and walks it lossily per line. vcard-rs writes a line with no wire shape straight to the formatter; keep the per-line lossy semantics, which a whole-tree conversion would change for a hand-built line ending.
7. Small writes: `percent_into` and `escape_param` are byte loops over every URI and parameter value; `fold_long` scans the head of every line past 75 octets for `QUOTED-PRINTABLE`. A block scan, as `escape_into` has, or the head's answer kept from the tokeniser.
8. A structural scanner on simd-cpu, gated (vcard lever 5): memchr is built without its `std` feature, so it never dispatches to AVX2, and each of the 132k physical lines pays its setup. Only a kernel bench on the corpus's line lengths that beats memchr's per-call scans earns it. If the parser must ask for positions, it asks through an I/O-free coroutine, never a trait.
9. The content codecs: `base64` and `quoted-printable` still run on the base64 0.23 and quoted_printable 0.5 crates; vcard-rs moved them to base64-codec and quoted-printable-codec under simd-cpu (vcard cairn/log/2026-11-02-the-simd-codecs.md). That moves the codecs' decoding rules (it did in vcard-rs), so it is a change of its own with a delta, not a speed pass. Landed as the-simd-codecs on 2026-11-03.

## The bench rows owed

- The corpus through `parse_many`, for ical-rs and for every rival that accepts each file. Whether the rivals parse all 293 files is unchecked: run each over the corpus first and state in the row how many calendars it took.
- A calendar with a folded inline `ATTACH` of some tens of kilobytes, and a vCalendar 1.0 calendar in `QUOTED-PRINTABLE` (`CHARSET`, soft breaks).
- The first pass added `to_string` of the decoded calendar to the pipeline group.

## Tried and dropped

From the first pass (numbers in its log):

- `to_bytes` sized component by component, and with no presizing at all: both lost on the in-cache rows.
- Item lists of 8, and of 8 below depth 2.
- `shrink_to_fit` on each component's items once read.

## Traps

- Other sessions measure on this machine too, and pin to core 4 (another session's criterion ran `taskset -c 4` through the first pass, overlapping it for an hour unseen). Measure on core 12 (CPUs 12 and 28, the second CCD) and check `ps -eo pcpu,psr,args --sort=-pcpu` before every run: any process on the measuring core or its sibling voids the run. Never run the differential or the fuzzers while measuring; run them on cores 8 to 11 and 13 to 15 otherwise.
- One binary measured twice in single runs can move 5%: only medians of interleaved runs decide, and minimums beside them when the load moved. Two builds of near-identical code still differ by up to 1.5% from layout.
- The corpus rows over trees parsed beforehand (`dec`, `tob`) are memory-bound and swing 37k to 58k cycles between runs of one binary: decide on the card and on the rows that parse first.
- A failed-forward site removed often moves the wait to the next load: keep a change only for its cycles, not for fewer instructions or a lower stall count.
- A block-scan predicate must join `==` tests with `|`: `matches!` compiles to a branch per byte.
- perf is user space only here (`perf_event_paranoid` 2): `cycles:u`, `instructions:u` and branch stacks (`perf record -b`) work, call graphs do not.
- Edit code with the Edit tool, not `perl` or `sed` substitutions: a `s{}{}` with braces in the replacement fails, a `|` delimiter turns `\|` into an alternation. Never recover a file with git: stop and report.
- A planted break restored by an exact-string edit can match a second, now identical, line: check the restore by grepping both.
- The merge fuzz target saves slow units on calendars of several hundred kilobytes (the merge takes about a second on one in a release build, the baseline alike): not a speed regression of a parser pass. A `pgrep -f` waiter matches its own command line: wait on a background task's notice instead.
- Never edit src/ while a background build or fuzz run may still compile it.
- `cargo doc` with `RUSTDOCFLAGS='--cfg docsrs'` fails on stable; check docs with `RUSTDOCFLAGS=-Dwarnings` under all features, where the bare URLs of src/tzdb/windows.rs fail on the baseline too (allow `rustdoc::bare_urls` to see the rest).

## How to measure

Keep the harness outside the repo, in a scratchpad, its path dependency pointing at the working tree, and a second one pointing at a copy of the tree before the pass (src, benches, examples, Cargo.toml, Cargo.lock). Cargo.toml:

```toml
[package]
name = "icyc"
version = "0.0.0"
edition = "2024"

[dependencies]
ical = { package = "ical-rs", path = "/home/soywod/code/pimalaya/ical" }

[profile.release]
codegen-units = 1
debug = 1

[workspace]
```

src/main.rs, run as `icyc <work> <passes>`, `<work>` one of `parse`, `decode` (parse then `decode`), `bytes` (parse then `to_bytes`), `dec` (`decode` of calendars parsed beforehand), `tob` (`to_bytes` of them), `encode` (`to_string` of calendars decoded beforehand), `enc` (`encode` of them), or any of them prefixed `card-` for the criterion calendar parsed 300 times a pass:

```rust
use std::{fs, hint::black_box, path::Path};

use ical::tree::cst::IcalCst;

fn walk(p: &Path, out: &mut Vec<Vec<u8>>) {
    let mut es: Vec<_> = fs::read_dir(p).unwrap().flatten().map(|e| e.path()).collect();
    es.sort();
    for e in es {
        if e.is_dir() { walk(&e, out) } else if e.extension().is_some_and(|x| x == "ics") { out.push(fs::read(e).unwrap()) }
    }
}

fn main() {
    let mut a = std::env::args().skip(1);
    let work = a.next().unwrap();
    let n: usize = a.next().unwrap().parse().unwrap();
    let mut corpus = Vec::new();
    walk(Path::new("/home/soywod/code/pimalaya/ical/tests/corpus"), &mut corpus);
    if let Some(short) = work.strip_prefix("card-") {
        corpus = vec![CAL.as_bytes().to_vec(); 300];
        return run(short, n, &corpus);
    }
    run(&work, n, &corpus);
}

fn run(work: &str, n: usize, corpus: &[Vec<u8>]) {
    let csts = || corpus.iter().flat_map(|f| IcalCst::parse_many(f.as_slice()).flatten()).collect::<Vec<_>>();
    if work == "dec" || work == "tob" {
        let csts = csts();
        for _ in 0..n {
            for cst in &csts {
                if work == "dec" { black_box(black_box(cst).decode()); } else { black_box(black_box(cst).to_bytes()); }
            }
        }
        return;
    }
    if work == "encode" || work == "enc" {
        let csts = csts();
        let cals: Vec<_> = csts.iter().map(|c| c.decode()).collect();
        for _ in 0..n {
            for cal in &cals {
                if work == "encode" { black_box(black_box(cal).to_string()); } else { black_box(black_box(cal).encode()); }
            }
        }
        return;
    }
    for _ in 0..n {
        for f in corpus {
            for cal in IcalCst::parse_many(black_box(f.as_slice())) {
                let Ok(cal) = cal else { continue };
                match work {
                    "parse" => { black_box(&cal); }
                    "decode" => { black_box(cal.decode()); }
                    "bytes" => { black_box(cal.to_bytes()); }
                    _ => panic!(),
                }
            }
        }
    }
}

// NOTE: the CAL constant of benches/parse.rs, copied verbatim.
const CAL: &str = "...";
```

Build it aligned, each version in its own target directory, copy the binary aside under its own name, and stop on a failed build:

```sh
RUSTFLAGS='-C llvm-args=-align-all-functions=6 -C llvm-args=-align-loops=64' \
  nix-shell /home/soywod/code/pimalaya/ical/shell.nix \
  --run "CARGO_TARGET_DIR=$PWD/target-aligned cargo build --release -q" \
  && cp target-aligned/release/icyc bin/icyc-new && echo built
```

Count cycles at two pass counts and take the difference, which removes the corpus loading: 4 and 24 passes over the corpus, 20 and 120 over the card. Compare two versions by interleaving 7 runs of each (11 for a close call or a final table) and comparing medians and minimums, the instruction count beside (`nix-shell -p perf --run "./ab.sh icyc-old icyc-new 7 parse decode bytes encode card-parse card-decode card-bytes card-encode"`, from bin/):

```sh
A=$1; B=$2; r=$3; shift 3
one() {
  case $2 in card-*) lo=20; hi=120;; *) lo=4; hi=24;; esac
  for n in $lo $hi; do
    taskset -c 12 perf stat -x, -e cycles:u,instructions:u ./$1 $2 $n 2>&1 >/dev/null | cut -d, -f1 | tr '\n' ' '
    echo
  done | awk -v d=$((hi-lo)) 'NR==1{c=$1;i=$2} NR==2{printf "%.1f %.1f\n", ($1-c)/d/1e3, ($2-i)/d/1e3}'
}
for w in "$@"; do
  : > a.$w.txt; : > b.$w.txt
  for i in $(seq $r); do one $A $w >> a.$w.txt; one $B $w >> b.$w.txt; done
  ma=$(sort -n a.$w.txt | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}')
  mb=$(sort -n b.$w.txt | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}')
  na=$(sort -n a.$w.txt | head -1 | cut -d' ' -f1); nb=$(sort -n b.$w.txt | head -1 | cut -d' ' -f1)
  ia=$(head -1 a.$w.txt | cut -d' ' -f2); ib=$(head -1 b.$w.txt | cut -d' ' -f2)
  awk -v w=$w -v ma=$ma -v mb=$mb -v na=$na -v nb=$nb -v ia=$ia -v ib=$ib 'BEGIN{printf "%-12s med %9.1f %9.1f %+6.1f%%  min %9.1f %9.1f %+6.1f%%  instr %9.1f %9.1f %+6.1f%%\n", w, ma, mb, (mb-ma)/ma*100, na, nb, (nb-na)/na*100, ia, ib, (ib-ia)/ia*100}'
done
```

To count allocations, a second harness wraps `System` in a counting `#[global_allocator]` (allocations, reallocations and bytes, counted only inside a closure) and reports `parse`, `decode`, `to_bytes` and `to_string` for one corpus pass and one calendar.

To name a stalling copy, sample cycles on the debug-info build and annotate the symbol as `perf report` prints it (`perf annotate --stdio -s '<ical::tree::line::IcalLine>::take_into'`); the hot instruction is the one after the waiting load.

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

`cargo test` under default, all and no features; clippy `--all-targets --all-features -- -D warnings`; `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features`; `cargo build --no-default-features` depending on nothing; Miri over the unit tests and the corpus tests when `unsafe` changed; the parse and merge fuzz targets for ten minutes each (`nix-shell fuzz/shell.nix --run "cd fuzz && cargo fuzz run -O parse -- -max_total_time=600"` on cores away from the measuring one), seeded as fuzz/README.md says; `cargo fmt`; the differential run; the cycle harness on every workload against the pass's baseline; a criterion run interleaved with the baseline's; benches/README.md numbers; CHANGELOG; a log entry; this proposal rewritten; then ask the user before committing.
