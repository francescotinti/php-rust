#!/bin/bash
# s168-misura-chain2.sh — seconda finestra S-168 (sequenziale, detached):
# (1) verifica dump: m4 NON emette Sweep nel loop di arith-dq (m0 sì);
# (2) braccio pin-phpr.sh --braccio s168-m4; disasm bl; (3) A/B m4 vs m0;
# (4) riesecuzione stride col criterio EMENDATO (TAG=stride2). Log ab-out/chain2.log.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"; R="$H/.."; OUT="$H/ab-out"
LOG="$OUT/chain2.log"; : > "$LOG"; rm -f "$OUT/chain2.done"
BIN=/private/tmp/s168-mock-bin; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
h8(){ shasum -a 256 "$1" | cut -c1-8; }
say(){ echo "$(date '+%T') $*" >> "$LOG"; }
[ "$(cat "$OUT/build-m4.rc" 2>/dev/null)" = 0 ] || { say "STOP: build m4 non verde"; touch "$OUT/chain2.done"; exit 7; }
S0=$(PHPR_DUMP_OPS=1 "$BIN/phpr-m0" "$R/wp164-harness/arith-dq.php" 2>&1 | sed -n '/n_slots/,/Ret/p' | grep -c Sweep)
S4=$(PHPR_DUMP_OPS=1 "$BIN/phpr-m4" "$R/wp164-harness/arith-dq.php" 2>&1 | sed -n '/n_slots/,/Ret/p' | grep -c Sweep)
say "dump Sweep in {main} arith-dq: m0=$S0 m4=$S4"
[ -e "$STASH/phpr-s168-m4" ] || "$R/scripts/pin-phpr.sh" --braccio s168-m4 "$BIN/phpr-m4" >> "$LOG" 2>&1 || { say "STOP braccio m4"; touch "$OUT/chain2.done"; exit 1; }
"$R/wp160-harness/s160-disasm-bl.sh" "$BIN/phpr-m0" "$BIN/phpr-m4" "$OUT/disasm-m4.out" >> "$LOG" 2>&1
"$H/s168-ab-mock.sh" "$BIN/phpr-m0" "$(h8 "$BIN/phpr-m0")" "$BIN/phpr-m4" "$(h8 "$BIN/phpr-m4")" m4; say "ab m4 rc=$?"
TAG=stride2 "$H/s168-stride.sh"; say "stride2 rc=$?"
touch "$OUT/chain2.done"; say "fine"
