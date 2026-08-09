#!/bin/bash
# s120-re1-build.sh — criterio s120-criterio-re1.md p.1/6: candidato L-RE1
# (HEAD + re1.patch) nel target CANONICO con ricetta A′; admission (parità
# output 6 giudici + dump INTERO byte-id, deroga p.6 dichiarata nel criterio);
# conserva il candidato; poi RIPRISTINA tree e release al pin AL BYTE.
# Collaudo-nell'atto: admission fallita => niente stash.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp120-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s119"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
M="$REPO/wp97-harness/micro"
OUT="$H/re1-out"; mkdir -p "$OUT"
VERD="$H/s120-re1-verdetto.out"
PIN_EXP="350582e55d21533d"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
fail(){ note "$1"; echo 1 > "$OUT/build.rc"; exit 1; }

cd "$REPO" || exit 2
H0=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H0" = "$PIN_EXP" ] || fail "PRE: release/phpr=$H0 != pin $PIN_EXP — STOP"
git diff --quiet -- crates/ || fail "PRE: crates/ sporco — STOP"

git apply "$H/re1.patch" || fail "patch NON applica"
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build-cand.log" 2>&1
brc=$?; echo "$brc" > "$OUT/build-cand.rc"
if [ "$brc" != 0 ]; then git checkout -- crates/; fail "build candidato rc=$brc (tree ripristinato)"; fi
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
note "candidato L-RE1 = $HB (target canonico, ricetta A′; A = pin $PIN_EXP)"

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
  fail "admission VIOLATA (rc=1 da re1-out/admission.rc) — candidato NON conservato, tree ripristinato"
fi
note "admission: 6/6 parità output + 6/6 dump INTERO byte-id (rc=0 da re1-out/admission.rc)"
cp "$BIN" "$STASH/phpr-s120-re1"
HS=$(shasum -a 256 "$STASH/phpr-s120-re1" | cut -c1-16)
[ "$HB" = "$HS" ] || fail "stash candidato hash diverso ($HB vs $HS)"
note "candidato CONSERVATO: phpr-s120-re1 ($HB)"

git checkout -- crates/ || fail "checkout ripristino fallito"
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build-restore.log" 2>&1
rrc=$?; echo "$rrc" > "$OUT/build-restore.rc"
[ "$rrc" = 0 ] || fail "build ripristino rc=$rrc"
H1=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H1" = "$PIN_EXP" ] || fail "RIPRISTINO NON AL BYTE: release=$H1 != $PIN_EXP — STOP (determinismo violato)"
note "ripristino: release/phpr == pin $PIN_EXP AL BYTE; A/B pronte su stash A=phpr-s119 B=phpr-s120-re1"
echo 0 > "$OUT/build.rc"
