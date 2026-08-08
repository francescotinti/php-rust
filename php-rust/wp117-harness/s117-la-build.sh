#!/bin/bash
# s117-la-build.sh — criterio s117-criterio-la.md p.1: candidato L-A sotto A′.
# Tree pulito → cherry-pick -n 2c18b2e (MAI checkout parziale) → build ricetta
# A′ → conserva phpr-s117-la → reset --hard HEAD → tree pulito verificato.
# Poi ADMISSION p.6 (5 casi: parità output+rc, dump {main} A↔B, bigramma in A).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
export SOURCE_DATE_EPOCH=0
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
ROOT="/Volumes/Extreme Pro/Claude/php-rust-experiment"
SRC="$ROOT/php-rust"
TGT="/Volumes/Extreme Pro/Claude/phpr-s117-aprime-target"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s117-aprime"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s117-la"
CASES_DIR="$H/../wp115-harness/admission-cases"
OUT="$H/la-out"; mkdir -p "$OUT"
VERD="$H/s117-la-verdetto.out"
note(){ echo "$1"; echo "$1" >> "$VERD"; }

cd "$ROOT" || exit 4
git diff --quiet && git diff --cached --quiet || { note "la-build: tree NON pulito PRIMA del cherry-pick — STOP"; echo 4 > "$OUT/build.rc"; exit 4; }
git cherry-pick -n 2c18b2e || { note "la-build: cherry-pick -n 2c18b2e FALLITO"; git cherry-pick --abort 2>/dev/null; git reset --hard HEAD >/dev/null; echo 4 > "$OUT/build.rc"; exit 4; }
cd "$SRC"
CARGO_TARGET_DIR="$TGT" CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build.rc"
if [ "$rc" != 0 ]; then note "la-build: build FALLITA rc=$rc"; cd "$ROOT"; git reset --hard HEAD >/dev/null; exit 4; fi
cp "$TGT/release/phpr" "$B"
HB=$(shasum -a 256 "$B" | cut -c1-8); echo "$HB" > "$OUT/cand.hash"
cd "$ROOT"; git reset --hard HEAD >/dev/null
git diff --quiet -- '*.rs' && ST=pulito || ST=SPORCO
[ "$ST" = pulito ] || { note "la-build: tree SPORCO dopo reset — STOP"; echo 4 > "$OUT/build.rc"; exit 4; }
note "la-build: rc=0, candidato phpr-s117-la=$HB conservato, tree ripulito (cherry-pick -n + reset --hard)"

# --- admission p.6 ---
HA=$(shasum -a 256 "$A" | cut -c1-8)
if [ "$HA" != 8135dcf8 ]; then note "admission: A=$HA != 8135dcf8 — IDENTITÀ DIVERSA DAL CRITERIO"; echo 4 > "$OUT/admission.rc"; exit 4; fi
CASES="hit miss-magic miss-undef miss-ref miss-str"
RC=0
main_only() { awk '/^-- \{main\} /{f=1;next} /^-- /{f=0} f' "$1"; }
bigram() { awk '/PropGetSlotRecv/{p=1;next} p&&/BinaryTCPropSetPop/{print "SI";exit} {p=0}' "$1"; }
BIGRAMS=""
for c in $CASES; do
  "$A" "$CASES_DIR/$c.php" > "$OUT/$c-A.out" 2>&1; ra=$?
  "$B" "$CASES_DIR/$c.php" > "$OUT/$c-B.out" 2>&1; rb=$?
  if ! diff -q "$OUT/$c-A.out" "$OUT/$c-B.out" > /dev/null || [ "$ra" != "$rb" ]; then
    note "admission $c: output/rc DIVERGE (rc $ra vs $rb) — VIOLAZIONE"; RC=1
  fi
  PHPR_DUMP_OPS=1 "$A" "$CASES_DIR/$c.php" 2> "$OUT/$c-A.dump" > /dev/null
  PHPR_DUMP_OPS=1 "$B" "$CASES_DIR/$c.php" 2> "$OUT/$c-B.dump" > /dev/null
  main_only "$OUT/$c-A.dump" > "$OUT/$c-A.main"; main_only "$OUT/$c-B.dump" > "$OUT/$c-B.main"
  cmp -s "$OUT/$c-A.main" "$OUT/$c-B.main" || { note "admission $c: {main} DIVERGE — VIOLAZIONE (leva runtime-only)"; RC=1; }
  bg=$(bigram "$OUT/$c-A.main"); [ -n "$bg" ] || bg=NO
  BIGRAMS="$BIGRAMS $c=$bg"
  [ "$c" = hit ] && [ "$bg" != SI ] && { note "admission hit: bigramma ASSENTE in A — GATE FALLITO"; RC=1; }
done
echo "$RC" > "$OUT/admission.rc"
note "admission (rc=$RC da la-out/admission.rc): A=$HA B=$HB · 5 casi parità+dump: $([ "$RC" -eq 0 ] && echo OK || echo VIOLAZIONE) · bigramma:$BIGRAMS · batteria tree L-A×A′ rinviata alla promozione (deroga dichiarata, criterio p.6)"
exit "$RC"
