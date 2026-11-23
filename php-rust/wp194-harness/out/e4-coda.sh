#!/bin/bash
# e4-coda.sh — S-194 p.17: final acceptance of the tree with the cures, then whole-workload instruction counts of the E3 arm
# (ORM: C2b and E3, 2 legs each; WordPress: E3 only, 1 leg, read against the pin, C1 and C2 legs recorded the same day).
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; H="$W/../wp192-harness"; rm -f "$W/out/e4-coda.done"
cp "$HOME/Claude/phpr-target/s192-armD/release/phpr" "$W/out/phpr-e3" || exit 9
shasum -a 256 "$W/out/phpr-e3" | cut -c1-16 > "$W/out/phpr-e3.sha"
bash "$H/s192-accettazione.sh" s194E4 > "$W/out/e4-accettazione.log" 2>&1
a=$(cat "$H/out/accettazione-s194E4.rc" 2>/dev/null || echo 99)
python3 "$W/s194-orm-istruzioni.py" "C2=$W/out/phpr-c2b" "E=$W/out/phpr-e3" > "$W/out/e4-orm-istruzioni.log" 2>&1; o=$?
cp "$W/s194-orm-istruzioni.out" "$W/s194-orm-istruzioni-E.out"
python3 "$W/s194-wp-istruzioni.py" "E=$W/out/phpr-e3" > "$W/out/e4-wp-istruzioni.log" 2>&1; w=$?
cp "$W/s194-wp-istruzioni.out" "$W/s194-wp-istruzioni-E.out"
echo "accettazione=$a orm=$o wp=$w $(date '+%F %T')" > "$W/out/e4-coda.done"
