#!/bin/bash
# s124-promozione.sh <cand_hash16> — gate di promozione PhpStr single-alloc
# (criterio s124-criterio-phpstr.md p.8; bersaglio+guardie già a verbale rc=0).
# Ordine REGOLE §6 (leva GIÀ committata @ 43d7610): build ricetta (DEVE
# riprodurre il candidato) → batteria (rc da comando, inventario per NOME vs
# pin s120 + SOLI 4 test nuovi zstr DICHIARATI) → re-build ricetta → pin-phpr.sh
# s124 → corpus-gate CANONICO → fixture chain s109 → micro R=5 sul pin.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp124-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
OUT="$H/promo-out"; mkdir -p "$OUT"
VERD="$H/s124-str1-verdetto.out"
CAND_EXP="${1:?uso: s124-promozione.sh <cand_hash16>}"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rc"; exit 1; }

cd "$SRC" || exit 4
git diff --quiet -- crates/ || stop "PRE: crates/ sporco — STOP"

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build.rc"
[ "$rc" = 0 ] || stop "build rc=$rc"
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$HB" = "$CAND_EXP" ] || stop "build NON riproduce il candidato giudicato ($HB != $CAND_EXP) — STOP"
note "promozione: build ricetta riproduce il candidato $HB"

CARGO_INCREMENTAL=0 cargo test --release > "$OUT/batteria.log" 2>&1
brc=$?; echo "$brc" > "$OUT/batteria.rc"
cnt=$(awk '/^test result:/{p+=$4; f+=$6; ig+=$8} END{printf "%d/%d/%d", p, f, ig}' "$OUT/batteria.log")
grep -E '^test .* \.\.\. ' "$OUT/batteria.log" | sed 's/^test //; s/ \.\.\..*//' | sort > "$OUT/batteria-nomi.txt"
# Inventario: vs pin s120 + SOLI i 4 test nuovi della leva (dichiarati nel
# criterio p.9/commit 43d7610). Ogni altra differenza = STOP.
cat > "$OUT/nuovi-attesi.txt" <<'EOF'
zstr::tests::append_grows_across_many_extends
zstr::tests::append_unique_shared_and_hash_invalidation
zstr::tests::builder_exact_and_underfill
zstr::tests::eq_identity_fast_path_and_ptr_api
EOF
MISS=$(comm -23 "$SRC/wp120-harness/promo-out/batteria-nomi.txt" "$OUT/batteria-nomi.txt" | wc -l | tr -d ' ')
comm -13 "$SRC/wp120-harness/promo-out/batteria-nomi.txt" "$OUT/batteria-nomi.txt" > "$OUT/batteria-extra.txt"
if [ "$MISS" = 0 ] && diff -q "$OUT/batteria-extra.txt" "$OUT/nuovi-attesi.txt" > /dev/null; then
  INV="IDENTICO + 4 nuovi zstr DICHIARATI"
else
  INV="DIVERGE (mancanti=$MISS, extra in promo-out/batteria-extra.txt)"
fi
note "promozione batteria: rc=$brc (da promo-out/batteria.rc) · $cnt · inventario vs pin s120: $INV"
[ "$brc" = 0 ] || stop "batteria rc=$brc"
case "$INV" in DIVERGE*) stop "inventario batteria DIVERGE";; esac

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build2.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build2.rc"
[ "$rc" = 0 ] || stop "build2 rc=$rc"
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H2" = "$CAND_EXP" ] || stop "re-hash post-batteria $H2 != $CAND_EXP — STOP"
note "promozione: churn batteria neutralizzato (build ricetta → $H2 al byte)"

"$SRC/scripts/pin-phpr.sh" s124 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || stop "pin-phpr.sh rc=$prc"
note "promozione: $(tail -1 "$OUT/pin.log")"

"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus"
crc=$?; echo "$crc" > "$OUT/corpus-rc"
[ "$crc" = 0 ] || stop "corpus-gate rc=$crc (vedi promo-out/corpus/corpus-gate.out)"
note "promozione corpus-gate: rc=0 — nomi==congelato, CONTENUTO==golden, off↔on zero"

PHPR_PIN_ATTESO="$CAND_EXP" "$SRC/wp109-harness/s109-fixture-chain.sh" > "$OUT/fixture-chain.out" 2>&1
frc=$?; echo "$frc" > "$OUT/fixture-chain.rc"
[ "$frc" = 0 ] || stop "fixture chain rc=$frc"
note "promozione fixture chain: rc=0 ($(grep -c '^-- .* rc=0' "$OUT/fixture-chain.out")/6 gate verdi)"

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s124.out" 2>&1
note "promozione micro pin s124: $(grep -E '^rapporto_' "$OUT/micro-pin-s124.out" | tr '\n' ' ')"
echo 0 > "$OUT/rc"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rc): pin s124 = $H2"
exit 0
