#!/bin/bash
# s124-chain-admission.sh — catena DETACHED: build census → run conteggi →
# report admission. Ogni stadio scrive il suo rc su file; il chain si ferma
# al primo rc != 0. Progress in classifica-out/progress.txt.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
OUT="$H/classifica-out"; mkdir -p "$OUT"
rm -f "$OUT/chain.done"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }

p "chain admission start"
bash "$H/s124-classifica-build.sh"; rc=$?
p "build rc=$rc"
if [ "$rc" = 0 ]; then
  bash "$H/s124-classifica-run.sh"; rc=$?
  p "run rc=$rc"
fi
if [ "$rc" = 0 ]; then
  bash "$H/s124-classifica-report.sh"; rc=$?
  p "report rc=$rc"
fi
echo "$rc" > "$OUT/chain.done"
p "chain admission end rc=$rc"
