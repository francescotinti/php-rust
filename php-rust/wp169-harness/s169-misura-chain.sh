#!/bin/bash
# s169-misura-chain.sh — finestra di misura S-169 (sequenziale, detached):
# (0) attesa quiete (CI in coda: s129-quiescenza ogni 60 s, max 3 h);
# (1) dump m5: il loop DEVE avere 8 Nop; (2) bracci --braccio s169-m5/m7 + disasm;
# (3) A/B vs m0: m5 R=5, m7 R=5, m4b R=9, m3 R=9 (stash s168); (4) s169-xctrace.sh.
# Log ab-out/chain.log; fine = ab-out/chain.done. STOP solo su rc di apparato.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"; R="$H/.."; OUT="$H/ab-out"
LOG="$OUT/chain.log"; : > "$LOG"; rm -f "$OUT/chain.done"
BIN=/private/tmp/s169-mock-bin; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"; M0="$STASH/phpr-s168-m0"
h8(){ shasum -a 256 "$1" | cut -c1-8; }
say(){ echo "$(date '+%T') $*" >> "$LOG"; }
for t in m5 m7; do [ "$(cat "$OUT/build-$t.rc" 2>/dev/null)" = 0 ] || { say "STOP: build $t non verde"; touch "$OUT/chain.done"; exit 7; }; done
LOOP=$(PHPR_DUMP_OPS=1 "$BIN/phpr-m5" "$R/wp164-harness/arith-dq.php" 2>&1 | sed -n '/^-- {main}/,/Ret/p' | sed -n '/CmpJmpSC/,/IncDecSlotJmp/p' | awk '{print $2}' | tr '\n' ' ')
say "loop m5: $LOOP"
[ "$(echo "$LOOP" | grep -o Nop | wc -l | tr -d ' ')" = 8 ] || { say "STOP: m5 NULLO (Nop nel loop != 8)"; touch "$OUT/chain.done"; exit 5; }
n=0; until "$R/wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-wait.rc" > "$OUT/quiesce-wait.log" 2>&1; do n=$((n+1)); [ $n -ge 180 ] && { say "STOP: quiete mai raggiunta in 3 h"; touch "$OUT/chain.done"; exit 8; }; sleep 60; done
say "quiete raggiunta dopo $n attese"
for t in m5 m7; do
  [ -e "$STASH/phpr-s169-$t" ] || "$R/scripts/pin-phpr.sh" --braccio "s169-$t" "$BIN/phpr-$t" >> "$LOG" 2>&1 || { say "STOP braccio $t"; touch "$OUT/chain.done"; exit 1; }
  "$R/wp160-harness/s160-disasm-bl.sh" "$M0" "$BIN/phpr-$t" "$OUT/disasm-$t.out" >> "$LOG" 2>&1
done
run_ab(){ "$H/s169-ab-mock.sh" "$M0" "$(h8 "$M0")" "$1" "$(h8 "$1")" "$2" "$3"; rc=$?; say "ab $2 R=$3 rc=$rc"; case "$rc" in 1|2|7|9) say "STOP apparato"; touch "$OUT/chain.done"; exit "$rc";; esac; }
run_ab "$BIN/phpr-m5" m5 5
run_ab "$BIN/phpr-m7" m7 5
run_ab "$STASH/phpr-s168-m4b" m4b9 9
run_ab "$STASH/phpr-s168-m3" m39 9
"$H/s169-xctrace.sh"; say "xctrace rc=$?"
touch "$OUT/chain.done"; say "fine"
