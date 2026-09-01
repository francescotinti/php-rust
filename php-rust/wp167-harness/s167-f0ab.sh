#!/bin/bash
# s167-f0ab.sh — FETTA 0, bracci (a)+(b) del criterio s167-criterio-f0.md
# (PRE-registrato): (a) A/B PHPR_REG_LOWER on/off sul PIN s166 · (b) gemello
# data-stride vs arith-dq (entrambi ON). + riferimento oracle della finestra.
# R=5 interleaved per coppia; ns/iter=(med raw−floor)/N; rumore drop-1.
# rc autoritativo = f0-out/f0ab.rc; verdetto s167-f0ab-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
P=~/Claude/php-rust-output/release/phpr
O=/opt/homebrew/opt/php/bin/php
DQ="$H/../wp164-harness/arith-dq.php"
ST="$H/arith-stride.php"
EMPTY="$H/../wp164-harness/empty.php"
OUT="$H/f0-out"; mkdir -p "$OUT"
VERD="$H/s167-f0ab-verdetto.out"; RC="$OUT/f0ab.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo" >&2; exit 7; }
grep -qi "s167\|s-167" /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock assente" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-f0ab.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
PM="$(shasum -a 256 "$P" | cut -c1-8)"
[ "$PM" = 092dcff4 ] || { echo "pin!=s166 ($PM)" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$@" "$EMPTY"); b=$(ucpu "$@" "$EMPTY"); c=$(ucpu "$@" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
{
echo "== s167 F0 bracci (a)+(b) — pin s166 $PM MISURATO; giudice arith-dq N=250M; R=5 interleaved; criterio s167-criterio-f0.md =="
echo "sentinella LS: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
FON=$(floor3 env -u PHPR_REG_LOWER "$P"); FOFF=$(floor3 env PHPR_REG_LOWER=0 "$P"); FO=$(floor3 "$O")
echo "floors: on=$FON off=$FOFF oracle=$FO"
TSV="$OUT/f0ab-runs.tsv"; : > "$TSV"
for i in 1 2 3 4 5; do
  TON=$(ucpu env -u PHPR_REG_LOWER "$P" "$DQ"); TOFF=$(ucpu env PHPR_REG_LOWER=0 "$P" "$DQ")
  TDQ=$(ucpu env -u PHPR_REG_LOWER "$P" "$DQ"); TST=$(ucpu env -u PHPR_REG_LOWER "$P" "$ST")
  TOR=$(ucpu "$O" "$DQ")
  printf '%s\t%s\t%s\t%s\t%s\n' "$TON" "$TOFF" "$TDQ" "$TST" "$TOR" >> "$TSV"
  echo "  giro$i: on=$TON off=$TOFF dq=$TDQ stride=$TST oracle=$TOR"
done
python3 - "$TSV" "$FON" "$FOFF" "$FO" <<'PY'
import sys
rows = [l.split() for l in open(sys.argv[1])]
fon, foff, fo = map(float, sys.argv[2:5])
N = 250e6
def col(i, f): return sorted((float(r[i])-f)/N*1e9 for r in rows)
def med(v): return v[2]
def dr1(v):
    m = med(v); w = sorted(v, key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
on, off, dq, st, orc = col(0,fon), col(1,foff), col(2,fon), col(3,fon), col(4,fo)
print(f"(a) REG_LOWER: on={med(on):.2f} off={med(off):.2f} ns/iter D_ab={med(off)-med(on):+.2f} (rumore on'={dr1(on):.2f} off'={dr1(off):.2f}) — costo GIA' pagato dal lowering, non ricontare")
d_st = med(st)-med(dq)
print(f"(b) STRIDE: dq={med(dq):.2f} stride={med(st):.2f} D={d_st:+.2f} (rumore dq'={dr1(dq):.2f} st'={dr1(st):.2f})")
ref = "REFUTATO (KS-LE-167-1 parte 1: |D|<=0,5)" if abs(d_st) <= 0.5 else "NON refutato: canale pila/D-cache VIVO"
print(f"    canale pila/D-cache: {ref}")
gap = med(dq)-med(orc)
print(f"riferimento finestra: oracle={med(orc):.2f} (rumore {dr1(orc):.2f}) gap={gap:.2f} ns/iter (dossier: 38,2)")
PY
prc=$?
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
