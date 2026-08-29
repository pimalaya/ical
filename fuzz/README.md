# Fuzzing

Coverage-guided fuzzing of the iCalendar parser with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer). The `parse` target checks two oracles: parsing arbitrary bytes never panics, and whatever parses serializes to a byte-stable fixpoint.

cargo-fuzz needs a nightly toolchain (for the `-Z` sanitizer flags). On NixOS, get both from the dedicated fuzz/shell.nix (nightly via fenix plus cargo-fuzz, no rustup or nix-ld needed):

```sh
nix-shell fuzz/shell.nix --run "cargo fuzz run parse"
```

libFuzzer saves every interesting new input into fuzz/corpus/parse/ (gitignored), and any crash into fuzz/artifacts/parse/. Do not pass tests/corpus as the corpus directory: libFuzzer treats the first corpus directory as writable and would dump generated inputs into the curated fixtures. To warm-start coverage from the real calendars, seed the fuzz corpus once with a copy:

```sh
mkdir -p fuzz/corpus/parse && cp tests/corpus/*/*.ics fuzz/corpus/parse/
```

Off NixOS, `cargo install cargo-fuzz` and a nightly toolchain give the same `cargo fuzz run parse`.

## The merge target

The `merge` target fuzzes the three-way merge. Its input is one byte choosing the collision preference, then three calendars separated by a NUL byte, so a mutation can move a line from one side to another. It checks five oracles: merging never panics, the merged calendar reparses to its own bytes, every conflict names an action the right side is reported to have taken, merging a calendar with itself against itself changes nothing and reports nothing, and merging two identical sides yields them unchanged unless the calendar holds two components at one path, which no addressing can tell apart.

Seed it from the frozen corpora, three copies of each fixture plus a pair that differ on one line, so the merge has a collision from the first unit:

```sh
mkdir -p fuzz/corpus/merge
for f in tests/corpus/*/*.ics; do
  name=$(basename "$f" .ics)
  { printf '\x00'; cat "$f"; printf '\x00'; cat "$f"; printf '\x00'; cat "$f"; } > "fuzz/corpus/merge/${name}_same"
  { printf '\x01'; cat "$f"; printf '\x00'; sed 's/^SUMMARY:.*/SUMMARY:left/' "$f"; printf '\x00'; sed 's/^SUMMARY:.*/SUMMARY:right/' "$f"; } > "fuzz/corpus/merge/${name}_edit"
done
```

