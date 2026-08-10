#!/bin/bash
# s127-promozione2.sh — CONTINUAZIONE della catena di promozione L-OL1-F1 dal
# corpus-gate (fermata a rc=1 per il flip bug69534, emendato e DICHIARATO in
# s127-emenda-corpus.md). Stadi: corpus (rieseguito post-emenda) -> fixture
# chain -> micro R=5 -> gate ORM per NOME -> gate hk 0E/0F -> pin-server s127.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
SRC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$SRC/wp127-harness"
BIN="$HOME/Claude/php-rust-output/release/phpr"
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
SP="${PROMO_SP:?PROMO_SP richiesto}"
OUT="$H/promo-out"; mkdir -p "$OUT"
VERD="$H/s127-promo-verdetto.out"
CAND_EXP="834f5e01fbdb7ebc"
note(){ echo "$1"; echo "$1" >> "$VERD"; }
stop(){ note "$1"; echo 1 > "$OUT/rc2"; exit 1; }
rm -f "$OUT/rc2"
[ "$(shasum -a 256 "$BIN" | cut -c1-16)" = "$CAND_EXP" ] || stop "BIN != candidato"
H2="$CAND_EXP"

"$SRC/scripts/corpus-gate.sh" "$RUNNER" "$OUT/corpus2"
crc=$?; echo "$crc" > "$OUT/corpus2-rc"
[ "$crc" = 0 ] || stop "corpus-gate POST-EMENDA rc=$crc (promo-out/corpus2/corpus-gate.out)"
note "promozione corpus-gate POST-EMENDA (1414): rc=0 — nomi==congelato, CONTENUTO==golden, off<->on zero"



PHPR_PIN_ATTESO="$CAND_EXP" "$SRC/wp109-harness/s109-fixture-chain.sh" > "$OUT/fixture-chain.out" 2>&1
frc=$?; echo "$frc" > "$OUT/fixture-chain.rc"
[ "$frc" = 0 ] || stop "fixture chain rc=$frc"
note "promozione fixture chain: rc=0 ($(grep -c '^-- .* rc=0' "$OUT/fixture-chain.out")/6 gate verdi)"

PHPR="$BIN" R=5 "$SRC/wp97-harness/micro/run-micro.sh" > "$OUT/micro-pin-s127.out" 2>&1
note "promozione micro pin s127: $(grep -E '^rapporto_' "$OUT/micro-pin-s127.out" | tr '\n' ' ')"

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

"$SRC/scripts/pin-server.sh" s127 > "$OUT/pin-server.log" 2>&1
src_rc=$?; echo "$src_rc" > "$OUT/pin-server.rc"
[ "$src_rc" = 0 ] || stop "pin-server.sh rc=$src_rc"
note "promozione server: $(grep '^PIN server' "$OUT/pin-server.log" | tail -1)"

echo 0 > "$OUT/rc2"
note "PROMOZIONE COMPLETA rc=0 (da promo-out/rc): pin s127 = $H2"
exit 0
