#!/usr/bin/env bash
#
# Regenerates the frozen timeline corpus.
#
# Runs the calendars of tests/corpus/rie, tests/corpus/icaljs and
# tests/corpus/libical through ical-rs, through recurring-ical-events and
# through ical.js, crosses the three answers, and writes
# tests/corpus/timeline/windows.tsv, agreement.tsv (all three alike) and
# split.tsv (the oracles split, ical-rs with one of them). The series ical-rs
# agrees with neither oracle on, and the calendars an oracle refuses, go to the
# work directory's disagreement.txt, for a human to triage. Everything the
# oracles need comes from nix:
#
#     ./run.sh            # regenerate into ..
#     ./run.sh /tmp/work  # keep the intermediate answer files in /tmp/work
#
# The divergence corpus beside it is hand-curated, not generated.

set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
corpus=$(cd "$here/../.." && pwd)
work=${1:-$(mktemp -d)}
mkdir -p "$work"

echo "harness: work directory $work" >&2

# NOTE: Built into the work directory, so no build output lands in the
# repository.
export CARGO_TARGET_DIR="$work/target"
cargo build --quiet --release --manifest-path "$here/Cargo.toml"
harness="$work/target/release/harness"

echo "harness: windowing the calendars" >&2
"$harness" windows "$corpus" >"$work/windows.tsv"

echo "harness: expanding through ical-rs" >&2
"$harness" expand "$corpus" <"$work/windows.tsv" >"$work/ical.tsv"

echo "harness: expanding through recurring-ical-events" >&2
# `nix shell` puts the library on the store but not on any interpreter's path,
# so build an interpreter that carries it instead.
python=$(nix build --impure --no-link --print-out-paths \
	--expr 'let pkgs = import (builtins.getFlake "nixpkgs") {}; in pkgs.python3.withPackages (ps: [ ps.recurring-ical-events ])')
"$python/bin/python3" -I -c 'import importlib.metadata as m; print("harness: recurring-ical-events", m.version("recurring-ical-events"), "icalendar", m.version("icalendar"))' >&2
"$python/bin/python3" -I "$here/oracle_rie.py" "$corpus" <"$work/windows.tsv" >"$work/rie.tsv"

echo "harness: expanding through ical.js" >&2
# The npm release tarballs, unpacked into the store and pinned by their hash:
# the oracle imports ical.js's ES module by path and reads the time zone
# database's zones.json, so nothing is installed.
icaljs_version=2.2.1
icaljs=$(nix eval --raw --impure --expr "builtins.fetchTarball {
	url = \"https://registry.npmjs.org/ical.js/-/ical.js-$icaljs_version.tgz\";
	sha256 = \"0nyziiyxi5j8szwxg7mqvmji3yn8i45i7glp6pa6y29zb5w3gnvy\";
}")
timezones_version=1.0.2
timezones=$(nix eval --raw --impure --expr "builtins.fetchTarball {
	url = \"https://registry.npmjs.org/@nextcloud/timezones/-/timezones-$timezones_version.tgz\";
	sha256 = \"1i0mhf1660q6vl7d90nmckks8am4ia36x60a6mjayxpfpf6w7zhx\";
}")
zones="$timezones/resources/timezones/zones.json"
node=$(nix build --impure --no-link --print-out-paths \
	--expr '(import (builtins.getFlake "nixpkgs") {}).nodejs')
echo "harness: ical.js $icaljs_version, @nextcloud/timezones $timezones_version (zones $("$node/bin/node" -p "require('$zones').version")), node $("$node/bin/node" --version)" >&2
"$node/bin/node" "$here/oracle_icaljs.mjs" "$icaljs" "$corpus" "$zones" <"$work/windows.tsv" >"$work/icaljs.tsv"

echo "harness: crossing the three answers" >&2
"$harness" cross "$work/ical.tsv" "$work/rie.tsv" "$work/icaljs.tsv" "$work/split.tsv" \
	>"$work/agreement.tsv" 2>"$work/disagreement.txt"

cp "$work/windows.tsv" "$here/../windows.tsv"
cat "$here/header.txt" "$work/agreement.tsv" >"$here/../agreement.tsv"
cat "$here/split_header.txt" "$work/split.tsv" >"$here/../split.tsv"

echo "harness: $(wc -l <"$work/agreement.tsv") series agree, $(wc -l <"$work/split.tsv") split, $(grep -v '^ ' "$work/disagreement.txt" | grep -cv '	ERROR$' || true) against both, $(grep -c '	ERROR$' "$work/disagreement.txt" || true) refused (see $work/disagreement.txt)" >&2
