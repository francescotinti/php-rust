#!/bin/bash
# s172-leva-build.sh <COMMIT> <BRACCIO> — build di un braccio candidato (L-SL2, criterio p.3)
# da un COMMIT dichiarato (git archive → /private/tmp/s172-src-<BRACCIO>: il repo può
# intanto avanzare) con la ricetta del pin (SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0
# cargo build --release -p php-cli) in TARGET DEDICATO /private/tmp/s172-leva-tgt
# (condiviso tra B e C: build SEQUENZIALI; mai php-rust-output). COPIA DICHIARATA di
# wp171-harness/s171-leva-build.sh; adattamenti: sorgente = archivio del commit invece
# dell'albero di lavoro (la guardia «crates puliti» diventa «commit esistente»),
# nome braccio nei file di esito. Esiti: ab-out/build-<BRACCIO>.out/.rc/.done.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
C="${1:?COMMIT}"; B="${2:?BRACCIO}"
H="$REPO/wp172-harness"; OUT="$H/ab-out"; mkdir -p "$OUT"
TGT=/private/tmp/s172-leva-tgt; SRC="/private/tmp/s172-src-$B"
RC="$OUT/build-$B.rc"; LOG="$OUT/build-$B.log"; RES="$OUT/build-$B.out"; DONE="$OUT/build-$B.done"
rm -f "$RC" "$DONE"
cd "$REPO" || { echo 4 > "$RC"; touch "$DONE"; exit 4; }
SHA=$(git rev-parse --verify "$C^{commit}" 2>/dev/null) || { echo "STOP: commit $C inesistente" > "$RES"; echo 3 > "$RC"; touch "$DONE"; exit 3; }
echo "== build $B $(date '+%F %T') — sorgente @ ${SHA:0:12} (archivio) target $TGT" > "$RES"
rm -rf "$SRC"; mkdir -p "$SRC"
git archive "$SHA" crates Cargo.toml Cargo.lock rust-toolchain.toml .cargo 2>/dev/null | tar -x -C "$SRC" \
  || { echo "archivio FALLITO" >> "$RES"; echo 7 > "$RC"; touch "$DONE"; exit 7; }
[ -s "$SRC/Cargo.toml" ] || { echo "archivio VUOTO" >> "$RES"; echo 7 > "$RC"; touch "$DONE"; exit 7; }
( cd "$SRC" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" cargo build --release -p php-cli ) > "$LOG" 2>&1
brc=$?
if [ "$brc" -ne 0 ]; then echo "BUILD FALLITA rc=$brc (log $LOG)" >> "$RES"; echo "$brc" > "$RC"; touch "$DONE"; exit "$brc"; fi
[ -s "$TGT/release/phpr" ] || { echo "binario ASSENTE dopo build" >> "$RES"; echo 7 > "$RC"; touch "$DONE"; exit 7; }
cp "$TGT/release/phpr" "$OUT/phpr-$B"
echo "bin=$OUT/phpr-$B hash=$(shasum -a 256 "$OUT/phpr-$B" | cut -c1-16) fine $(date '+%F %T') Data=$(df -k /System/Volumes/Data | awk 'NR>1{printf "%.1fG", $4/1048576}')" >> "$RES"
rm -rf "$SRC"
echo 0 > "$RC"; touch "$DONE"; exit 0
