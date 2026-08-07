#!/bin/bash
# s107-census-run.sh — S-107 punto 1: census op/bigrammi sui SEI giudici.
# Build census SEPARATA (feature op-census, target phpr-census-target): il
# binario di parità non contiene il hook. Il dump va su file per giudice
# (path assoluto ⇒ append su file, stderr resta pulito). Il census CONTA
# soltanto: nessuna cifra di tempo esce da questa run (REGOLE §3 — le cifre
# vengono solo dall'A/B sul binario di parità).
set -euo pipefail
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
CENSUS_BIN="/Volumes/Extreme Pro/Claude/phpr-census-target/release/phpr"
MICRO="$H/../wp97-harness/micro"
OUT="$H/census-out"
mkdir -p "$OUT"
CATS="arith prop calls str arr re"

echo "== s107 census =="
echo "census_bin=$(shasum -a 256 "$CENSUS_BIN" | cut -c1-16)  # NON è il pin di parità: build strumentata"
echo "parity_pin=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)"
echo "reg_lower=default-ON (env PHPR_REG_LOWER assente)"

for c in $CATS; do
  f="$OUT/census-$c.txt"
  : > "$f"
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$MICRO/$c.php")
  echo "-- $c (N_iter=$N) --"
  PHPR_OP_CENSUS="$f" "$CENSUS_BIN" "$MICRO/$c.php" > /dev/null
  head -2 "$f"
done
echo "== fine census: dump in $OUT/ =="
