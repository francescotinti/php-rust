#!/bin/bash
# s168-stride.sh — SANATURA az.rev.3 S-167 (braccio (b) stride): sul PIN s166,
# R=5 interleaved: arith-dq vs arith-stride (S-167: $s=14,$i=15 — STESSA linea
# di cache, dichiarato dal dump PHPR_DUMP_OPS) vs arith-stride2 ($s=0,$i=15 —
# linee DISTINTE) + CONTROLLO POSITIVO dcnear/dcfar (stesso corpo, maschera
# 1023 vs 4194303: il cronometro DEVE vedere il D-cache). COPIA DICHIARATA di
# wp167-harness/s167-f0ab.sh (manifest s168-stride-copia.diff + copia-gate v2).
# ns/iter=(med raw−floor)/N con N dal sorgente; rumore drop-1.
# rc autoritativo = ab-out/stride.rc; verdetto s168-stride-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
P=~/Claude/php-rust-output/release/phpr
DQ="$H/../wp164-harness/arith-dq.php"
ST="$H/../wp167-harness/arith-stride.php"
ST2="$H/../wp167-harness/arith-stride2.php"
NEAR="$H/arith-dcnear.php"; FAR="$H/arith-dcfar.php"
EMPTY="$H/../wp164-harness/empty.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s168-stride-verdetto.out"; RC="$OUT/stride.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo" >&2; exit 7; }
grep -qi "s168\|s-168" /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock assente" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-stride.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
PM="$(shasum -a 256 "$P" | cut -c1-8)"
[ "$PM" = 092dcff4 ] || { echo "pin!=s166 ($PM)" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$@" "$EMPTY"); b=$(ucpu "$@" "$EMPTY"); c=$(ucpu "$@" "$EMPTY"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
nof(){ awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$1"; }
{
echo "== s168 STRIDE sanatura — pin s166 $PM MISURATO; R=5 interleaved; N dal sorgente: dq=$(nof "$DQ") stride=$(nof "$ST") stride2=$(nof "$ST2") near=$(nof "$NEAR") far=$(nof "$FAR") =="
echo "layout (PHPR_DUMP_OPS, dichiarato): dq \$s=slot0 \$i=slot1 (stessa linea) · stride S-167 \$s=14 \$i=15 (STESSA linea, byte 224-255) · stride2 \$s=0 \$i=15 (linee 0 e 3, DISTINTE)"
echo "sentinella LS: $(pgrep -fl 'rust-analyzer|Antigravity|serena' 2>/dev/null | grep -v pgrep | awk '{print $2}' | sort -u | tr '\n' ' ')"
F=$(floor3 env -u PHPR_REG_LOWER "$P"); echo "floor phpr=$F"
TSV="$OUT/stride-runs.tsv"; : > "$TSV"
for i in 1 2 3 4 5; do
  if [ $((i % 2)) -eq 1 ]; then T1=$(ucpu "$P" "$DQ"); T2=$(ucpu "$P" "$ST"); T3=$(ucpu "$P" "$ST2"); else T3=$(ucpu "$P" "$ST2"); T2=$(ucpu "$P" "$ST"); T1=$(ucpu "$P" "$DQ"); fi
  T4=$(ucpu "$P" "$NEAR"); T5=$(ucpu "$P" "$FAR")
  printf '%s\t%s\t%s\t%s\t%s\n' "$T1" "$T2" "$T3" "$T4" "$T5" >> "$TSV"
  echo "  giro$i: dq=$T1 stride=$T2 stride2=$T3 near=$T4 far=$T5"
done
python3 - "$TSV" "$F" "$(nof "$DQ")" "$(nof "$NEAR")" <<'PY'
import sys
rows = [l.split() for l in open(sys.argv[1])]
f = float(sys.argv[2]); N = float(sys.argv[3]); NN = float(sys.argv[4])
def col(i, n): return sorted((float(r[i])-f)/n*1e9 for r in rows)
def med(v): return v[2]
def dr1(v):
    m = med(v); w = sorted(v, key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
dq, st, st2, near, far = col(0,N), col(1,N), col(2,N), col(3,NN), col(4,NN)
n = max(dr1(dq), dr1(st), dr1(st2)); thr = max(0.5, n)
print(f"dq={med(dq):.2f} stride(S-167, stessa linea)={med(st):.2f} D={med(st)-med(dq):+.2f} · stride2(linee distinte)={med(st2):.2f} D2={med(st2)-med(dq):+.2f} ns/iter (rumore drop-1 dq'={dr1(dq):.2f} st'={dr1(st):.2f} st2'={dr1(st2):.2f}; soglia=max(0,5, rumore)={thr:.2f})")
dpos = med(far)-med(near)
print(f"CONTROLLO POSITIVO D-cache: near={med(near):.1f} far={med(far):.1f} ns/iter D={dpos:+.1f} (rumore near'={dr1(near):.1f} far'={dr1(far):.1f}) -> {'MORDE: il cronometro vede il D-cache' if dpos > 10*max(dr1(near),dr1(far),0.5) else 'NON MORDE: strumento cieco, verdetto stride INDISPONIBILE'}")
d2 = med(st2)-med(dq)
if dpos <= 10*max(dr1(near),dr1(far),0.5): rc = 5
elif abs(d2) <= thr: rc = 0; print(f"VERDETTO: canale pila/D-cache su \\$s/\\$i NON RILEVATO a risoluzione {thr:.2f} ns/iter CON layout verificato (linee distinte) E strumento collaudato dal controllo positivo ⇒ REFUTATO alla risoluzione dichiarata")
else: rc = 4; print(f"VERDETTO: D2={d2:+.2f} > soglia: il layout degli slot MUOVE il tempo — canale pila VIVO (KS-LE-167-1 da rileggere)")
print(f"ESITO rc={rc}"); sys.exit(rc)
PY
prc=$?
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
