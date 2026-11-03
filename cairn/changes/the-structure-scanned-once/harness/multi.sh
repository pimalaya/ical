#!/usr/bin/env bash
# multi.sh <runs> "<binaries>" <work...>: interleaved cycle and instruction
# counts a pass (difference of two pass counts), medians and minimums, the
# first binary the reference; run i uses copy i%3 of each binary. CPU from
# $CPU (default 12).
B=/home/soywod/code/pimalaya/ical/target/icyc-bin
cd $B
CPU=${CPU:-12}
r=$1; bins=($2); shift 2
one() {
  case $2 in card-*) lo=20; hi=120;; *) lo=4; hi=24;; esac
  for n in $lo $hi; do
    taskset -c $CPU perf stat -x, -e cycles:u,instructions:u ./$1 $2 $n 2>&1 >/dev/null | cut -d, -f1 | tr '\n' ' '
    echo
  done | awk -v d=$((hi-lo)) 'NR==1{c=$1;i=$2} NR==2{printf "%.1f %.1f\n", ($1-c)/d/1e3, ($2-i)/d/1e3}'
}
for w in "$@"; do
  for b in "${bins[@]}"; do : > m.$b.$w.txt; done
  for i in $(seq $r); do for b in "${bins[@]}"; do one $b.$((i%3)) $w >> m.$b.$w.txt; done; done
  line=$(printf "%-12s" $w); first=""
  for b in "${bins[@]}"; do
    med=$(sort -n m.$b.$w.txt | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}')
    mn=$(sort -n m.$b.$w.txt | head -1 | cut -d' ' -f1)
    ins=$(sort -n m.$b.$w.txt | head -1 | cut -d' ' -f2)
    [ -z "$first" ] && first=$med && fmin=$mn
    line="$line $(awk -v m=$med -v n=$mn -v f=$first -v fm=$fmin -v i=$ins 'BEGIN{printf "| %9.1f %+5.1f%% min %9.1f %+5.1f%% i %9.1f", m, (m-f)/f*100, n, (n-fm)/fm*100, i}')"
  done
  echo "$line"
done
