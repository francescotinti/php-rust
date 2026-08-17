#!/bin/bash
# s150-promozione-parte2.sh — CONTINUAZIONE dichiarata di s150-promozione.sh
# dopo lo STOP fail-closed del flip-handler (famiglia generata per NOME:
# emenda dichiarata a MATCH DI PATH, commit 39aeff8). Build/batteria/pin/
# corpus-live GIÀ agli atti (stesso promo-out, stesso verdetto — righe 1-8):
# qui si riparte ESATTAMENTE dal punto di stop: flip-handler (famiglia
# emendata, STESSI raw del run live) → replay → fixture chain (10 gate) →
# quiescenza → micro R=5 → guardie R=5 → gate ORM → gate hk → pin-server.
# Le sezioni riprese sono COPIA LETTERALE della parte 1 (stessi comandi/gate).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp150-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
QUIESCE="$SRC/wp129-harness/s129-quiescenza.sh"
SP="${PROMO_SP:?PROMO_SP (workdir APFS per i gate ORM/hk) richiesto}"
OUT="$H/promo-out"
VERD="$H/s150-promo-verdetto.out"
CAND_EXP="cbbe71735effb165"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rcb"; exit 1; }

cd "$SRC" || exit 4
[ "$(shasum -a 256 "$BIN" | cut -c1-16)" = "$CAND_EXP" ] || stop "parte2 PRE: binario != candidato $CAND_EXP"
note "parte2 (dopo emenda famiglia 39aeff8): riparto dal flip-handler sugli STESSI raw del corpus live"

# idempotenza (secondo lancio parte2): l'aggiornamento perl del primo lancio
# è GIÀ applicato e il suo flip-handler.out è l'atto — non si ricalcola su
# congelato già mutato (darebbe 0 flip e cancellerebbe il record vero).
if grep -q "^FLIP-HANDLER rc=0" "$OUT/flip-handler.out" 2>/dev/null; then
  note "promozione flip: handler GIÀ applicato (flip-handler.out del primo passaggio = atto; solo il commit git era errato ed è EMENDATO)"
else
  "$H/s150-flip-handler.sh" "$OUT/corpus" > "$OUT/flip.log" 2>&1
  frc=$?; echo "$frc" > "$OUT/flip.rc"
  [ "$frc" = 0 ] || stop "flip-handler rc=$frc (violazione anche dopo l'emenda) — vedi promo-out/flip-handler.out"
fi
note "promozione flip: $(grep -E '^CONGELATO' "$OUT/flip-handler.out" | tr '\n' ' ')"
note "promozione flip per NOME: $(grep -E '^  [-~]' "$OUT/flip-handler.out" | sort -u | tr '\n' ' ')"
"$SRC/scripts/corpus-gate.sh" --replay "$OUT/corpus/off.norm" "$OUT/corpus/on.norm" "$OUT/corpus-replay"
rrc=$?; echo "$rrc" > "$OUT/corpus-replay-rc"
[ "$rrc" = 0 ] || stop "corpus-gate REPLAY rc=$rrc dopo il handler — STOP"
note "promozione corpus-gate replay: rc=0 sul congelato AGGIORNATO (flip dichiarati per nome)"

PHPR_PIN_ATTESO="$CAND_EXP" "$SRC/wp109-harness/s109-fixture-chain.sh" > "$OUT/fixture-chain.out" 2>&1
frc=$?; echo "$frc" > "$OUT/fixture-chain.rc"
[ "$frc" = 0 ] || stop "fixture chain rc=$frc"
FX_ATTESI="hc1 move recv fx20 fx21 w9 preg teardown stash backtrace"
FX_VISTI=$(sed -n 's/^FIXTURE-CHAIN inventario=//p' "$OUT/fixture-chain.out")
[ "$FX_VISTI" = "$FX_ATTESI" ] || stop "fixture chain inventario diverso: visti='$FX_VISTI' attesi='$FX_ATTESI'"
note "promozione fixture chain: rc=0 (inventario per NOME conforme, 10 gate: $FX_VISTI)"

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
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rcb): pin s150 = $CAND_EXP"
exit 0
