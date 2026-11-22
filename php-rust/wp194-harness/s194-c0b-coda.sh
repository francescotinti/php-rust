#!/bin/bash
# s194-c0b-coda.sh — S-194 package C, step C0b: wait for cargo to be quiet (the CI runner yields to this script by name), type-check
# with and without the instrumentation feature, LOC tooth, LOCAL commit, then the per-op instruction ledger of both workloads on that
# commit (branch tags of returns and calls). Outcomes: out/c0b-coda.out, out/c0b-coda.done, s194-registro-op.out.
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")" && pwd)"; V="$W/out/c0b-coda.out"; : > "$V"; rm -f "$W/out/c0b-coda.done"
fine() { echo "fine rc=$1 $(date '+%F %T')" >> "$V"; echo "$1" > "$W/out/c0b-coda.done"; exit "$1"; }
q=0; while [ "$q" -lt 2 ]; do if pgrep -qx cargo || pgrep -qx rustc; then q=0; else q=$((q + 1)); fi; sleep 30; done
cd "$W/.." || fine 2
bash "$W/out/r-check.sh" > "$W/out/c0b-check.txt" 2>&1
grep -c '^error' "$W/out/c0b-check.txt" >> "$V"; grep -q '^error' "$W/out/c0b-check.txt" && fine 3
cargo test --profile dev-release -p php-runtime --test loc_dente > "$W/out/c0b-loc.txt" 2>&1 || fine 4
git add -u; git restore --staged wp193-harness/catena-out/watchdog-coda.txt
git add wp194-harness/s194-c0b-coda.sh
git commit -q -F /private/tmp/s194-msg-c0b.txt || fine 5
C=$(git rev-parse HEAD); echo "commit $C" >> "$V"
mv "$W/s194-registro-op.out" "$W/out/registro-op-v2.out"; mv "$W/s194-registro-op-orm.txt" "$W/out/registro-op-orm-v2.txt"; mv "$W/s194-registro-op-wp.txt" "$W/out/registro-op-wp-v2.txt"
python3 "$W/s194-registro-op.py" "$C" orm wp
fine "$(cat "$W/out/registro-op.rc")"
