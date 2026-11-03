#!/usr/bin/env bash
# lay.sh <runs> "<versions>" <work...>: every version built in layouts 0 to 3
# (<version>-L<n>.{0,1,2}), all interleaved; prints per version against the
# first the median delta in each layout and their mean, and the first
# version's layout-0 median and instruction count. CPU from $CPU (default 12).
B=/home/soywod/code/pimalaya/ical/target/icyc-bin
cd $B
CPU=${CPU:-12}
r=$1; vers=($2); shift 2
one() {
  case $2 in card-*) lo=20; hi=120;; *) lo=4; hi=24;; esac
  for n in $lo $hi; do
    taskset -c $CPU perf stat -x, -e cycles:u,instructions:u ./$1 $2 $n 2>&1 >/dev/null | cut -d, -f1 | tr '\n' ' '
    echo
  done | awk -v d=$((hi-lo)) 'NR==1{c=$1;i=$2} NR==2{printf "%.1f %.1f\n", ($1-c)/d/1e3, ($2-i)/d/1e3}'
}
med() { sort -n $1 | awk '{a[NR]=$1} END{print a[int((NR+1)/2)]}'; }
for w in "$@"; do
  for v in "${vers[@]}"; do for L in 0 1 2 3; do : > l.$v-L$L.$w.txt; done; done
  for i in $(seq $r); do for L in 0 1 2 3; do for v in "${vers[@]}"; do one $v-L$L.$((i%3)) $w >> l.$v-L$L.$w.txt; done; done; done
  ref=${vers[0]}
  line=$(printf "%-12s %9.1f i %9.1f" $w $(med l.$ref-L0.$w.txt) $(head -1 l.$ref-L0.$w.txt | cut -d' ' -f2))
  for v in "${vers[@]:1}"; do
    ds=""; sum=0
    for L in 0 1 2 3; do
      a=$(med l.$ref-L$L.$w.txt); b=$(med l.$v-L$L.$w.txt)
      d=$(awk -v a=$a -v b=$b 'BEGIN{printf "%+.1f", (b-a)/a*100}')
      ds="$ds $d"; sum=$(awk -v s=$sum -v d=$d 'BEGIN{print s+d}')
    done
    line="$line | $v$(awk -v s=$sum 'BEGIN{printf " %+5.1f%%", s/4}') [$ds ] i $(head -1 l.$v-L0.$w.txt | cut -d' ' -f2)"
  done
  echo "$line"
done
