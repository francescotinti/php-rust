#!/bin/bash
# s170-misura-chain-b.sh — finestra di misura S-170 criterio-b (sequenziale, detached; p.4):
# (0) build verdi m10..m13 (`[ -s ]` sui .rc); (1) dump del loop dq di ogni braccio: DEVE essere
# CmpJmpSC · BinarySCSCDst · Sweep · IncDecSlotJmp (4 op); (2) attesa quiete (s129-quiescenza, max 3 h);
# (3) bracci --braccio s170-m10..m13 + disasm bl vs m0; (4) A/B m10, m11, m12, m13 R=5 con A = m0-s170
# (giudice dq, guardia e2: s170-ab-dq.sh). COPIA DICHIARATA di s170-misura-chain.sh (manifest v3).
# Log ab-out/chain-b.log; fine = ab-out/chain-b.done. STOP solo su rc di apparato.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"; R="$H/.."; OUT="$H/ab-out"
LOG="$OUT/chain-b.log"; : > "$LOG"; rm -f "$OUT/chain-b.done"
BIN=/private/tmp/s170-mock-bin; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
M0="$BIN/phpr-m0"; DQ="$R/wp164-harness/arith-dq.php"
h8(){ shasum -a 256 "$1" | cut -c1-8; }
say(){ echo "$(date '+%T') $*" >> "$LOG"; }
stop(){ say "STOP: $*"; touch "$OUT/chain-b.done"; exit "${2:-7}"; }
[ -s "$M0" ] || stop "m0 assente" 7
for t in m10 m11 m12 m13; do [ -s "$OUT/build-$t.rc" ] && [ "$(cat "$OUT/build-$t.rc")" = 0 ] || stop "build $t non verde" 7; [ -s "$BIN/phpr-$t" ] || stop "binario $t assente" 7; done
[ -s "$DQ" ] || stop "driver dq assente/vuoto" 7
for t in m10 m11 m12 m13; do
  LOOP=$(PHPR_DUMP_OPS=1 "$BIN/phpr-$t" "$DQ" 2>&1 | sed -n '/^-- {main}/,/Ret/p' | sed -n '/CmpJmpSC/,/IncDecSlotJmp/p' | awk '{print $2}' | tr '\n' ' ')
  say "loop $t: $LOOP"
  [ "$LOOP" = "CmpJmpSC BinarySCSCDst Sweep IncDecSlotJmp " ] || stop "loop $t ≠ 4 op attesi" 5
done
n=0; until "$R/wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-wait-b.rc" > "$OUT/quiesce-wait-b.log" 2>&1; do n=$((n+1)); [ $n -ge 180 ] && stop "quiete mai raggiunta ($n attese)" 8; /bin/sleep 60; done
say "quiete raggiunta dopo $n attese"
for t in m10 m11 m12 m13; do
  [ -e "$STASH/phpr-s170-$t" ] || "$R/scripts/pin-phpr.sh" --braccio "s170-$t" "$BIN/phpr-$t" >> "$LOG" 2>&1 || stop "braccio $t" 1
  "$R/wp160-harness/s160-disasm-bl.sh" "$M0" "$BIN/phpr-$t" "$OUT/disasm-$t.out" >> "$LOG" 2>&1
done
run_ab(){ "$H/s170-ab-dq.sh" "$M0" "$(h8 "$M0")" "$1" "$(h8 "$1")" "$2" "$3"; rc=$?; say "ab $2 R=$3 rc=$rc"; case "$rc" in 1|2|7|8|9) stop "apparato ab $2 rc=$rc" "$rc";; esac; }
for t in m10 m11 m12 m13; do run_ab "$BIN/phpr-$t" "$t" 5; done
touch "$OUT/chain-b.done"; say "fine"
