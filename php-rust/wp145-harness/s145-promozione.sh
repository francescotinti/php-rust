#!/bin/bash
# s145-promozione.sh <cand_hash16> — gate di promozione LEVA FR1 dim-read fuso — S-145 (copia dichiarata di s138-promozione.sh: tag s145, path wp145; baseline ORM INVARIATA; DIVERGENZA DICHIARATA: la batteria ha UN test nuovo — il dente rczval_pattern_resta_nel_funnel (az.rev. S-144 #4) — e l'inventario lo attende PER NOME; collaudo copia-gate + manifest s145-promozione-copia.diff)
# (criterio s145-criterio-fr1.md; A/B R=5 s145-ab-fr1).
# Ordine REGOLE §6 (leva FD1 GIÀ committata prima del lancio): build ricetta (riproduce il
# candidato) → batteria (rc da comando, inventario per NOME vs pin s125, ZERO
# test nuovi attesi) → re-build ricetta → pin-phpr.sh s145 → corpus-gate
# CANONICO → fixture chain s109 → micro R=5 → gate ORM per NOME (baseline
# committata orm-baseline-failnames.txt) → gate http-kernel (0E/0F) sotto
# watchdog → pin-server.sh s145 (grado minimo). Registro nel verdetto del run
# PROMOSSO (rev. S-124 #3).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp145-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
SP="${PROMO_SP:?PROMO_SP (workdir APFS per i gate ORM/hk) richiesto}"
OUT="$H/promo-out"; mkdir -p "$OUT"
VERD="$H/s145-promo-verdetto.out"
CAND_EXP="${1:?uso: s145-promozione.sh <cand_hash16>}"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rcb"; exit 1; }

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
# inventario vs pin s125: S-145 dichiara UN test NUOVO (il dente rczval,
# az.rev. S-144 #4) — l'inventario è conforme SOLO se l'aggiunta è ESATTAMENTE
# quel nome e nessun test è SPARITO. I nomi dei compile-fail VmGate portano il
# NUMERO DI RIGA (volatile: slitta con ogni edit di vm/mod.rs — classe S-96:
# mai fidarsi della FORMA): normalizzato su ENTRAMBI i lati prima del comm.
normline(){ sed 's/(line [0-9][0-9]*)/(line N)/' "$1" | sort; }
normline "$SRC/wp125-harness/promo-out/batteria-nomi.txt" > "$OUT/base-norm.txt"
normline "$OUT/batteria-nomi.txt" > "$OUT/nomi-norm.txt"
NEW_ONLY=$(comm -13 "$OUT/base-norm.txt" "$OUT/nomi-norm.txt")
GONE=$(comm -23 "$OUT/base-norm.txt" "$OUT/nomi-norm.txt")
if [ "$NEW_ONLY" = "rczval_pattern_resta_nel_funnel" ] && [ -z "$GONE" ]; then
  INV="baseline s125 + il SOLO test dichiarato (rczval_pattern_resta_nel_funnel)"
else
  INV="DIVERGE (nuovi='$NEW_ONLY' spariti='$GONE' — diff in promo-out/batteria-inv.diff)"
  diff "$SRC/wp125-harness/promo-out/batteria-nomi.txt" "$OUT/batteria-nomi.txt" > "$OUT/batteria-inv.diff" || true
fi
note "promozione batteria: rc=$brc (da promo-out/batteria.rc) · $cnt · inventario: $INV"
[ "$brc" = 0 ] || stop "batteria rc=$brc"
case "$INV" in DIVERGE*) stop "inventario batteria DIVERGE";; esac

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build2.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build2.rc"
[ "$rc" = 0 ] || stop "build2 rc=$rc"
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H2" = "$CAND_EXP" ] || stop "re-hash post-batteria $H2 != $CAND_EXP — STOP"
note "promozione: churn batteria neutralizzato (build ricetta → $H2 al byte)"

