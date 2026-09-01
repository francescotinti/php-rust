#!/bin/bash
# s168-misura-chain3.sh — terza finestra S-168: build m4b (già preparata), dump
# del loop (deve essere SENZA Sweep), braccio, disasm, A/B m4b vs m0. Log ab-out/chain3.log.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"; R="$H/.."; OUT="$H/ab-out"
LOG="$OUT/chain3.log"; : > "$LOG"; rm -f "$OUT/chain3.done"
BIN=/private/tmp/s168-mock-bin; STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
h8(){ shasum -a 256 "$1" | cut -c1-8; }
say(){ echo "$(date '+%T') $*" >> "$LOG"; }
"$H/s168-mock-build.sh" build m4b || { say "STOP build m4b rc=$(cat "$OUT/build-m4b.rc")"; touch "$OUT/chain3.done"; exit 7; }
say "build m4b ok $(tail -1 "$OUT/build-m4b.out" | grep -o 'hash=[0-9a-f]*')"
LOOP=$(PHPR_DUMP_OPS=1 "$BIN/phpr-m4b" "$R/wp164-harness/arith-dq.php" 2>&1 | sed -n '/^-- {main}/,/Ret/p' | sed -n '/CmpJmpSC/,/IncDecSlotJmp/p' | awk '{print $2}' | tr '\n' ' ')
say "loop m4b: $LOOP"
case "$LOOP" in *Sweep*) say "STOP: m4b NULLO (Sweep ancora nel loop)"; touch "$OUT/chain3.done"; exit 5;; esac
[ -e "$STASH/phpr-s168-m4b" ] || "$R/scripts/pin-phpr.sh" --braccio s168-m4b "$BIN/phpr-m4b" >> "$LOG" 2>&1 || { say "STOP braccio m4b"; touch "$OUT/chain3.done"; exit 1; }
"$R/wp160-harness/s160-disasm-bl.sh" "$BIN/phpr-m0" "$BIN/phpr-m4b" "$OUT/disasm-m4b.out" >> "$LOG" 2>&1
sleep 20
"$H/s168-ab-mock.sh" "$BIN/phpr-m0" "$(h8 "$BIN/phpr-m0")" "$BIN/phpr-m4b" "$(h8 "$BIN/phpr-m4b")" m4b; say "ab m4b rc=$?"
touch "$OUT/chain3.done"; say "fine"
