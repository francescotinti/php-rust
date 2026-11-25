#!/bin/bash
# p-coda.sh <tag> <prev> — S-194 package P, one step after its code is written: s194-passo-P1.sh <tag> (arm build + whole probe against
# the pin reading), day acceptance s194<tag>, then the instruction count of the whole ORM and WordPress legs, the arm of the previous
# step (out/phpr-<prev>, already measured: its leg is repeated as the reference) against this one. Generalises p1-coda.sh.
# Outcome: out/p-coda-<tag>.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; T="$1"; PV="$2"; D="$W/out/p-coda-$T.done"; : > "$D.tmp"; rm -f "$D"
lc() { echo "$1" | tr 'A-Z' 'a-z'; }
REF="$W/out/phpr-$(lc "$PV")"; ARM="$W/out/phpr-$(lc "$T")"
bash "$W/s194-passo-P1.sh" "$T"; echo "passo: rc=$? done=$(cat "$W/out/passo-$T.done")" >> "$D.tmp"
[ "$(cat "$W/out/passo-$T.done")" = 0 ] || { mv "$D.tmp" "$D"; exit 1; }
cp "$HOME/Claude/phpr-target/s192-armD/release/phpr" "$ARM"; echo "braccio $(shasum -a 256 "$ARM" | cut -c1-16) rif $(shasum -a 256 "$REF" | cut -c1-16)" >> "$D.tmp"
bash "$W/../wp192-harness/s192-accettazione.sh" "s194$T"; a=$?; echo "accettazione: rc=$a" >> "$D.tmp"
[ "$a" = 0 ] || { mv "$D.tmp" "$D"; exit 1; }
python3 "$W/s194-orm-istruzioni.py" "$PV=$REF" "$T=$ARM" > "$W/out/p-coda-$T-orm.log" 2>&1; echo "orm: rc=$?" >> "$D.tmp"
cp "$W/s194-orm-istruzioni.out" "$W/s194-orm-istruzioni-$T.out"
python3 "$W/s194-wp-istruzioni.py" "$PV=$REF" "$T=$ARM" > "$W/out/p-coda-$T-wp.log" 2>&1; echo "wp: rc=$?" >> "$D.tmp"
cp "$W/s194-wp-istruzioni.out" "$W/s194-wp-istruzioni-$T.out"
mv "$D.tmp" "$D"
