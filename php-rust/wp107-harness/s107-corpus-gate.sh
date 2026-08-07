#!/bin/bash
# s107-corpus-gate.sh — gate corpus Zend per NOME, versione BIMODALE del
# contratto di modo S-100 (A-KL-101-6): OGNI gamba spende PHPR_REG_LOWER
# per VALORE ESPLICITO dalla lista chiusa {0,1} — mai l'assenza, che dopo
# il flip del default farebbe girare le due gambe nello STESSO modo
# (falso verde stesso-modo, R1 Hejlsberg). Ricetta S-98 invariata per il
# resto: insieme dei NOMI byte-uguale alla lista tracked wp82, estrazione
# con tr -d '\0' (log con NUL), path INDENTATI di 2 spazi, MAI awk-$1.
# A-KL-101-3: assert conteggi↔nomi su OGNI gamba (estratti == dichiarati
# dallo strumento), pena gate VOID (KS-KL-101-3).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
CORPUS="/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
REF="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp82-harness/p2-parity/corpus82p2.fails"
OUT="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp107-harness/corpus-gate"
mkdir -p "$OUT"
step(){ echo "$(date +%H:%M:%S) $1" | tee -a "$OUT/progress.txt"; }
FAILS=0

leg() { # $1=off|on — modo ESPLICITO (contratto value-parsed: off=>0, on=>1)
  local mode="$1" logf="$OUT/corpus-s107-$1.log" names="$OUT/corpus-s107-$1.fails" reg
  case "$mode" in off) reg=0 ;; on) reg=1 ;; *) step "modo ignoto $mode"; exit 64 ;; esac
  step "corpus $mode (PHPR_REG_LOWER=$reg esplicito): run"
  /usr/bin/env PHPR_REG_LOWER="$reg" "$RUNNER" --isolate "$CORPUS" > "$logf" 2>&1
  # I path dei fail sono INDENTATI di due spazi sotto "failures: N" (morso S-99).
  tr -d '\0' < "$logf" | sed -n 's|^  \(/Volumes/.*\.phpt\)$|\1|p' | sort > "$names"
  tr -d '\0' < "$logf" | grep -E "pass rate|failures:" | tail -2 | tee -a "$OUT/progress.txt"
  # A-KL-101-3: i nomi estratti devono quadrare col conteggio dichiarato.
  local n_decl n_extr
  n_decl="$(tr -d '\0' < "$logf" | sed -n 's/^failures: \([0-9]*\).*/\1/p' | tail -1)"
  n_extr="$(wc -l < "$names" | tr -d ' ')"
  if [ -z "$n_decl" ] || [ "$n_decl" != "$n_extr" ]; then
    step "corpus $mode: GATE VOID — nomi estratti ($n_extr) != dichiarati (${n_decl:-assente}) (KS-KL-101-3)"
    FAILS=$((FAILS+1)); return
  fi
  if cmp -s "$names" <(sort "$REF"); then
    step "corpus $mode: $n_extr fail, insieme IDENTICO alla lista wp82"
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
