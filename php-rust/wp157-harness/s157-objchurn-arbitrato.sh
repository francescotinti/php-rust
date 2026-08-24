#!/bin/bash
# s157-objchurn-arbitrato.sh — ARBITRATO della guardia objchurn morsa al R=5
# (D=-10,0 vs soglia -6,7 con tick 3,33 = guardia SOTTO-RISOLUTA, REGOLE §3:
# «guardia su giudice quantizzato si ri-risolve a N con tick <= soglia/4»;
# precedente: backtrace->24 S-154/S-156). Giudice DERIVATO m-objchurn12.php
# (SOLO N: 3M->12M, tick 0,83). Stessa meccanica ucpu/floor3/med/trange
# dell'harness s157-ab-al1.sh (estratto dichiarato). R=5 ABAB.
# Comparatore STRETTO pre-registrato: morde sse D < -soglia,
# soglia = max(4; banda fondata 6,7; rumore drop-1).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s156-gemelloA"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s157-al1-B"
QUIESCE="$H/../wp129-harness/s129-quiescenza.sh"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s157-objchurn-arbitrato-verdetto.out"
[ -e "$VERD" ] && { echo "verdetto ESISTE" >&2; exit 7; }
RC="$OUT/objchurn-arb.rc"
grep -qi "s-157\|s157" /private/tmp/phpr-measure.lock 2>/dev/null || { echo 9 > "$RC"; exit 9; }
"$QUIESCE" "$OUT/quiesce-objchurn-arb.rc" > "$OUT/quiesce-objchurn-arb.log" 2>&1
QRC=$(cat "$OUT/quiesce-objchurn-arb.rc" 2>/dev/null || echo MANCANTE)
[ "$QRC" = 0 ] || { echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$A" | cut -c1-8)" = "c19079d3" ] || { echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$B" | cut -c1-8)" = "76787303" ] || { echo 1 > "$RC"; exit 1; }
ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
P="$H/m-objchurn12.php"
"$A" "$P" > "$OUT/objchurn12-A.out" 2>&1
"$B" "$P" > "$OUT/objchurn12-B.out" 2>&1
diff -q "$OUT/objchurn12-A.out" "$OUT/objchurn12-B.out" > /dev/null || { echo 2 > "$RC"; exit 2; }
FA=$(floor3 "$A" "$H/empty.php"); FB=$(floor3 "$B" "$H/empty.php")
TSV="$OUT/objchurn-arb.tsv"; : > "$TSV"
{
echo "== s157-objchurn-arbitrato: A=c19079d3 B=76787303 R=5 ABAB; derivato m-objchurn12 N=12000000 (tick 0,83 <= soglia/4); soglia=max(4; 6,7; rumore drop-1); comparatore STRETTO; quiescenza rc=$QRC =="
echo "N=12000000 floor_A=$FA floor_B=$FB"
for i in 1 2 3 4 5; do
  if [ $((i % 2)) -eq 1 ]; then
    TA=$(ucpu "$A" "$P"); TB=$(ucpu "$B" "$P"); ord=AB
  else
    TB=$(ucpu "$B" "$P"); TA=$(ucpu "$A" "$P"); ord=BA
  fi
  printf '%s\t%s\t%s\t%s\n' "$TA" "$TB" "$FA" "$FB" >> "$TSV"
  echo "  coppia$i [$ord]: rawA=$TA rawB=$TB"
done
python3 - "$TSV" <<'PY'
import sys
N=12000000.0
rows=[l.split("\t") for l in open(sys.argv[1])]
na=[(float(t[0])-float(t[2]))/N*1e9 for t in rows]
nb=[(float(t[1])-float(t[3]))/N*1e9 for t in rows]
def med(v):
    s=sorted(v); n=len(s)
    return s[n//2] if n%2 else (s[n//2-1]+s[n//2])/2
def trange(v):
    m=med(v); w=sorted(v,key=lambda x:(abs(x-m),x))[:-1]
    return max(w)-min(w)
ma,mb=med(na),med(nb); d=ma-mb
ra,rb=trange(na),trange(nb); noise=max(ra,rb)
thr=max(4.0,6.7,noise)
reg = d < -thr
print(f"ARBITRATO objchurn12: A={ma:.2f} B={mb:.2f} ns/iter D={d:+.2f} soglia_reg={-thr:.2f} (rumore drop-1 A'={ra:.2f} B'={rb:.2f}; banda fondata 6,7) -> {'REGRESSIONE CONFERMATA' if reg else 'morso REFUTATO (fluttuazione sotto-risoluta del giudice a N=3M)'}")
sys.exit(5 if reg else 0)
PY
prc=$?
echo "$prc" > "$RC"
exit "$prc"
} >> "$VERD" 2>&1
