#!/usr/bin/env bash
#
# Times python-dateutil, recurring-ical-events and libical on the cases of
# benches/recur.rs, for the ballpark table of benches/README.md. Everything
# comes from nix, as in the corpus harnesses:
#
#     benches/ballpark/run.sh

set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
corpus=$(cd "$here/../../tests/corpus" && pwd)
work=$(mktemp -d)

python=$(nix build --impure --no-link --print-out-paths \
	--expr 'let pkgs = import (builtins.getFlake "nixpkgs") {}; in pkgs.python3.withPackages (ps: [ ps.python-dateutil ps.recurring-ical-events ])')
"$python/bin/python3" -I "$here/ballpark.py" "$corpus"

libical_dev=$(nix build --no-link --print-out-paths 'nixpkgs#libical.dev')
libical_lib=$(nix build --no-link --print-out-paths 'nixpkgs#libical')
nix shell nixpkgs#gcc --command sh -c "
	cc -O2 -o '$work/ballpark' '$here/ballpark.c' \
		-I'$libical_dev/include' -L'$libical_lib/lib' -lical \
		-Wl,-rpath,'$libical_lib/lib'"
"$work/ballpark"
