#!/bin/bash
# s126-orm-profile.sh — indizio UNILATERALE (criterio s126-criterio-orm.md p.5):
# 2 campioni `sample` da 25 s del phpr mentre esegue la suite doctrine/orm.
# NESSUNA cifra di tempo da questo run (il sampling perturba): solo quote
# top-of-stack, a pesare le categorie dell'istruttoria.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
OUT="$H/orm-out"; mkdir -p "$OUT"
VERD="$H/s126-orm-profile-verdetto.out"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
SP="${PROF_SP:?PROF_SP (workdir) richiesto}"
rm -f "$OUT/profile.done"
[ "$(shasum -a 256 "$PHPR" | cut -c1-16)" = "002e6cc12047ab9f" ] || { echo "rc=9 pin!=s125" > "$OUT/profile.done"; exit 9; }

rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=8 untar" > "$OUT/profile.done"; exit 8; }
cd "$SP/orm-work" || { echo "rc=7 cd" > "$OUT/profile.done"; exit 7; }
"$PHPR" vendor/bin/phpunit --no-coverage > "$OUT/profile-run.txt" 2>&1 &
PID=$!
sleep 4
sample "$PID" 25 -file "$OUT/sample1.txt" > "$OUT/sample1.log" 2>&1
sample "$PID" 25 -file "$OUT/sample2.txt" > "$OUT/sample2.log" 2>&1
wait "$PID"; RUNRC=$?

{
echo "== s126 profilo ORM phpr (INDIZIO unilaterale — criterio p.5; MAI cifra) =="
echo "grade=INDIZIO  # sample perturba: nessun tempo citabile da questo run"
echo "phpr=$(shasum -a 256 "$PHPR" | cut -c1-16) run_rc=$RUNRC"
echo "summary_run=$(tr -d '\0' < "$OUT/profile-run.txt" | grep -E '^(Tests:|OK)' | tail -1)"
for s in 1 2; do
  echo "--- sample$s top-of-stack (prime 30 righe) ---"
  sed -n '/Sort by top of stack/,/Binary Images/p' "$OUT/sample$s.txt" | head -32
done
} > "$VERD" 2>&1
echo "rc=$RUNRC $(date +%T)" > "$OUT/profile.done"
