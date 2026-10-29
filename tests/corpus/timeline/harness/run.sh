#!/usr/bin/env bash
#
# Regenerates the frozen timeline corpus.
#
# Runs the calendars of tests/corpus/rie through ical-rs and through
# recurring-ical-events, crosses the two answers, and writes
# tests/corpus/timeline/windows.tsv and agreement.tsv. The series the two
# answer differently go to the work directory's disagreement.txt, for a human
# to triage. Everything the oracle needs comes from nix:
#
#     ./run.sh            # regenerate into ..
#     ./run.sh /tmp/work  # keep the intermediate answer files in /tmp/work
#
# The divergence corpus beside it is hand-curated, not generated.

set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
calendars=$(cd "$here/../../rie" && pwd)
work=${1:-$(mktemp -d)}
mkdir -p "$work"

echo "harness: work directory $work" >&2

# NOTE: Built into the work directory, so no build output lands in the
# repository.
export CARGO_TARGET_DIR="$work/target"
cargo build --quiet --release --manifest-path "$here/Cargo.toml"
harness="$work/target/release/harness"

echo "harness: windowing the calendars" >&2
"$harness" windows "$calendars" >"$work/windows.tsv"

echo "harness: expanding through ical-rs" >&2
"$harness" expand "$calendars" <"$work/windows.tsv" >"$work/ical.tsv"

echo "harness: expanding through recurring-ical-events" >&2
# `nix shell` puts the library on the store but not on any interpreter's path,
# so build an interpreter that carries it instead.
python=$(nix build --impure --no-link --print-out-paths \
	--expr 'let pkgs = import (builtins.getFlake "nixpkgs") {}; in pkgs.python3.withPackages (ps: [ ps.recurring-ical-events ])')
"$python/bin/python3" -I -c 'import importlib.metadata as m; print("harness: recurring-ical-events", m.version("recurring-ical-events"), "icalendar", m.version("icalendar"))' >&2
"$python/bin/python3" -I "$here/oracle_rie.py" "$calendars" <"$work/windows.tsv" >"$work/oracle.tsv"

echo "harness: crossing the two answers" >&2
"$harness" cross "$work/ical.tsv" "$work/oracle.tsv" \
	>"$work/agreement.tsv" 2>"$work/disagreement.txt"

cp "$work/windows.tsv" "$here/../windows.tsv"
cat "$here/header.txt" "$work/agreement.tsv" >"$here/../agreement.tsv"

echo "harness: $(wc -l <"$work/agreement.tsv") series agree, $(grep -cv '^ ' "$work/disagreement.txt" || true) differ (see $work/disagreement.txt)" >&2
