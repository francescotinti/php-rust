#!/bin/bash
# c3a-carichi.sh — S-194 C3a: whole-workload instruction counts (ORM, then WordPress), pin s187 against the C3a arm, started only
# after the acceptance s194C3a2 has written its rc. Outcome: out/c3a-carichi.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; D="$W/out/c3a-carichi.done"; rm -f "$D"
until [ -f "$W/../wp192-harness/out/accettazione-s194C3a2.rc" ]; do sleep 30; done
PIN="$HOME/Claude/php-rust-output/release/phpr"; ARM="$W/out/phpr-c3a"
python3 "$W/s194-orm-istruzioni.py" "pin=$PIN" "C3a=$ARM" > "$W/out/c3a-orm.log" 2>&1; a=$?
python3 "$W/s194-wp-istruzioni.py" "pin=$PIN" "C3a=$ARM" > "$W/out/c3a-wp.log" 2>&1; b=$?
echo "orm rc=$a wp rc=$b $(date '+%F %T')" > "$D"
