#!/bin/bash
# s170-misura-chain.sh — finestra di misura S-170 (sequenziale, detached; criterio s170-criterio.md p.6):
# (0) build verdi m0/m8/m9 (`[ -s ]` sui .rc); (1) dump del loop E2 di m8 e m9: DEVE essere
# esattamente CmpJmpSC + IncDecSlotJmp (2 op, nessun Nop); (2) attesa quiete (s129-quiescenza, max 3 h);
# (3) bracci --braccio s170-m8/m9 + disasm bl vs m0; (4) se hash(m0-s170) ≠ 36d73812 (phpr-s168-m0):
# A/B m0-s170 vs phpr-s168-m0 R=5 = banda-copia; (5) A/B m8 R=5, m9 R=5 con A = m0-s170.
# COPIA DICHIARATA di wp169-harness/s169-misura-chain.sh (manifest s170-misura-chain-copia-v3.diff).
# Log ab-out/chain.log; fine = ab-out/chain.done. STOP solo su rc di apparato.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"; R="$H/.."; OUT="$H/ab-out"
LOG="$OUT/chain.log"; : > "$LOG"; rm -f "$OUT/chain.done"
BIN=/private/tmp/s170-mock-bin; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
M0="$BIN/phpr-m0"; M0OLD="$STASH/phpr-s168-m0"; E2="$R/wp168-harness/arith-e2.php"
h8(){ shasum -a 256 "$1" | cut -c1-8; }
say(){ echo "$(date '+%T') $*" >> "$LOG"; }
stop(){ say "STOP: $*"; touch "$OUT/chain.done"; exit "${2:-7}"; }
for t in m0 m8 m9; do [ -s "$OUT/build-$t.rc" ] && [ "$(cat "$OUT/build-$t.rc")" = 0 ] || stop "build $t non verde" 7; [ -s "$BIN/phpr-$t" ] || stop "binario $t assente" 7; done
[ -s "$E2" ] || stop "driver E2 assente/vuoto" 7
for t in m8 m9; do
  LOOP=$(PHPR_DUMP_OPS=1 "$BIN/phpr-$t" "$E2" 2>&1 | sed -n '/^-- {main}/,/Ret/p' | sed -n '/CmpJmpSC/,/IncDecSlotJmp/p' | awk '{print $2}' | tr '\n' ' ')
  say "loop $t: $LOOP"
  [ "$LOOP" = "CmpJmpSC IncDecSlotJmp " ] || stop "loop $t ≠ CmpJmpSC+IncDecSlotJmp (2 op)" 5
done
n=0; until "$R/wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-wait.rc" > "$OUT/quiesce-wait.log" 2>&1; do n=$((n+1)); [ $n -ge 180 ] && stop "quiete mai raggiunta ($n attese)" 8; /bin/sleep 60; done
say "quiete raggiunta dopo $n attese"
for t in m8 m9; do
  [ -e "$STASH/phpr-s170-$t" ] || "$R/scripts/pin-phpr.sh" --braccio "s170-$t" "$BIN/phpr-$t" >> "$LOG" 2>&1 || stop "braccio $t" 1
  "$R/wp160-harness/s160-disasm-bl.sh" "$M0" "$BIN/phpr-$t" "$OUT/disasm-$t.out" >> "$LOG" 2>&1
done
run_ab(){ "$H/s170-ab-mock.sh" "$M0" "$(h8 "$M0")" "$1" "$(h8 "$1")" "$2" "$3"; rc=$?; say "ab $2 R=$3 rc=$rc"; case "$rc" in 1|2|7|8|9) stop "apparato ab $2 rc=$rc" "$rc";; esac; }
say "m0-s170 hash=$(h8 "$M0") vs phpr-s168-m0 36d73812"
if [ "$(h8 "$M0")" != "36d73812" ]; then run_ab "$M0OLD" m0x 5; else say "m0 riproducibile al byte: banda-copia non dovuta"; fi
run_ab "$BIN/phpr-m8" m8 5
run_ab "$BIN/phpr-m9" m9 5
touch "$OUT/chain.done"; say "fine"
