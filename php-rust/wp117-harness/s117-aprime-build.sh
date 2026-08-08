#!/bin/bash
# s117-aprime-build.sh — spike A′ stadio-1 (criterio PRE s117-criterio-aprime.md §2):
# 2 build a sorgente invariato, target FRESCO ciascuna, CARGO_INCREMENTAL=0,
# target DEDICATO (il pin in ~/Claude/php-rust-output NON si tocca).
# Esito: hash-16 di phpr e phpt-runner per build, rc per build in FILE,
# verdetto determinismo appeso DALLO script. Le build sono sequenziali.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
TGT="/Volumes/Extreme Pro/Claude/phpr-s117-aprime-target"
OUT="$H/aprime-out"; mkdir -p "$OUT"
VERD="$H/s117-aprime-verdetto.out"

cd "$SRC" || { echo 4 > "$OUT/build-rc"; exit 4; }
SRCH=$(git rev-parse --short HEAD)

for i in 1 2; do
  rm -rf "$TGT"
  CARGO_TARGET_DIR="$TGT" CARGO_INCREMENTAL=0 cargo build --release \
    > "$OUT/build$i.log" 2>&1
  rc=$?
  echo "$rc" > "$OUT/build$i.rc"
  if [ "$rc" != 0 ]; then
    echo "build$i FALLITA rc=$rc (log: aprime-out/build$i.log)" | tee -a "$VERD"
    echo "$rc" > "$OUT/build-rc"; exit "$rc"
  fi
  for b in phpr phpt-runner; do
    shasum -a 256 "$TGT/release/$b" | cut -c1-16 > "$OUT/build$i-$b.hash"
  done
  echo "build$i: rc=$rc phpr=$(cat "$OUT/build$i-phpr.hash") phpt-runner=$(cat "$OUT/build$i-phpt-runner.hash")"
done

DET=OK
for b in phpr phpt-runner; do
  if ! diff -q "$OUT/build1-$b.hash" "$OUT/build2-$b.hash" > /dev/null; then DET=ROTTO; fi
done
if [ "$DET" = OK ]; then RCT=0; else RCT=1; fi
echo "$RCT" > "$OUT/build-rc"
{
  echo "# S-117 spike A′ stadio-1 — determinismo build ×2 (sorgente $SRCH, lto=fat cgu=1, target fresco, INCREMENTAL=0)"
  echo "build1: rc=$(cat "$OUT/build1.rc") phpr=$(cat "$OUT/build1-phpr.hash") runner=$(cat "$OUT/build1-phpt-runner.hash")"
  echo "build2: rc=$(cat "$OUT/build2.rc") phpr=$(cat "$OUT/build2-phpr.hash") runner=$(cat "$OUT/build2-phpt-runner.hash")"
  echo "determinismo=$DET (rc=$RCT in aprime-out/build-rc)"
} >> "$VERD"
echo "determinismo=$DET"
exit "$RCT"
