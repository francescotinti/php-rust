#!/bin/bash
# p4-coda2.sh — S-194 P4 after the declared corpus emend (s194-corpus-emenda-2026-11-25.out): day acceptance again (tag s194P4b), then
# the instruction count of the whole ORM and WordPress legs, arm P3 against arm P4 (the binaries p-coda.sh already copied). The tail of
# p-coda.sh, verbatim, from the acceptance on. Outcome: out/p4-coda2.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; T=P4; PV=P3; D="$W/out/p4-coda2.done"; : > "$D.tmp"; rm -f "$D"
REF="$W/out/phpr-p3"; ARM="$W/out/phpr-p4"
echo "braccio $(shasum -a 256 "$ARM" | cut -c1-16) rif $(shasum -a 256 "$REF" | cut -c1-16)" >> "$D.tmp"
bash "$W/../wp192-harness/s192-accettazione.sh" "s194P4b"; a=$?; echo "accettazione: rc=$a" >> "$D.tmp"
[ "$a" = 0 ] || { mv "$D.tmp" "$D"; exit 1; }
python3 "$W/s194-orm-istruzioni.py" "$PV=$REF" "$T=$ARM" > "$W/out/p-coda-$T-orm.log" 2>&1; echo "orm: rc=$?" >> "$D.tmp"
cp "$W/s194-orm-istruzioni.out" "$W/s194-orm-istruzioni-$T.out"
python3 "$W/s194-wp-istruzioni.py" "$PV=$REF" "$T=$ARM" > "$W/out/p-coda-$T-wp.log" 2>&1; echo "wp: rc=$?" >> "$D.tmp"
cp "$W/s194-wp-istruzioni.out" "$W/s194-wp-istruzioni-$T.out"
mv "$D.tmp" "$D"
