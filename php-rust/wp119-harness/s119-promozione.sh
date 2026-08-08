#!/bin/bash
# s119-promozione.sh <cand_hash16> — §6 PIENO del treno-2 (criterio
# s119-criterio-treno2.md p.4/8; guardie A/B + held-out N=3 già a verbale).
# Ordine REGOLE §6: commit leva → build ricetta (DEVE riprodurre il candidato)
# → batteria (rc in FILE, inventario per NOME vs pin s118) → re-build ricetta
# → pin-phpr.sh s119 → corpus-gate CANONICO → fixture chain → micro R=5 sul pin.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp119-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
OUT="$H/promo-out"; mkdir -p "$OUT"
VERD="$H/s119-treno2-verdetto.out"
CAND_EXP="${1:?uso: s119-promozione.sh <cand_hash16>}"
PIN_OLD="15dfb6b3da215b56"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rc"; exit 1; }

cd "$SRC" || exit 4
H0=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H0" = "$PIN_OLD" ] || stop "PRE: release=$H0 != pin $PIN_OLD — STOP"
git diff --quiet -- crates/ || stop "PRE: crates/ sporco — STOP"

git apply "$H/treno2.patch" || stop "patch NON applica"
git add -u crates/ || stop "git add -u fallito"
printf 'S-119 treno-2 (V3+V4+V5): guardia-Ref sul ricevitore POSSEDUTO\n\nPropIncDec/PropIsset/MethodCall: il deref_clone del ricevitore popped si\nesegue solo su Zval::Ref; altrimenti il valore si MUOVE. Criterio PRE\nfd2e8d3; guardie A/B tengono; held-out N=3 ok. Admission 6/6 output +\n6/6 dump interi byte-id (deroga 5 dichiarata).\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>\n' > /tmp/cmsg-treno2
git commit -F /tmp/cmsg-treno2 > "$OUT/commit.log" 2>&1 || stop "commit leva fallito"
HEADL=$(git rev-parse --short HEAD)
note "promozione: leva committata @ $HEADL"

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
if diff -q "$OUT/batteria-nomi.txt" "$SRC/wp118-harness/promo-out/batteria-nomi.txt" > /dev/null; then INV=IDENTICO; else
  INV="DIVERGE (promo-out/batteria-nomi.diff)"; diff "$SRC/wp118-harness/promo-out/batteria-nomi.txt" "$OUT/batteria-nomi.txt" > "$OUT/batteria-nomi.diff"
fi
note "promozione batteria: rc=$brc (da promo-out/batteria.rc) · $cnt · inventario vs pin s118: $INV"
[ "$brc" = 0 ] || stop "batteria rc=$brc"
[ "$INV" = IDENTICO ] || stop "inventario batteria DIVERGE"

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build2.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build2.rc"
[ "$rc" = 0 ] || stop "build2 rc=$rc"
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H2" = "$CAND_EXP" ] || stop "re-hash post-batteria $H2 != $CAND_EXP — STOP"
note "promozione: churn batteria neutralizzato (build ricetta → $H2 al byte)"

"$SRC/scripts/pin-phpr.sh" s119 > "$OUT/pin.log" 2>&1
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

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s119.out" 2>&1
note "promozione micro pin s119: $(grep -E '^rapporto_' "$OUT/micro-pin-s119.out" | tr '\n' ' ')"
echo 0 > "$OUT/rc"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rc): pin s119 = $H2 @ $HEADL"
exit 0
