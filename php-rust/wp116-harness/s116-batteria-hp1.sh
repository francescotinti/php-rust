#!/bin/bash
# s116-batteria-hp1.sh — finestra S-116: batteria N=3 sul tree H-P1 (8bb395c)
# per stanare il flaky del 1741 S-113 (az.4 rev. S-113; apertura S-114/115).
# Worktree DEDICATO + CARGO_TARGET_DIR separato (MAI ~/Claude/php-rust-output:
# il pin non si tocca) + CARGO_INCREMENTAL=0 (vincolo hard-link Extreme Pro).
# rc di OGNI run dal comando in FILE (batteria-out/hp1-runN.rc); inventario
# per NOME per run + diff vs inventario S-114; esiti appesi al verbale
# s116-batteria-verdetto.out DALLO script a esiti acquisiti.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"
WT="/Volumes/Extreme Pro/Claude/phpr-s116-hp1-wt"
TGT="/Volumes/Extreme Pro/Claude/phpr-s116-hp1-target"
OUT="$H/batteria-out"; mkdir -p "$OUT"
VERD="$H/s116-batteria-verdetto.out"
REF114="$H/../wp114-harness/batteria-out/hp1-batteria-nomi.txt"
N=3

if [ ! -d "$WT" ]; then git -C "$REPO" worktree add "$WT" 8bb395c || exit 4; fi
HEADH=$(git -C "$WT" rev-parse --short HEAD)
if [ "$HEADH" != 8bb395c ]; then echo "worktree HEAD=$HEADH != 8bb395c — STOP"; echo 4 > "$OUT/rc"; exit 4; fi
echo "worktree=$WT HEAD=$HEADH target=$TGT N=$N"

cd "$WT/php-rust" || exit 4
summ() { awk '/^test result:/{p+=$4; f+=$6; ig+=$8} END{printf "%d passed / %d failed / %d ignored", p, f, ig}' "$1"; }
nomi() { grep -E '^test .* \.\.\. ' "$1" | sed 's/^test //; s/ \.\.\..*//' | sort > "$2"; wc -l < "$2" | tr -d ' '; }

for i in $(seq 1 "$N"); do
  CARGO_TARGET_DIR="$TGT" CARGO_INCREMENTAL=0 cargo test --release > "$OUT/hp1-run$i.log" 2>&1
  rc=$?
  echo "$rc" > "$OUT/hp1-run$i.rc"
  cnt=$(summ "$OUT/hp1-run$i.log")
  nn=$(nomi "$OUT/hp1-run$i.log" "$OUT/hp1-run$i-nomi.txt")
  echo "run$i: rc=$rc · $cnt · $nn nomi"
  echo "run$i: rc=$rc (da hp1-run$i.rc) · $cnt · $nn nomi" >> "$OUT/esiti.txt"
done

DIFFS=""
for i in 2 3; do
  if ! diff -q "$OUT/hp1-run1-nomi.txt" "$OUT/hp1-run$i-nomi.txt" > /dev/null; then
    DIFFS="$DIFFS run1-vs-run$i"
    diff "$OUT/hp1-run1-nomi.txt" "$OUT/hp1-run$i-nomi.txt" > "$OUT/diff-run1-run$i.txt"
  fi
done
V114="n/d"
if [ -f "$REF114" ]; then
  if diff -q "$OUT/hp1-run1-nomi.txt" "$REF114" > /dev/null; then V114="IDENTICO"; else V114="DIVERGE (vedi diff-vs-s114.txt)"; diff "$OUT/hp1-run1-nomi.txt" "$REF114" > "$OUT/diff-vs-s114.txt"; fi
fi
RCT=0; for i in $(seq 1 "$N"); do r=$(cat "$OUT/hp1-run$i.rc"); [ "$r" != 0 ] && RCT=1; done
[ -n "$DIFFS" ] && RCT=1
echo "$RCT" > "$OUT/rc"
echo "batteria_rc=$RCT (scritto in batteria-out/rc) · diff inventari tra run:${DIFFS:- NESSUNO} · vs S-114: $V114"
{
  echo "# S-116 batteria N=$N su 8bb395c (tree H-P1; worktree dedicato, target separato, INCREMENTAL=0; rc per run in batteria-out/hp1-runN.rc)"
  cat "$OUT/esiti.txt"
  echo "diff inventari per NOME tra le $N run:${DIFFS:- NESSUNO} · run1 vs inventario S-114: $V114 · rc_complessivo=$RCT"
} >> "$VERD"
exit "$RCT"
