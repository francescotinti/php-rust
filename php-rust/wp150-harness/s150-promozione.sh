#!/bin/bash
# s150-promozione.sh <cand_hash16> — gate di promozione LEVA BT1 debug_backtrace — S-150 (copia dichiarata di s145-promozione.sh: tag s150, path wp150; DIVERGENZE DICHIARATE nel manifest s150-promozione-copia.diff: (1) inventario batteria = baseline s125 + SOLO rczval, INVARIATO da s145 — zero test nuovi dal diff 4a968b7..HEAD; (2) corpus: flip BT1 attesi ⊆ famiglia backtrace, gestione s150-flip-handler.sh fail-closed + replay rc=0; (3) fixture chain EMENDATA 9→10 gate (backtrace); (4) guardie R=5 + conferma m-backtrace + disasm bl-count = riparazione incidente 17 (az.rev.1 S-149), con quiescenza prima delle misure; (5) lock CI /private/tmp/phpr-measure.lock annotato, di proprietà della SESSIONE)
# (criterio s150-criterio-promo.md; A/B R=5 in s149-ab-bt1-verdetto.out).
# Ordine REGOLE §6 (leva BT1 GIÀ committata 6a7adc8): build ricetta (riproduce il
# candidato) → batteria (rc da comando, inventario per NOME vs pin s125) →
# re-build ricetta → pin-phpr.sh s150 → corpus-gate CANONICO (+flip-handler se
# flip dichiarati) → fixture chain s109 (10 gate) → quiescenza → micro R=5 →
# guardie R=5 → gate ORM per NOME → gate http-kernel (0E/0F) sotto watchdog →
# pin-server.sh s150 (grado minimo). Registro nel verdetto del run PROMOSSO.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp150-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
QUIESCE="$SRC/wp129-harness/s129-quiescenza.sh"
SP="${PROMO_SP:?PROMO_SP (workdir APFS per i gate ORM/hk) richiesto}"
OUT="$H/promo-out"; mkdir -p "$OUT"
VERD="$H/s150-promo-verdetto.out"
CAND_EXP="${1:?uso: s150-promozione.sh <cand_hash16>}"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rcb"; exit 1; }

cd "$SRC" || exit 4
git diff --quiet -- crates/ || stop "PRE: crates/ sporco — STOP"
[ -e /private/tmp/phpr-measure.lock ] && note "lock CI: presente (finestra di sessione, non lo tocco)" || note "lock CI: ASSENTE — la sessione lo doveva creare (proseguo, dichiarato)"

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build.rc"
[ "$rc" = 0 ] || stop "build rc=$rc"
HB=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$HB" = "$CAND_EXP" ] || stop "build NON riproduce il candidato dichiarato ($HB != $CAND_EXP) — STOP"
note "promozione: build ricetta riproduce il candidato $HB"

CARGO_INCREMENTAL=0 cargo test --release > "$OUT/batteria.log" 2>&1
brc=$?; echo "$brc" > "$OUT/batteria.rc"
cnt=$(awk '/^test result:/{p+=$4; f+=$6; ig+=$8} END{printf "%d/%d/%d", p, f, ig}' "$OUT/batteria.log")
grep -E '^test .* \.\.\. ' "$OUT/batteria.log" | sed 's/^test //; s/ \.\.\..*//' | sort > "$OUT/batteria-nomi.txt"
# inventario vs pin s125: come s145, l'UNICA aggiunta ammessa resta il dente
# rczval (S-145); dal diff 4a968b7..HEAD ZERO #[test] nuovi (criterio p.3).
# I nomi dei compile-fail VmGate portano il NUMERO DI RIGA (volatile):
# normalizzato su ENTRAMBI i lati prima del comm.
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
grep -E '^test .*debug_backtrace_array_fields' "$OUT/batteria.log" | grep -q ' ok$' \
  && note "promozione batteria: debug_backtrace_array_fields VERDE (criterio p.3)" \
  || stop "batteria: debug_backtrace_array_fields NON verde/assente"

SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build2.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build2.rc"
[ "$rc" = 0 ] || stop "build2 rc=$rc"
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
[ "$H2" = "$CAND_EXP" ] || stop "re-hash post-batteria $H2 != $CAND_EXP — STOP"
note "promozione: churn batteria neutralizzato (build ricetta → $H2 al byte)"

