#!/bin/bash
# s99-corpus-gate.sh — gate corpus Zend per NOME per la rotazione pin post-
# sigillo eager (S-99.0). Ricetta S-98: insieme dei NOMI byte-uguale alla
# lista tracked wp82 (catena wp82→ha2→ha1→hb2 verificata), flag-OFF e
# flag-ON sullo stesso albero. Estrazione con tr -d '\0' (i log del runner
# contengono NUL) e MAI awk-$1 (i path contengono spazi).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
CORPUS="/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
REF="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp82-harness/p2-parity/corpus82p2.fails"
OUT="/Volumes/Extreme Pro/Claude/wp99-harness/corpus-gate"
mkdir -p "$OUT"
step(){ echo "$(date +%H:%M:%S) $1" | tee -a "$OUT/progress.txt"; }
FAILS=0

leg() { # $1=off|on
  local mode="$1" logf="$OUT/corpus-s99-$1.log" names="$OUT/corpus-s99-$1.fails"
  step "corpus $mode: run"
  if [ "$mode" = on ]; then
    /usr/bin/env PHPR_REG_LOWER=1 "$RUNNER" --isolate "$CORPUS" > "$logf" 2>&1
  else
    /usr/bin/env -u PHPR_REG_LOWER "$RUNNER" --isolate "$CORPUS" > "$logf" 2>&1
  fi
  # I path dei fail sono INDENTATI di due spazi sotto "failures: N" (morso
  # in S-99: la prima forma senza indent estraeva ZERO righe e il gate
  # urlava DIVERSO su un artefatto).
  tr -d '\0' < "$logf" | sed -n 's|^  \(/Volumes/.*\.phpt\)$|\1|p' | sort > "$names"
  tr -d '\0' < "$logf" | grep -E "pass rate|failures:" | tail -2 | tee -a "$OUT/progress.txt"
  if cmp -s "$names" <(sort "$REF"); then
    step "corpus $mode: $(wc -l < "$names" | tr -d ' ') fail, insieme IDENTICO alla lista wp82"
  else
    step "corpus $mode: DIVERSO — diff in corpus-$mode.diff"
    diff <(sort "$REF") "$names" > "$OUT/corpus-$mode.diff"
    FAILS=$((FAILS+1))
  fi
}

echo "phpr=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)" | tee -a "$OUT/progress.txt"
leg off
leg on
step "CORPUS GATE DONE fails=$FAILS"
echo "rc=$FAILS $(date +%T)" > "$OUT/corpus-gate.done"
exit "$FAILS"
