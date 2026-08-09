#!/bin/bash
# s121-census-st1.sh — MECCANISMO L-ST1 (criterio s121-criterio-st1.md p.2):
# build census (HEAD + census-clite.patch wp119 + st1.patch) nel target
# SEPARATO phpr-census-target (MAI il pin, MAI il target canonico); run R=2
# su empty + le sei categorie; alloc/iter netto = (galloc_cat − galloc_empty)/N.
# SOLO CONTEGGI: nessuna cifra di tempo esce da questa run (REGOLE §3).
# A fine run il tree si RIPRISTINA (niente rebuild del canonico: mai toccato).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp121-harness"
H119="$REPO/wp119-harness"
MICRO="$REPO/wp97-harness/micro"
OUT="$H/census-out"; mkdir -p "$OUT"
CTD="/Volumes/Extreme Pro/Claude/phpr-census-target"
CENSUS_BIN="$CTD/release/phpr"
R=2
CATS="empty arith prop calls str arr re"
fail(){ echo "$1"; echo 1 > "$OUT/census.rc"; git -C "$REPO" checkout -- crates/ 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet -- crates/ || fail "PRE: crates/ sporco — STOP"
git apply "$H119/census-clite.patch" || fail "census-clite.patch NON applica"
git apply "$H/st1.patch" || { git checkout -- crates/; fail "st1.patch NON applica sopra il census"; }

CARGO_TARGET_DIR="$CTD" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli --features mem-census,zval-census \
  > "$OUT/build-census.log" 2>&1
brc=$?; echo "$brc" > "$OUT/build-census.rc"
[ "$brc" = 0 ] || fail "build census rc=$brc"

{
  echo "census_bin=$(shasum -a 256 "$CENSUS_BIN" | cut -c1-16)  # build strumentata, NON il pin"
  echo "parity_pin=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)"
  echo "head=$(git -C "$REPO" rev-parse --short HEAD)"
  echo "patches=census-clite(wp119)+st1"
  echo "epoch=$(date +%s)"
} > "$OUT/identity.txt"

for c in $CATS; do
  for r in $(seq 1 $R); do
    f="$OUT/phpr-$c-r$r.census"; rm -f "$f"
    PHPR_MEM_CENSUS="$f" MIMALLOC_PURGE_DELAY=0 "$CENSUS_BIN" "$MICRO/$c.php" > /dev/null 2>"$OUT/phpr-$c-r$r.err"
    echo "rc=$?" >> "$OUT/phpr-$c-r$r.err"
  done
done

git checkout -- crates/ || fail "ripristino tree fallito"
echo 0 > "$OUT/census.rc"
echo done > "$OUT/run.done"
