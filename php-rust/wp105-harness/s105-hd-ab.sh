#!/bin/bash
# s105-hd-ab.sh — A/B della LEVA H-D args *DA SOLA* (S-105, ordine WP-106 §1c).
# A = pin S-104 stashato (phpr-s104, 86a50d1c: SENZA leva)
# B = build HEAD con la leva (ArgsBuf SmallVec inline-2)
# Giudice: calls.php (N EMESSO dal sorgente, KS-GR-105-2), pattern ABAB
# stessa sera, netto pavimenti per-binario. La regola di verdetto NON vive
# qui: è PRE-REGISTRATA in wp105-harness/hd-args-criterio.out (commit
# 11cc23a, PRIMA della misura); questo launcher la CITA soltanto:
#   promozione: Δ ≥ 6 ns/iter con R=5 segno 5/5 E census→0,0000 ·
#   Δ ∈ [4,6) ⇒ R≥9 e segno stabile · pavimento 4 (si registra) ·
#   caduta sotto max(rumore ~3, banda-layout 0,67 [N=1]).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
MICRO="$H/../wp97-harness/micro"
A="${A:-/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s104}"
B="${B:-$HOME/Claude/php-rust-output/release/phpr}"
R="${R:-5}"
OUT="${OUT:-$H/hd-ab-out}"
mkdir -p "$OUT"

PIN_A_ATTESO="86a50d1c01c6f45a"
pin_a=$(shasum -a 256 "$A" | cut -c1-16)
pin_b=$(shasum -a 256 "$B" | cut -c1-16)
if [ "$pin_a" != "$PIN_A_ATTESO" ]; then
  echo "VOID: braccio A ha pin $pin_a, atteso $PIN_A_ATTESO" ; exit 66
fi
if [ "$pin_a" = "$pin_b" ]; then
  echo "VOID: A e B sono lo stesso binario ($pin_a)" ; exit 66
fi

# N del giudice EMESSO dal sorgente (mai a memoria)
N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$MICRO/calls.php")
[ -n "$N" ] || { echo "VOID: N non estraibile dal giudice"; exit 66; }

user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
median() { printf '%s\n' "$@" | sort -n | awk '{v[NR]=$1} END{print (NR%2)?v[(NR+1)/2]:(v[NR/2]+v[NR/2+1])/2}'; }

{
echo "hd-ab: A/B della leva H-D args da sola — giudice calls, ABAB, R=$R"
echo "grade=MISURA  # il VERDETTO si legge col criterio pre-registrato hd-args-criterio.out"
echo "data=$(date '+%Y-%m-%d %H:%M')"
echo "A=$pin_a (pin S-104, senza leva)  B=$pin_b (leva ArgsBuf)"
echo "N_giudice=$N (estratto dal sorgente di calls.php)"

# sanity: output byte-identico dei due bracci sul giudice (o il confronto è VOID)
oa=$("$A" "$MICRO/calls.php"); ob=$("$B" "$MICRO/calls.php")
if [ "$oa" != "$ob" ]; then echo "VOID: output divergente A='$oa' B='$ob'"; exit 66; fi
echo "sanity_output=identico ($oa)"

# pavimenti per-binario (R misure, mediana)
fa=(); fb=()
for i in $(seq 1 "$R"); do
  fa+=("$(user_cpu "$A" "$MICRO/empty.php")")
  fb+=("$(user_cpu "$B" "$MICRO/empty.php")")
done
FA=$(median "${fa[@]}"); FB=$(median "${fb[@]}")
echo "pavimento_A_s=$FA  pavimento_B_s=$FB"

# ABAB: R coppie alternate sul giudice
ta=(); tb=()
for i in $(seq 1 "$R"); do
  va=$(user_cpu "$A" "$MICRO/calls.php")
  vb=$(user_cpu "$B" "$MICRO/calls.php")
  ta+=("$va"); tb+=("$vb")
  echo "coppia_$i: A=$va B=$vb"
done
MA=$(median "${ta[@]}"); MB=$(median "${tb[@]}")

python3 - "$MA" "$MB" "$FA" "$FB" "$N" "${ta[@]}" "${tb[@]}" <<PY
import sys
ma, mb, fa, fb = map(float, sys.argv[1:5])
n = int(sys.argv[5]); rest = list(map(float, sys.argv[6:]))
r = len(rest) // 2
ta, tb = rest[:r], rest[r:]
na, nb = ma - fa, mb - fb
d_ns = (na - nb) / n * 1e9
print(f"netto_A_s={na:.2f}  netto_B_s={nb:.2f}")
print(f"ns_iter_A={na/n*1e9:.2f}  ns_iter_B={nb/n*1e9:.2f}")
print(f"DELTA_ns_iter={d_ns:.2f}  # >0 = la leva GUADAGNA")
sign = sum(1 for x, y in zip(ta, tb) if x > y)
print(f"segno: B<A in {sign}/{r} coppie")
sa = max(ta) - min(ta); sb = max(tb) - min(tb)
print(f"spread_A_s={sa:.2f}  spread_B_s={sb:.2f}  # rumore run-to-run della sera")
print(f"rumore_ns_iter=max({sa/n*1e9:.2f},{sb/n*1e9:.2f})")
PY
echo "verdetto: DA LEGGERE contro hd-args-criterio.out (co-primari T e C; pavimento 4; caduta sotto max(layout,rumore))"
} | tee "$OUT/hd-ab-R$R-$(date '+%H%M').out"
