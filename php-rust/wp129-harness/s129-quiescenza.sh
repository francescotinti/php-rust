#!/bin/bash
# s129-quiescenza.sh — GATE SEPARATO di quiescenza (criterio s129 p.8; lezione
# S-128: mai nello stesso comando del lancio). rc autoritativo = $1 (default
# wp129-harness/quiesce.rc): 0=PASS. Campione doppio a 5 s per i divoratori CPU.
set -u
export PATH=/usr/bin:/bin:/usr/sbin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
RC="${1:-$H/quiesce.rc}"
fail(){ echo "QUIESCENZA-FAIL: $1"; echo 1 > "$RC"; exit 1; }
pgrep -fl "phpr|php-server" && fail "processi phpr/php-server vivi"
pgrep -x cargo && fail "cargo in volo"
pgrep -x rustc && fail "rustc in volo"
snap(){ ps -Ao %cpu,comm | awk -v p="$1" 'index($0,p){s+=$1} END{printf "%.1f", s+0}'; }
for pat in rust-analyzer mediaanalysisd; do
  c1=$(snap "$pat"); /bin/sleep 5; c2=$(snap "$pat")
  awk -v a="$c1" -v b="$c2" -v p="$pat" 'BEGIN{ if (a>=5.0 && b>=5.0) exit 1 }' \
    || fail "$pat sopra il 5% CPU su entrambi i campioni ($c1 / $c2)"
done
LOAD=$(sysctl -n vm.loadavg | awk '{print $2}')
awk -v l="$LOAD" 'BEGIN{ if (l+0 >= 6.0) exit 1 }' || fail "loadavg1 $LOAD >= 6"
echo "QUIESCENZA-PASS load1=$LOAD"
echo 0 > "$RC"
