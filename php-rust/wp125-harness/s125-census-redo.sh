#!/bin/bash
# s125-census-redo.sh — recupero del ramo census del p.2 (incidente PRE:
# mv dei ratios TRACCIATI wp109 a fine coppia; ripristinati). Sequenziale:
# census-build → census-run → census-report; poi il chain cbargs si rilancia.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp125-harness"
p(){ echo "$(date +%H:%M:%S) $1" >> "$H/p2-progress.txt"; }
rm -f "$H/census-redo.done"
p "census-REDO START (post-incidente ratios)"
"$H/s125-census-build.sh" >> "$H/p2-progress.txt" 2>&1; b=$?
p "census-build(redo) rc=$b"
RC=1
if [ "$b" = 0 ]; then
  "$H/s125-census-run.sh" >> "$H/p2-progress.txt" 2>&1; r=$?
  p "census-run(redo) rc=$r"
  if [ "$r" = 0 ]; then
    "$H/s125-census-report.sh" >> "$H/p2-progress.txt" 2>&1; rep=$?
    p "census-report(redo) rc=$rep (0=confermata 2=grado-mancato)"
    [ "$rep" != 1 ] && RC=0
  fi
fi
echo "rc=$RC $(date +%T)" > "$H/census-redo.done"
p "census-REDO DONE rc=$RC"
