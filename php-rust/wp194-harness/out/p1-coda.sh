#!/bin/bash
# p1-coda.sh — S-194 P1 (one-pass isset/empty of a mixed path): day acceptance, then the instruction count of the whole ORM and
# WordPress legs, pin s188 against the arm built by s194-passo-P1.sh. Declared copy of c3b2-coda.sh without the step (already run)
# and with the WordPress count added. Outcome: out/p1-coda.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; D="$W/out/p1-coda.done"; : > "$D.tmp"; rm -f "$D"
PIN="$HOME/Claude/php-rust-output/release/phpr"; ARM="$W/out/phpr-p1"
cp "$HOME/Claude/phpr-target/s192-armD/release/phpr" "$ARM"; echo "braccio $(shasum -a 256 "$ARM" | cut -c1-16)" >> "$D.tmp"
bash "$W/../wp192-harness/s192-accettazione.sh" s194P1; echo "accettazione: rc=$?" >> "$D.tmp"
python3 "$W/s194-orm-istruzioni.py" "pin=$PIN" "P1=$ARM" > "$W/out/p1-orm.log" 2>&1; echo "orm: rc=$?" >> "$D.tmp"
python3 "$W/s194-wp-istruzioni.py" "pin=$PIN" "P1=$ARM" > "$W/out/p1-wp.log" 2>&1; echo "wp: rc=$?" >> "$D.tmp"
mv "$D.tmp" "$D"
