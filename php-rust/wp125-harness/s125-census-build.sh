#!/bin/bash
# s125-census-build.sh — criterio s125-criterio-census.md p.2: due build census
# dallo STESSO head (A=mem-census fusa; B=mem-census,zval-census unfused),
# target SEPARATI, admission parità 7/7 vs pin s124. Tree ripristinato al byte.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp125-harness"
H119="$REPO/wp119-harness"
MICRO="$REPO/wp97-harness/micro"
OUT="$H/census-out"; mkdir -p "$OUT"
CTA="/Volumes/Extreme Pro/Claude/phpr-census-target"
CTB="/Volumes/Extreme Pro/Claude/phpr-census-zv-target"
PIN="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s124"
VERD="$H/s125-census-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/build.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
[ "$(shasum -a 256 "$PIN" | cut -c1-16)" = "c5ba2573a23adf69" ] || fail "pin stash != c5ba2573 — STOP"
git apply "$H119/census-clite.patch" || fail "census-clite.patch NON applica"

CARGO_TARGET_DIR="$CTA" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli --features mem-census \
  > "$OUT/build-A.log" 2>&1 || fail "build A (mem-census) FALLITA"
CARGO_TARGET_DIR="$CTB" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli --features mem-census,zval-census \
  > "$OUT/build-B.log" 2>&1 || fail "build B (mem-census,zval-census) FALLITA — debito apparato dichiarato (criterio p.2)"

git checkout -- . || fail "ripristino tree fallito"
git diff --quiet || fail "POST: tree NON pulito dopo il revert — STOP"

for side in A B; do
  BIN="$([ "$side" = A ] && echo "$CTA" || echo "$CTB")/release/phpr"
  for c in empty arith prop calls str arr re; do
    "$BIN" "$MICRO/$c.php" > "$OUT/adm-$c-$side.out" 2>&1
    "$PIN" "$MICRO/$c.php" > "$OUT/adm-$c-pin.out" 2>&1
    diff -q "$OUT/adm-$c-$side.out" "$OUT/adm-$c-pin.out" > /dev/null \
      || fail "admission $side: output DIVERGE su $c — build SCARTATA"
  done
done

{
  echo "== s125 census ±zval STESSO head: A=$(shasum -a 256 "$CTA/release/phpr" | cut -c1-16) (mem-census, fusa) B=$(shasum -a 256 "$CTB/release/phpr" | cut -c1-16) (mem-census+zval-census, unfused)"
  echo "   head=$(git -C "$REPO" rev-parse --short HEAD) crates@fb140d1 patch=census-clite(wp119) admission=7/7+7/7 pin=c5ba2573"
} >> "$VERD"
echo 0 > "$OUT/build.rc"
