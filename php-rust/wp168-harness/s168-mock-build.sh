#!/bin/bash
# s168-mock-build.sh prep <tag> [patch…] | build <tag> | chain <tag…> — build di un MOCK da
# COPIA dell'albero (tree==pin al byte + patch dichiarate `s168-*.patch`
# applicate con `patch -p1`), ricetta del pin (SOURCE_DATE_EPOCH=0
# CARGO_INCREMENTAL=0 cargo build --release -p php-cli) in TARGET DEDICATO
# /private/tmp/s168-mock-tgt (mai php-rust-output). `prep` è SINCRONO (copia +
# patch: l'albero di lavoro può essere editato subito dopo senza contaminare
# la copia); `build` va detached. Esiti: bin /private/tmp/s168-mock-bin/phpr-<tag>,
# hash in ab-out/build-<tag>.out, rc in ab-out/build-<tag>.rc, .done a fine.
# La copia-sorgente si RIMUOVE a fine build (lezione ENOSPC S-167); il target
# resta per il riuso tra mock e lo rimuove l'epilogo di sessione.
# EMENDA (m0 hash 36d73812 != pin: i path della copia entrano nel binario via
# file!()/panic-location): la copia vive a un path FISSO /private/tmp/s168-mock-src
# per TUTTI i mock, così m0..m123 differiscono SOLO per le patch; `chain` fa
# prep+build in sequenza (patch = s168-<tag>.patch, nessuna per m0).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
MODE="${1:?prep|build|chain}"; TAG="${2:?tag}"; shift 2
if [ "$MODE" = chain ]; then
  SELF="$0"; for t in "$TAG" "$@"; do
    if [ "$t" = m0 ]; then "$SELF" prep "$t" || exit $?; else "$SELF" prep "$t" "$(dirname "$SELF")/s168-$t.patch" || exit $?; fi
    "$SELF" build "$t" || exit $?
  done; exit 0
fi
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp168-harness"; OUT="$H/ab-out"; mkdir -p "$OUT" /private/tmp/s168-mock-bin
SRC="/private/tmp/s168-mock-src"
RC="$OUT/build-$TAG.rc"; LOG="$OUT/build-$TAG.log"; RES="$OUT/build-$TAG.out"; DONE="$OUT/build-$TAG.done"
if [ "$MODE" = prep ]; then
  rm -rf "$SRC"; mkdir -p "$SRC/php-rust"; rm -f "$RC" "$DONE"
  [ -z "$(cd "$REPO" && git status --porcelain crates)" ] || { echo "STOP: albero crates SPORCO — la copia deve nascere da tree==pin"; exit 3; }
  for f in crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo; do
    cp -R "$REPO/$f" "$SRC/php-rust/" || { echo "copia fallita ($f)"; exit 7; }
  done
  { echo "== prep mock $TAG $(date '+%F %T') — sorgente @ $(cd "$REPO" && git rev-parse --short HEAD), patch: $*"
    for p in "$@"; do
      patch -p1 -d "$SRC/php-rust" -N < "$p" || { echo "PATCH $p FALLITA"; echo 7 > "$RC"; exit 7; }
    done
  } > "$RES" 2>&1 || exit $?
  echo "prep $TAG ok (copia in $SRC, patch: $*)"; exit 0
fi
[ -d "$SRC/php-rust/crates" ] || { echo "copia assente: prep prima"; echo 7 > "$RC"; touch "$DONE"; exit 7; }
( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/private/tmp/s168-mock-tgt \
  cargo build --release -p php-cli ) > "$LOG" 2>&1
brc=$?
if [ "$brc" -ne 0 ]; then echo "BUILD FALLITA rc=$brc (log $LOG)" >> "$RES"; echo "$brc" > "$RC"; rm -rf "$SRC"; touch "$DONE"; exit "$brc"; fi
cp /private/tmp/s168-mock-tgt/release/phpr "/private/tmp/s168-mock-bin/phpr-$TAG" || { echo 7 > "$RC"; rm -rf "$SRC"; touch "$DONE"; exit 7; }
echo "bin=/private/tmp/s168-mock-bin/phpr-$TAG hash=$(shasum -a 256 "/private/tmp/s168-mock-bin/phpr-$TAG" | cut -c1-16) fine $(date '+%F %T')" >> "$RES"
rm -rf "$SRC"; echo 0 > "$RC"; touch "$DONE"; exit 0
