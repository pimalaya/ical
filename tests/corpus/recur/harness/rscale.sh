#!/usr/bin/env bash
#
# Regenerates the frozen RFC 7529 RSCALE corpus.
#
# Answers the RSCALE case list through ical-rs, built with its rscale feature,
# and through libical built against ICU, and writes the cases the two answer
# alike to tests/corpus/recur/rscale.tsv. The cases they answer differently go
# to the work directory's disagreement.tsv, for a human to triage. Everything
# comes from nix:
#
#     ./rscale.sh            # regenerate into ../rscale.tsv
#     ./rscale.sh /tmp/work  # keep the intermediate answer files in /tmp/work

set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
work=${1:-$(mktemp -d)}
mkdir -p "$work"

echo "harness: work directory $work" >&2

# NOTE: Built into the work directory, so no build output lands in the
# repository.
export CARGO_TARGET_DIR="$work/target"

cargo run --quiet --release --manifest-path "$here/Cargo.toml" -- generate-rscale >"$work/rscale-cases.tsv"
cargo run --quiet --release --manifest-path "$here/Cargo.toml" -- expand \
	<"$work/rscale-cases.tsv" >"$work/rscale-ical.tsv"

libical_dev=$(nix build --no-link --print-out-paths 'nixpkgs#libical.dev')
libical_lib=$(nix build --no-link --print-out-paths 'nixpkgs#libical')
echo "harness: libical $(nix eval --raw 'nixpkgs#libical.version'), ICU $(nix eval --raw 'nixpkgs#icu.version')" >&2
nix shell nixpkgs#gcc --command sh -c "
	cc -O2 -o '$work/libical-oracle' '$here/oracle_libical.c' \
		-I'$libical_dev/include' -L'$libical_lib/lib' -lical
	LD_LIBRARY_PATH='$libical_lib/lib' '$work/libical-oracle' \
		<'$work/rscale-cases.tsv' >'$work/rscale-libical.tsv'
"

# NOTE: Both answer files list the cases in the order generated, so they are
# crossed line by line. A case libical refuses or gives up on is no answer.
paste "$work/rscale-ical.tsv" "$work/rscale-libical.tsv" | awk -F '\t' -v agree="$work/rscale.tsv" -v differ="$work/disagreement.tsv" '
	$6 == "REFUSED" || $6 == "START_ERROR" || $6 == "TIMEOUT" { next }
	$3 == $6 { print $1 "\t" $2 "\t" $3 > agree; next }
	{ print $1 "\t" $2 "\tical-rs: " $3 "\tlibical: " $6 > differ }
'

cat "$here/rscale-header.txt" "$work/rscale.tsv" >"$here/../rscale.tsv"

echo "harness: $(wc -l <"$work/rscale.tsv") cases agree, $(wc -l <"$work/disagreement.tsv" 2>/dev/null || echo 0) differ (see $work/disagreement.tsv)" >&2
