#!/bin/bash
# s109-census-run.sh — S-109 punto 3a: census op/bigrammi TERZO GIRO sul pin
# nuovo (la classifica dei bigrammi residui DOPO il lotto-2 S-108 e' un'altra
# classifica). Build census SEPARATA (feature op-census, phpr-census-target):
# il binario di parita' non contiene il hook. Il census CONTA soltanto:
# nessuna cifra di tempo esce da questa run (REGOLE par.3).
set -euo pipefail
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="$H/.."
CENSUS_TARGET="/Volumes/Extreme Pro/Claude/phpr-census-target"
CENSUS_BIN="$CENSUS_TARGET/release/phpr"
MICRO="$H/../wp97-harness/micro"
OUT="$H/census-out"
mkdir -p "$OUT"
CATS="arith prop calls str arr re"

echo "== s109 census build (feature op-census, target separato) =="
( cd "$REPO" && CARGO_TARGET_DIR="$CENSUS_TARGET" cargo build --release -p php-cli --features php-runtime/op-census ) \
  > "$OUT/census-build.log" 2>&1
echo "build rc=$?"

echo "== s109 census run =="
echo "census_bin=$(shasum -a 256 "$CENSUS_BIN" | cut -c1-16)  # NON e' il pin di parita': build strumentata"
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
