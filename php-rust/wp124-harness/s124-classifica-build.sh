#!/bin/bash
# s124-classifica-build.sh — criterio s124-criterio-phpstr.md p.3 (admission):
# build census SOLO mem-census (fusione VIVA) su HEAD CON la patch single-alloc,
# stesso target SEPARATO e stessa ricetta di s123-classifica-build.sh.
# Admission parità output 7/7 vs pin stash (i micro sono deterministici: la
# patch non cambia la semantica, quindi l'output DEVE combaciare col pin).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp124-harness"
H119="$REPO/wp119-harness"
MICRO="$REPO/wp97-harness/micro"
OUT="$H/classifica-out"; mkdir -p "$OUT"
CTD="/Volumes/Extreme Pro/Claude/phpr-census-target"
CENSUS_BIN="$CTD/release/phpr"
PIN="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s120-re1"
VERD="$H/s124-classifica-verdetto.out"
fail(){ echo "$1" | tee -a "$VERD"; echo 1 > "$OUT/build.rc"; git -C "$REPO" checkout -- . 2>/dev/null; exit 1; }

cd "$REPO" || exit 2
git diff --quiet || fail "PRE: tree sporco — STOP"
[ "$(shasum -a 256 "$PIN" | cut -c1-8)" = "885d2c64" ] || fail "pin stash != 885d2c64 — STOP"
git apply "$H119/census-clite.patch" || fail "census-clite.patch NON applica"

CARGO_TARGET_DIR="$CTD" SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
  cargo build --release -p php-cli --features mem-census \
  > "$OUT/build-census.log" 2>&1
brc=$?
[ "$brc" = 0 ] || fail "build census rc=$brc"

git checkout -- . || fail "ripristino tree fallito"
git diff --quiet || fail "POST: tree NON pulito dopo il revert — STOP"

# admission: parità output 7/7 census-bin (patch) vs PIN (pre-patch)
for c in empty arith prop calls str arr re; do
  "$CENSUS_BIN" "$MICRO/$c.php" > "$OUT/adm-$c-census.out" 2>&1
  "$PIN"        "$MICRO/$c.php" > "$OUT/adm-$c-pin.out" 2>&1
  diff -q "$OUT/adm-$c-census.out" "$OUT/adm-$c-pin.out" > /dev/null \
    || fail "admission: output DIVERGE su $c — build SCARTATA"
done

{
  echo "== s124 classifica build: census_bin=$(shasum -a 256 "$CENSUS_BIN" | cut -c1-16) (SOLO mem-census, fusione VIVA, head CON patch single-alloc)"
  echo "   head=$(git -C "$REPO" rev-parse --short HEAD) patch=census-clite(wp119) admission=7/7 pin=885d2c64"
} >> "$VERD"
echo 0 > "$OUT/build.rc"
