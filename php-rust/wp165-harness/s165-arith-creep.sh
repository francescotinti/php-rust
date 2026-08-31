#!/bin/bash
# s165-arith-creep.sh — az.rev. S-164 #1: nota aperta «creep arith +0,6 ns/iter
# da s158». CRITERIO PRE-registrato (in testa, prima del run):
# bracci = A phpr-s158 (92b0aea3, stash pinnato) vs B phpr-s165 (1fd8757d,
# pin corrente; proxy DICHIARATO di s163: guardia arith mc1dr5 D=−0,9/+0,2 ≈ 0
# sugli stessi binari); giudice = wp164-harness/arith-dq.php N=250.000.000
# (de-quantizzato S-164: tick 0,04 ns/iter); R=5 interleaved, ns/iter =
# (med raw − floor)/N; SOGLIA = max(0,5 ns/iter, rumore drop-1) — la taglia
# del creep sospetto e' 0,6. Esiti: Δ=A−B ≤ −soglia ⇒ CREEP CONFERMATO con
# cifra (s158 piu' veloce); |Δ| < soglia ⇒ creep REFUTATO alla risoluzione
# 0,5; Δ ≥ +soglia ⇒ miglioramento da s158 (nota chiusa in direzione
# opposta). In ogni caso la NOTA si CHIUDE. rc autoritativo = ab-out/arith-creep.rc
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s158"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s165"
DRV="$H/../wp164-harness/arith-dq.php"
OUT="$H/ab-out"; VERD="$H/s165-arith-creep-verdetto.out"; RC="$OUT/arith-creep.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE" >&2; exit 7; }
grep -qi "s165\|s-165" /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock assente" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-creep.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
AM="$(shasum -a 256 "$A" | cut -c1-8)"; BM="$(shasum -a 256 "$B" | cut -c1-8)"
{ [ "$AM" = 92b0aea3 ] && [ "$BM" = 1fd8757d ]; } || { echo "hash bracci errati $AM/$BM" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3(){ local a b c; a=$(ucpu "$1" "$H/empty.php"); b=$(ucpu "$1" "$H/empty.php"); c=$(ucpu "$1" "$H/empty.php"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
{
echo "== s165 arith-creep s158↔s165 (criterio in testa allo script; A=$AM B=$BM MISURATI; N=250000000 R=5) =="
"$A" "$DRV" > "$OUT/creep-A.out" 2>&1; "$B" "$DRV" > "$OUT/creep-B.out" 2>&1
diff -q "$OUT/creep-A.out" "$OUT/creep-B.out" > /dev/null || { echo "output DIVERGE"; echo 2 > "$RC"; exit 2; }
FA=$(floor3 "$A"); FB=$(floor3 "$B"); echo "floor_A=$FA floor_B=$FB"
TSV="$OUT/creep-runs.tsv"; : > "$TSV"
for i in 1 2 3 4 5; do
  if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$A" "$DRV"); TB=$(ucpu "$B" "$DRV"); ord=AB; else TB=$(ucpu "$B" "$DRV"); TA=$(ucpu "$A" "$DRV"); ord=BA; fi
  printf '%s\t%s\t%s\t%s\n' "$TA" "$TB" "$FA" "$FB" >> "$TSV"
  echo "  coppia$i [$ord]: rawA=$TA rawB=$TB"
done
python3 - "$TSV" <<'PY'
import sys
rows = [l.split() for l in open(sys.argv[1])]
N = 250000000.0
na = [(float(t[0])-float(t[2]))/N*1e9 for t in rows]
nb = [(float(t[1])-float(t[3]))/N*1e9 for t in rows]
def med(v): s=sorted(v); return s[2]
def trange(v):
    m=med(v); w=sorted(v, key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
ma,mb=med(na),med(nb); d=ma-mb; noise=max(trange(na),trange(nb))
thr=max(0.5, noise)
print(f"A(s158)={ma:.2f} B(s165)={mb:.2f} ns/iter Delta=A-B={d:+.2f} soglia={thr:.2f} (rumore drop-1 A'={trange(na):.2f} B'={trange(nb):.2f})")
if d <= -thr: print(f"VERDETTO: CREEP CONFERMATO da s158, cifra {-d:.2f} ns/iter — nota CHIUSA con cifra"); sys.exit(3)
if d >= thr: print("VERDETTO: s165 PIU' VELOCE di s158 oltre soglia — nota chiusa in direzione opposta"); sys.exit(0)
print("VERDETTO: creep REFUTATO alla risoluzione 0,5 ns/iter — nota CHIUSA")
sys.exit(0)
PY
prc=$?; echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
