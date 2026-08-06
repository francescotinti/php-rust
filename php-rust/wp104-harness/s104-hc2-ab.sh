#!/bin/bash
# s104-hc2-ab.sh — A/B della LEVA H-C2 *DA SOLA* (S-104, ordine WP-105 §2d).
# A = pin S-103 stashato (phpr-s103, f45a5d19: SENZA leva)
# B = build HEAD con la leva (dispose/is_trivial_drop)
# Giudice: prop.php (N EMESSO dal sorgente, KS-GR-105-2), pattern ABAB
# stessa sera, netto pavimenti per-binario. La regola di verdetto NON vive
# qui: è PRE-REGISTRATA in wp104-harness/hc2-criterio-v2.out (commit
# 8dbb16d, PRIMA della misura); questo launcher la CITA soltanto:
#   promozione: Δ ≥ 8 ns/iter con R=5 · Δ ∈ [4,8) ⇒ serve R≥9 e segno
#   stabile su tutte le coppie · Δ < max(banda-layout, rumore ~3) ⇒ caduta
#   (registra e chiudi) · [3,4) ⇒ si registra senza promozione.
set -euo pipefail
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
MICRO="$H/../wp97-harness/micro"
A="${A:-/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s103}"
B="${B:-$HOME/Claude/php-rust-output/release/phpr}"
R="${R:-5}"
OUT="${OUT:-$H/hc2-ab-out}"
mkdir -p "$OUT"

PIN_A_ATTESO="f45a5d199ab34132"
pin_a=$(shasum -a 256 "$A" | cut -c1-16)
pin_b=$(shasum -a 256 "$B" | cut -c1-16)
if [ "$pin_a" != "$PIN_A_ATTESO" ]; then
  echo "VOID: braccio A ha pin $pin_a, atteso $PIN_A_ATTESO" ; exit 66
fi
if [ "$pin_a" = "$pin_b" ]; then
  echo "VOID: A e B sono lo stesso binario ($pin_a)" ; exit 66
fi

# N del giudice EMESSO dal sorgente (mai a memoria)
N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$MICRO/prop.php")
[ -n "$N" ] || { echo "VOID: N non estraibile dal giudice"; exit 66; }

user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
median() { printf '%s\n' "$@" | sort -n | awk '{v[NR]=$1} END{print (NR%2)?v[(NR+1)/2]:(v[NR/2]+v[NR/2+1])/2}'; }

{
echo "hc2-ab: A/B della leva H-C2 da sola — giudice prop, ABAB, R=$R"
echo "grade=MISURA  # il VERDETTO si legge col criterio pre-registrato hc2-criterio-v2.out"
echo "data=$(date '+%Y-%m-%d %H:%M')"
echo "A=$pin_a (pin S-103, senza leva)  B=$pin_b (leva)"
echo "N_giudice=$N (estratto dal sorgente di prop.php)"

# sanity: output byte-identico dei due bracci sul giudice (o il confronto è VOID)
oa=$("$A" "$MICRO/prop.php"); ob=$("$B" "$MICRO/prop.php")
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
  va=$(user_cpu "$A" "$MICRO/prop.php")
  vb=$(user_cpu "$B" "$MICRO/prop.php")
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
echo "verdetto: DA LEGGERE contro hc2-criterio-v2.out (pavimento 4, promozione 8, caduta sotto max(layout,rumore))"
} | tee "$OUT/hc2-ab-$(date '+%H%M').out"
