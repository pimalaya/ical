#!/usr/bin/env bash
# build.sh <new|old|DIR> <name> [layout]: aligned release build of the cycle
# harness against the working tree (new, snapshotted to snap/<name>), the
# baseline copy (old) or a tree copy (DIR), always compiled from the one path
# lib/ so the same code builds the same binary. Layout 0 links functions in
# natural order; layouts 1 to 9 shuffle input sections with lld's seed.
# Three copies of the binary land on ext4 in $B as <name>.{0,1,2}.
set -e
R=/home/soywod/code/pimalaya/ical
P=/tmp/claude-1000/-home-soywod-code-pimalaya/763c3b3e-4fde-4255-978b-60f2447b812a/scratchpad
B=$R/target/icyc-bin
case $1 in new) SRC=$R ;; old) SRC=$P/base ;; *) SRC=$1 ;; esac
if [ $1 = new ]; then
  N=${2%-L*}
  rm -rf $P/snap/$N && mkdir -p $P/snap/$N
  cp -r $R/src $R/Cargo.toml $R/Cargo.lock $R/benches $R/examples $P/snap/$N/
fi
rm -rf $P/lib && mkdir -p $P/lib
cp -r $SRC/src $SRC/Cargo.toml $SRC/Cargo.lock $SRC/benches $SRC/examples $P/lib/
L=${3:-0}
SHUFFLE=""
if [ $L != 0 ]; then SHUFFLE="-C link-arg=-Wl,--shuffle-sections=.text.*=$L"; fi
T=$P/h/target-$L
rm -f $T/release/icyc
RUSTFLAGS="-C llvm-args=-align-all-functions=6 -C llvm-args=-align-loops=64 --remap-path-prefix=$P/lib=$R $SHUFFLE" \
  taskset -c 0-3,5-11,13-15 nix-shell $R/shell.nix --run "cd $P/h && CARGO_TARGET_DIR=$T cargo build --release -q 2>&1 | grep -E '^error' -A 12 || true; test -x $T/release/icyc"
for c in 0 1 2; do cp $T/release/icyc $B/$2.$c; done
echo built $2
