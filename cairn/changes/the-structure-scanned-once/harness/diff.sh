#!/usr/bin/env bash
# diff.sh <rounds> <seed...>: snapshot the working tree into dnew/, build the
# differential crate once, then run one process per seed on cores 8-11,13-15.
set -e
R=/home/soywod/code/pimalaya/ical
P=/tmp/claude-1000/-home-soywod-code-pimalaya/763c3b3e-4fde-4255-978b-60f2447b812a/scratchpad
rounds=$1; shift
rm -rf $P/dnew && mkdir -p $P/dnew
cp -r $R/src $R/Cargo.toml $R/Cargo.lock $R/benches $R/examples $P/dnew/
taskset -c 8-11,13-15 nix-shell $R/shell.nix --run "cd $P/diff && CARGO_TARGET_DIR=$P/diff/target cargo build --release -q 2>&1 | grep -E '^error' -A 12 || true; test -x $P/diff/target/release/diff"
cores=(8 9 10 11 13 14 15)
i=0
for seed in "$@"; do
  taskset -c ${cores[$((i % 7))]} $P/diff/target/release/diff $rounds $seed > $P/diff/log.$seed 2>&1 &
  i=$((i+1))
done
wait
for seed in "$@"; do echo "seed $seed: $(tail -1 $P/diff/log.$seed)"; done
