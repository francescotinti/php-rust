#!/bin/bash
# c3b2-coda.sh — S-194 C3b final form (no scoped read in the ThisMethodCall arm of run_loop): arm build + probe against C3bbase,
# day acceptance, ORM instruction count pin against the arm. Outcome: out/c3b2-coda.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; D="$W/out/c3b2-coda.done"; : > "$D.tmp"; rm -f "$D"
PIN="$HOME/Claude/php-rust-output/release/phpr"; ARM="$W/out/phpr-c3b2"
bash "$W/s194-passo-C3b.sh" C3b2; echo "passo: rc=$? done=$(cat "$W/out/passo-C3b2.done")" >> "$D.tmp"
cp "$HOME/Claude/phpr-target/s192-armD/release/phpr" "$ARM"
bash "$W/../wp192-harness/s192-accettazione.sh" s194C3b2; echo "accettazione: rc=$?" >> "$D.tmp"
python3 "$W/s194-orm-istruzioni.py" "pin=$PIN" "C3b2=$ARM" > "$W/out/c3b2-orm.log" 2>&1; echo "orm: rc=$?" >> "$D.tmp"
mv "$D.tmp" "$D"
