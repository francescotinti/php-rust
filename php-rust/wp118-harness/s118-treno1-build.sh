#!/bin/bash
# s118-treno1-build.sh — criterio s118-criterio-treno1.md p.1-2: candidato B
# (HEAD + hp1-composto.patch) nel target CANONICO con ricetta A′; admission
# (parità output 6 giudici + dump INTERO byte-id A↔B); conserva il candidato;
# poi RIPRISTINA il tree e il release al pin (build ricetta, H1 al byte, pena
# rc!=0). Collaudo-nell'atto: admission fallita => niente stash del candidato.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp118-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s117"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
M="$REPO/wp97-harness/micro"
OUT="$H/treno-out"; mkdir -p "$OUT"
VERD="$H/s118-treno1-verdetto.out"
PIN_EXP="1656580e4e590e20"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
fail(){ note "$1"; echo 1 > "$OUT/build.rc"; exit 1; }

cd "$REPO" || exit 2
H0=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H0" = "$PIN_EXP" ] || fail "PRE: release/phpr=$H0 != pin $PIN_EXP — STOP"
git diff --quiet -- crates/ || fail "PRE: crates/ sporco — STOP"

# --- candidato: patch + build ricetta nel target canonico ---
git apply "$H/hp1-composto.patch" || fail "patch NON applica"
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build-cand.log" 2>&1
brc=$?; echo "$brc" > "$OUT/build-cand.rc"
if [ "$brc" != 0 ]; then git checkout -- crates/; fail "build candidato rc=$brc (tree ripristinato)"; fi
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
note "candidato B = $HB (target canonico, ricetta A′; A = pin $PIN_EXP)"

# --- admission: parità output + dump INTERO byte-id sui 6 giudici ---
ADM=0
for C in arith prop calls str arr re; do
  "$A" "$M/$C.php" > "$OUT/adm-$C-A.out" 2>&1
  "$BIN" "$M/$C.php" > "$OUT/adm-$C-B.out" 2>&1
  cmp -s "$OUT/adm-$C-A.out" "$OUT/adm-$C-B.out" || { note "admission $C: OUTPUT DIVERGE"; ADM=1; }
  PHPR_DUMP_OPS=1 "$A" "$M/$C.php" 2> "$OUT/adm-$C-A.dump" > /dev/null
  PHPR_DUMP_OPS=1 "$BIN" "$M/$C.php" 2> "$OUT/adm-$C-B.dump" > /dev/null
  cmp -s "$OUT/adm-$C-A.dump" "$OUT/adm-$C-B.dump" || { note "admission $C: DUMP DIVERGE (emissione NON invariata)"; ADM=1; }
done
echo "$ADM" > "$OUT/admission.rc"
if [ "$ADM" != 0 ]; then
  git checkout -- crates/
  SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build-restore.log" 2>&1
  fail "admission VIOLATA (rc=1 da treno-out/admission.rc) — candidato NON conservato, tree ripristinato"
fi
note "admission: 6/6 parità output + 6/6 dump INTERO byte-id (rc=0 da treno-out/admission.rc)"
cp "$BIN" "$STASH/phpr-s118-treno1"
HS=$(shasum -a 256 "$STASH/phpr-s118-treno1" | cut -c1-16)
[ "$HB" = "$HS" ] || fail "stash candidato hash diverso ($HB vs $HS)"
note "candidato CONSERVATO: phpr-s118-treno1 ($HB)"

# --- ripristino: tree al pin + release al pin AL BYTE ---
git checkout -- crates/ || fail "checkout ripristino fallito"
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build-restore.log" 2>&1
rrc=$?; echo "$rrc" > "$OUT/build-restore.rc"
[ "$rrc" = 0 ] || fail "build ripristino rc=$rrc"
H1=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H1" = "$PIN_EXP" ] || fail "RIPRISTINO NON AL BYTE: release=$H1 != $PIN_EXP — STOP (determinismo violato)"
note "ripristino: release/phpr == pin $PIN_EXP AL BYTE; A/B pronte su stash A=phpr-s117 B=phpr-s118-treno1"
echo 0 > "$OUT/build.rc"
