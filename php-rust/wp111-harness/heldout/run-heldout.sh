#!/bin/bash
# run-heldout.sh — rapporto phpr/oracle sui giudici HELD-OUT (S-111).
# Clone della ricetta di wp97-harness/micro/run-micro.sh: user CPU di
# /usr/bin/time -p AL NETTO del pavimento misurato PER-binario, mediana su R,
# N emesso dal sorgente (KS-GR-105-2). Si lancia SOLO a leva conclusa.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
R="${R:-5}"
CATS="poly err wploop"

user_cpu() { # `time` scrive su stderr: zittire solo lo STDOUT dello script.
  { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'
}
median() { printf '%s\n' "$@" | sort -n | awk '{v[NR]=$1} END{print (NR%2)?v[(NR+1)/2]:(v[NR/2]+v[NR/2+1])/2}'; }
minmax() { printf '%s\n' "$@" | sort -n | awk 'NR==1{lo=$1} {hi=$1} END{printf "%s %s", lo, hi}'; }

echo "heldout: rapporto phpr/oracle sui giudici FUORI dal ciclo di progettazione (S-111)"
echo "grade=VERDICT  # R=$R per lato, mediana e spread pubblicati"
echo "formato=ascii-nudo"
echo "oracle=$("$ORACLE" -r 'echo PHP_VERSION;') jit=$("$ORACLE" -r 'echo ini_get("opcache.jit") ?: "n/d";')"
echo "phpr=$(shasum -a 256 "$PHPR" | cut -c1-16)"
echo "ripetizioni=$R"
echo "metodo=user CPU /usr/bin/time -p, al netto del pavimento per-binario (empty.php dei micro)"

EMPTY="$H/../../wp97-harness/micro/empty.php"
fo=(); fp=()
for i in $(seq 1 "$R"); do
  fo+=("$(user_cpu "$ORACLE" "$EMPTY")")
  fp+=("$(user_cpu "$PHPR" "$EMPTY")")
done
FO=$(median "${fo[@]}"); FP=$(median "${fp[@]}")
echo "pavimento_oracle_s=$FO"
echo "pavimento_phpr_s=$FP"
echo

for c in $CATS; do
  to=(); tp=()
  for i in $(seq 1 "$R"); do
    to+=("$(user_cpu "$ORACLE" "$H/$c.php")")
    tp+=("$(user_cpu "$PHPR" "$H/$c.php")")
  done
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$H/$c.php")
  echo "${c}_N_iter=${N:-nd}"
  MO=$(median "${to[@]}"); MP=$(median "${tp[@]}")
  read -r LO HO <<<"$(minmax "${to[@]}")"
  read -r LP HP <<<"$(minmax "${tp[@]}")"
  python3 - "$c" "$MO" "$MP" "$FO" "$FP" "$LO" "$HO" "$LP" "$HP" <<'PY'
import sys
c, mo, mp, fo, fp, lo, ho, lp, hp = sys.argv[1], *map(float, sys.argv[2:])
no, np_ = mo - fo, mp - fp
print(f"{c}_oracle_netto_s={no:.2f}")
print(f"{c}_phpr_netto_s={np_:.2f}")
print(f"rapporto_{c}={np_/no:.1f}  [derivata: companion {np_:.2f}/{no:.2f}]")
print(f"{c}_spread_oracle_s={ho-lo:.2f}  {c}_spread_phpr_s={hp-lp:.2f}")
PY
done