"$SRC/scripts/pin-phpr.sh" s150 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || stop "pin-phpr.sh rc=$prc"
note "promozione: $(tail -1 "$OUT/pin.log")"

"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus"
crc=$?; echo "$crc" > "$OUT/corpus-rc"
if [ "$crc" = 0 ]; then
  note "promozione corpus-gate: rc=0 — nomi==congelato, CONTENUTO==golden, off↔on zero (NESSUN flip)"
else
  note "promozione corpus-gate live: rc=$crc — flip BT1 attesi: passo al handler fail-closed (criterio p.4)"
  "$H/s150-flip-handler.sh" "$OUT/corpus" > "$OUT/flip.log" 2>&1
  frc=$?; echo "$frc" > "$OUT/flip.rc"
  [ "$frc" = 0 ] || stop "flip-handler rc=$frc (violazione: nome fuori famiglia o EXTRA) — vedi promo-out/flip-handler.out"
  note "promozione flip: $(grep -E '^CONGELATO' "$OUT/flip-handler.out" | tr '\n' ' ')"
  note "promozione flip per NOME: $(grep -E '^  -' "$OUT/flip-handler.out" | tr '\n' ' ')"
  "$SRC/scripts/corpus-gate.sh" --replay "$OUT/corpus/off.norm" "$OUT/corpus/on.norm" "$OUT/corpus-replay"
  rrc=$?; echo "$rrc" > "$OUT/corpus-replay-rc"
  [ "$rrc" = 0 ] || stop "corpus-gate REPLAY rc=$rrc dopo il handler — STOP"
  note "promozione corpus-gate replay: rc=0 sul congelato AGGIORNATO (flip dichiarati per nome)"
fi

PHPR_PIN_ATTESO="$CAND_EXP" "$SRC/wp109-harness/s109-fixture-chain.sh" > "$OUT/fixture-chain.out" 2>&1
frc=$?; echo "$frc" > "$OUT/fixture-chain.rc"
[ "$frc" = 0 ] || stop "fixture chain rc=$frc"
FX_ATTESI="hc1 move recv fx20 fx21 w9 preg teardown stash backtrace"
FX_VISTI=$(sed -n 's/^FIXTURE-CHAIN inventario=//p' "$OUT/fixture-chain.out")
[ "$FX_VISTI" = "$FX_ATTESI" ] || stop "fixture chain inventario diverso: visti='$FX_VISTI' attesi='$FX_ATTESI'"
note "promozione fixture chain: rc=0 (inventario per NOME conforme, 10 gate: $FX_VISTI)"

# quiescenza PRIMA delle misure (micro + guardie): mai cifre con build/CI in volo
QOK=1
for t in $(seq 1 30); do
  if "$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce-t$t.log" 2>&1; then QOK=0; note "promozione quiescenza: PASS al tentativo $t"; break; fi
  sleep 60
done
[ "$QOK" = 0 ] || stop "quiescenza MAI PASS in 30 tentativi — STOP prima delle misure"

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s150.out" 2>&1
note "promozione micro pin s150: $(grep -E '^rapporto_' "$OUT/micro-pin-s150.out" | tr '\n' ' ')"

ARM_A="$STASH/phpr-s145" ARM_B="$BIN" "$H/s150-guardie-r5.sh"
grc=$?; echo "$grc" > "$OUT/guardie.rc"
while IFS= read -r l; do note "  $l"; done < <(grep -E '^(guardia|m-backtrace|bilaterale|disasm|parita|pavimenti|GUARDIE)' "$H/s150-guardie-verdetto.out")
[ "$grc" = 0 ] || stop "guardie R=5 rc=$grc (guardia ROSSA) — promozione ABORTITA"

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

"$SRC/scripts/pin-server.sh" s150 > "$OUT/pin-server.log" 2>&1
src_rc=$?; echo "$src_rc" > "$OUT/pin-server.rc"
[ "$src_rc" = 0 ] || stop "pin-server.sh rc=$src_rc"
note "promozione server: $(grep '^PIN server' "$OUT/pin-server.log" | tail -1)"

echo 0 > "$OUT/rcb"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb — il file rc che QUESTO script scrive): pin s150 = $H2"
exit 0