"$SRC/scripts/pin-phpr.sh" s145 > "$OUT/pin.log" 2>&1
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
# Az.rev. S-130 #2: inventario per NOME dal marker della catena, confrontato
# con la lista CONGELATA — mai piu' conteggio-grep con denominatore cablato.
FX_ATTESI="hc1 move recv fx20 fx21 w9 preg teardown stash"
FX_VISTI=$(sed -n 's/^FIXTURE-CHAIN inventario=//p' "$OUT/fixture-chain.out")
[ "$FX_VISTI" = "$FX_ATTESI" ] || stop "fixture chain inventario diverso: visti='$FX_VISTI' attesi='$FX_ATTESI'"
note "promozione fixture chain: rc=0 (inventario per NOME conforme: $FX_VISTI)"

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s145.out" 2>&1
note "promozione micro pin s145: $(grep -E '^rapporto_' "$OUT/micro-pin-s145.out" | tr '\n' ' ')"

# ---- gate ORM per NOME (ricetta wp9: tarball su APFS, run col pin) ----
rm -rf "$SP/orm-work" && mkdir -p "$SP" && tar xzf "$GATES/orm-work.tgz" -C "$SP" || stop "untar orm-work"
( cd "$SP/orm-work" && "$WD" -t 2400 -s 600 -p "$OUT/orm.log" -o "$OUT" -- \
    "$BIN" vendor/bin/phpunit --no-coverage > "$OUT/orm.log" 2>&1 )
orc=$?; echo "$orc" > "$OUT/orm.rc"
SUMM=$(tr -d '\0' < "$OUT/orm.log" | grep -E "^(Tests:|OK)" | tail -1)
tr -d '\0' < "$OUT/orm.log" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/orm-failnames.txt"
if diff -q "$SRC/wp125-harness/orm-baseline-failnames.txt" "$OUT/orm-failnames.txt" > /dev/null; then
  note "promozione gate ORM: fail-set per NOME == baseline (16 nomi) · $SUMM"
else
  diff "$SRC/wp125-harness/orm-baseline-failnames.txt" "$OUT/orm-failnames.txt" > "$OUT/orm-failnames.diff" || true
  stop "gate ORM: fail-set DIVERGE per NOME (promo-out/orm-failnames.diff) · $SUMM"
fi

# ---- gate http-kernel: 0E/0F dichiarati (il tip si muove: mai i totali) ----
rm -rf "$SP/hk-work" && tar xzf "$GATES/hk-work.tgz" -C "$SP" || stop "untar hk-work"
( cd "$SP/hk-work" && "$WD" -t 1800 -s 600 -p "$OUT/hk.log" -o "$OUT" -- \
    "$BIN" vendor/bin/phpunit --no-coverage > "$OUT/hk.log" 2>&1 )
hrc=$?; echo "$hrc" > "$OUT/hk.rc"
HSUMM=$(tr -d '\0' < "$OUT/hk.log" | grep -E "^(Tests:|OK)" | tail -1)
HE=$(printf '%s' "$HSUMM" | sed -n 's/.*Errors: \([0-9]*\).*/\1/p'); HE=${HE:-0}
HF=$(printf '%s' "$HSUMM" | sed -n 's/.*Failures: \([0-9]*\).*/\1/p'); HF=${HF:-0}
if [ "$HE" = 0 ] && [ "$HF" = 0 ] && [ -n "$HSUMM" ]; then
  note "promozione gate http-kernel: 0E/0F · $HSUMM"
else
  stop "gate http-kernel: E=$HE F=$HF (attesi 0/0) · $HSUMM"
fi

"$SRC/scripts/pin-server.sh" s145 > "$OUT/pin-server.log" 2>&1
src_rc=$?; echo "$src_rc" > "$OUT/pin-server.rc"
[ "$src_rc" = 0 ] || stop "pin-server.sh rc=$src_rc"
note "promozione server: $(grep '^PIN server' "$OUT/pin-server.log" | tail -1)"

echo 0 > "$OUT/rcb"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb — il file rc che QUESTO script scrive): pin s145 = $H2"
exit 0
