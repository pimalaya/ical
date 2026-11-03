#!/usr/bin/env bash
# codec.sh <runs> <work...>: cycles a pass of the codec harness for the
# pre-codec tree, the current tree without a token and with the detected
# token, interleaved, medians.
B=/home/soywod/code/pimalaya/ical/target/icyc-bin
cd $B
r=$1; shift
one() { # one <bin> <work> [token]
  for n in 200 1200; do
    taskset -c 12 perf stat -x, -e cycles:u ./$1 $2 $n $3 2>&1 >/dev/null | cut -d, -f1
  done | awk 'NR==1{c=$1} NR==2{printf "%.2f\n", ($1-c)/1000/1e3}'
}
med() { sort -n | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}'; }
for w in "$@"; do
  : > c.pre.$w; : > c.base.$w; : > c.tok.$w
  for i in $(seq $r); do
    one codec-pre.$((i%3)) $w >> c.pre.$w
    one codec-cur.$((i%3)) $w >> c.base.$w
    one codec-cur.$((i%3)) $w token >> c.tok.$w
  done
  p=$(med < c.pre.$w); b=$(med < c.base.$w); t=$(med < c.tok.$w)
  awk -v w=$w -v p=$p -v b=$b -v t=$t 'BEGIN{printf "%-5s pre %9.2fk  baseline %9.2fk (%+.0f%%)  token %9.2fk (%+.0f%%)\n", w, p, b, (b-p)/p*100, t, (t-p)/p*100}'
done
