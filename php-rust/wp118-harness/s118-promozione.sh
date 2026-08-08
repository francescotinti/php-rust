#!/bin/bash
# s118-promozione.sh — §6 PIENO del treno-1 (criterio s118-criterio-treno1.md
# p.6; A/B PROMUOVIBILE + held-out 3/3 già a verbale). Ordine REGOLE §6:
# commit leva → build ricetta (hash DEVE riprodurre il candidato 15dfb6b3,
# determinismo) → batteria (rc in FILE, inventario per NOME vs pin s117) →
# re-build ricetta (neutralizza il relink test) → pin-phpr.sh s118 (smoke+stash)
# → corpus-gate CANONICO (nomi E contenuto E off↔on — prima esecuzione live
# del gate S-118) → fixture chain → micro+held-out R=5 sul pin.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp118-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
OUT="$H/promo-out"; mkdir -p "$OUT"
VERD="$H/s118-treno1-verdetto.out"
CAND_EXP="15dfb6b3da215b56"
PIN_OLD="1656580e4e590e20"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rc"; exit 1; }

cd "$SRC" || exit 4
H0=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H0" = "$PIN_OLD" ] || stop "PRE: release=$H0 != pin $PIN_OLD — STOP"
git diff --quiet -- crates/ || stop "PRE: crates/ sporco — STOP"

# --- commit della leva (hook: add -u, mai path .rs) ---
git apply "$H/hp1-composto.patch" || stop "patch NON applica"
git add -u crates/ || stop "git add -u fallito"
printf 'S-118 treno-1 (V1+V2 = H-P1a/b): probe IC col ricevitore in prestito, composto sopra L-A\n\nPropGetSlot (P1a) e ramo !fused di PropGetSlotRecv (P1b): hit senza clone\nRc del ricevitore; miss al sentiero storico invariato. Criterio PRE 9013b2b;\nA/B R=5: prop +5,33 (soglia 4,00) 5/5; guardie tengono (calls +0,50);\nheld-out 3/3. Admission 6/6 output + 6/6 dump interi byte-id.\n\nCo-Authored-By: Claude Fable 5 <noreply@anthropic.com>\n' > /tmp/cmsg-treno1
git commit -F /tmp/cmsg-treno1 > "$OUT/commit.log" 2>&1 || stop "commit leva fallito"
HEADL=$(git rev-parse --short HEAD)
note "promozione: leva committata @ $HEADL"

# --- build ricetta: deve RIPRODURRE il candidato giudicato ---
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build.rc"
[ "$rc" = 0 ] || stop "build rc=$rc"
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$HB" = "$CAND_EXP" ] || stop "build NON riproduce il candidato giudicato ($HB != $CAND_EXP) — churn NON dichiarabile, STOP"
note "promozione: build ricetta riproduce il candidato $HB (A/B e held-out valgono per QUESTO binario)"

# --- batteria (rc dal comando in FILE; inventario per NOME vs pin s117) ---
CARGO_INCREMENTAL=0 cargo test --release > "$OUT/batteria.log" 2>&1
brc=$?; echo "$brc" > "$OUT/batteria.rc"
cnt=$(awk '/^test result:/{p+=$4; f+=$6; ig+=$8} END{printf "%d/%d/%d", p, f, ig}' "$OUT/batteria.log")
grep -E '^test .* \.\.\. ' "$OUT/batteria.log" | sed 's/^test //; s/ \.\.\..*//' | sort > "$OUT/batteria-nomi.txt"
if diff -q "$OUT/batteria-nomi.txt" "$SRC/wp117-harness/promo-out/batteria-nomi.txt" > /dev/null; then INV=IDENTICO; else
  INV="DIVERGE (promo-out/batteria-nomi.diff)"; diff "$SRC/wp117-harness/promo-out/batteria-nomi.txt" "$OUT/batteria-nomi.txt" > "$OUT/batteria-nomi.diff"
fi
note "promozione batteria: rc=$brc (da promo-out/batteria.rc) · $cnt · inventario vs pin s117: $INV"
[ "$brc" = 0 ] || stop "batteria rc=$brc"
[ "$INV" = IDENTICO ] || stop "inventario batteria DIVERGE"

# --- re-build ricetta: neutralizza il relink della batteria ---
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build2.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build2.rc"
[ "$rc" = 0 ] || stop "build2 rc=$rc"
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H2" = "$CAND_EXP" ] || stop "re-hash post-batteria $H2 != $CAND_EXP — churn NON neutralizzato, STOP"
note "promozione: churn batteria neutralizzato (build ricetta → $H2 al byte)"

# --- pin (smoke parità 2 modi + stash + registro, SOLO dallo script) ---
"$SRC/scripts/pin-phpr.sh" s118 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || stop "pin-phpr.sh rc=$prc"
note "promozione: $(tail -1 "$OUT/pin.log")"

# --- corpus-gate CANONICO (nomi E contenuto E off↔on) ---
"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus"
crc=$?; echo "$crc" > "$OUT/corpus-rc"
[ "$crc" = 0 ] || stop "corpus-gate rc=$crc (vedi promo-out/corpus/corpus-gate.out)"
note "promozione corpus-gate: rc=0 — nomi==congelato, CONTENUTO==golden, off↔on zero (prima esecuzione live del gate canonico)"

# --- fixture chain ---
PHPR_PIN_ATTESO="$CAND_EXP" "$SRC/wp109-harness/s109-fixture-chain.sh" > "$OUT/fixture-chain.out" 2>&1
frc=$?; echo "$frc" > "$OUT/fixture-chain.rc"
[ "$frc" = 0 ] || stop "fixture chain rc=$frc"
note "promozione fixture chain: rc=0 ($(grep -c '^-- .* rc=0' "$OUT/fixture-chain.out")/6 gate verdi)"

# --- micro + held-out R=5 sul pin nuovo ---
PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s118.out" 2>&1
PHPR="$BIN" R=5 "$SRC/wp111-harness/heldout/run-heldout.sh" > "$OUT/heldout-pin-s118.out" 2>&1
note "promozione micro pin s118: $(grep -E '^rapporto_' "$OUT/micro-pin-s118.out" | tr '\n' ' ')"
note "promozione held-out pin s118: $(grep -E '^rapporto_' "$OUT/heldout-pin-s118.out" | tr '\n' ' ')"
echo 0 > "$OUT/rc"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rc): pin s118 = $H2 @ $HEADL"
exit 0
