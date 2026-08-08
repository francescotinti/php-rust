#!/bin/bash
# s117-promozione-2.sh — continuazione §6 dopo lo STOP del re-hash (churn
# DICHIARATO: `cargo test --release` rilinka il bin col grafo di test — lezione
# S-104; il binario 9595e59a NON è della ricetta). Cura resa lecita dal
# determinismo provato ×2: build della RICETTA → hash DEVE tornare == H1
# (1656580e…) → stash via pin-phpr.sh → corpus ×2 → micro/held-out R=5.
# Nessuna build dopo lo stash (gate nullo altrimenti).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
export SOURCE_DATE_EPOCH=0
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
CORPUS="/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
FROZ="$H/../wp109-harness/corpus-gate"
OUT="$H/promo-out"; mkdir -p "$OUT/corpus"
VERD="$H/s117-la-verdetto.out"
H1EXP="1656580e4e590e20"
note(){ echo "$1"; echo "$1" >> "$VERD"; }

cd "$SRC" || exit 4
CARGO_INCREMENTAL=0 cargo build --release > "$OUT/build2.log" 2>&1
rc=$?; echo "$rc" > "$OUT/build2.rc"
[ "$rc" = 0 ] || { note "promozione-2: build FALLITA rc=$rc"; echo 1 > "$OUT/rc"; exit 1; }
H2=$(shasum -a 256 "$BIN" | cut -c1-16)
if [ "$H2" != "$H1EXP" ]; then
  note "promozione-2: hash $H2 != H1 $H1EXP — il churn NON è neutralizzato, STOP (pin non eseguito)"
  echo 1 > "$OUT/rc"; exit 1
fi
note "promozione-2: churn batteria neutralizzato — build ricetta riproduce H1=$H2 (determinismo esercitato); batteria 1742/0/2 copre questo sorgente, binario byte-identico"

"$SRC/scripts/pin-phpr.sh" s117 > "$OUT/pin.log" 2>&1
prc=$?; echo "$prc" > "$OUT/pin.rc"
[ "$prc" = 0 ] || { note "promozione-2: pin-phpr.sh FALLITO rc=$prc"; echo 1 > "$OUT/rc"; exit 1; }
note "promozione-2: $(tail -1 "$OUT/pin.log")"

GRC=0
for mode in off on; do
  case "$mode" in off) reg=0 ;; on) reg=1 ;; esac
  /usr/bin/env PHPR_REG_LOWER="$reg" "$RUNNER" --isolate --list-fails "$CORPUS" > "$OUT/corpus/$mode.log" 2>&1
  echo $? > "$OUT/corpus/$mode.rc"
  tr -d '\0' < "$OUT/corpus/$mode.log" > "$OUT/corpus/$mode.norm"
  grep -E '^--- .+ ---$' "$OUT/corpus/$mode.norm" | sed 's/^--- //; s/ ---$//' | sort > "$OUT/corpus/$mode.fails"
  nf=$(wc -l < "$OUT/corpus/$mode.fails" | tr -d ' ')
  if diff -q "$OUT/corpus/$mode.fails" "$FROZ/corpus-s109-$mode.fails" > /dev/null; then
    note "promozione-2 corpus($mode): fail-set per NOME == congelato s109 ($nf)"
  else
    diff "$FROZ/corpus-s109-$mode.fails" "$OUT/corpus/$mode.fails" > "$OUT/corpus/$mode-vs-s109.diff"
    note "promozione-2 corpus($mode): DIVERGE dal congelato ($nf vs 1415)"; GRC=1
  fi
done
if ! diff -q "$OUT/corpus/off.fails" "$OUT/corpus/on.fails" > /dev/null; then note "promozione-2 corpus: nomi off≠on"; GRC=1; fi
echo "$GRC" > "$OUT/corpus-rc"
[ "$GRC" = 0 ] || { note "promozione-2: CORPUS FALLITO"; echo 1 > "$OUT/rc"; exit 1; }

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s117.out" 2>&1
PHPR="$BIN" R=5 "$SRC/wp111-harness/heldout/run-heldout.sh" > "$OUT/heldout-pin-s117.out" 2>&1
note "promozione micro pin s117: $(grep -E '^rapporto_' "$OUT/micro-pin-s117.out" | tr '\n' ' ')"
note "promozione held-out pin s117: $(grep -E '^rapporto_' "$OUT/heldout-pin-s117.out" | tr '\n' ' ')"
echo 0 > "$OUT/rc"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rc): pin s117 = $H2"
exit 0
