#!/usr/bin/env bash
#
# Regenerates the frozen recurrence corpus.
#
# Runs the case list through ical-rs and the two oracles, crosses the three
# answers, and writes tests/corpus/recur/consensus.tsv. Everything the oracles
# need comes from nix, so nothing has to be installed:
#
#     ./run.sh            # regenerate into ../consensus.tsv
#     ./run.sh /tmp/work  # keep the intermediate answer files in /tmp/work
#
# The divergence corpus beside it is hand-curated, not generated: it records the
# cases where ical-rs deliberately parts from an oracle.

set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
work=${1:-$(mktemp -d)}
mkdir -p "$work"

echo "harness: work directory $work" >&2

echo "harness: generating cases" >&2
cargo run --quiet --release --manifest-path "$here/Cargo.toml" -- generate >"$work/cases.tsv"
wc -l <"$work/cases.tsv" | xargs echo "harness: cases" >&2

echo "harness: expanding through ical-rs" >&2
cargo run --quiet --release --manifest-path "$here/Cargo.toml" -- expand \
	<"$work/cases.tsv" >"$work/ical.tsv"

echo "harness: expanding through python-dateutil" >&2
# `nix shell nixpkgs#python3Packages.python-dateutil` puts the library on the
# store but not on any interpreter's path, so build an interpreter that carries
# it instead.
python=$(nix build --impure --no-link --print-out-paths \
	--expr 'let pkgs = import (builtins.getFlake "nixpkgs") {}; in pkgs.python3.withPackages (ps: [ ps.python-dateutil ])')
"$python/bin/python3" "$here/oracle_dateutil.py" <"$work/cases.tsv" >"$work/dateutil.tsv"

echo "harness: building and expanding through libical" >&2
# The headers live in libical's `dev` output, which `nix shell` does not put on
# any search path, so take both store paths and point the compiler at them.
libical_dev=$(nix build --no-link --print-out-paths 'nixpkgs#libical.dev')
libical_lib=$(nix build --no-link --print-out-paths 'nixpkgs#libical')
nix shell nixpkgs#gcc --command sh -c "
	cc -O2 -o '$work/libical-oracle' '$here/oracle_libical.c' \
		-I'$libical_dev/include' -L'$libical_lib/lib' -lical
	LD_LIBRARY_PATH='$libical_lib/lib' '$work/libical-oracle' \
		<'$work/cases.tsv' >'$work/libical.tsv'
"

echo "harness: crossing the three answers" >&2
cargo run --quiet --release --manifest-path "$here/Cargo.toml" -- cross \
	"$work/ical.tsv" "$work/dateutil.tsv" "$work/libical.tsv" >"$work/consensus.tsv"

cat "$here/header.txt" "$work/consensus.tsv" >"$here/../consensus.tsv"
echo "harness: wrote $(cd "$here/.." && pwd)/consensus.tsv" >&2
