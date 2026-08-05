#!/bin/bash
# hc1-fixtures.sh — S-101 punto 3a: le fixture semantiche H-C1 (team hc-canale
# §4) giudicate per BYTE-IDENTITA' oracle <-> phpr nei DUE modi.
# rc: 0 = tutte identiche nei due modi; 1 = almeno una diverge (elenco per NOME).
# Ogni esito e' verificato sul FILE di output, mai sull'rc di una pipe.
set -u
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
F="$H/hc1-fixtures"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
OUT="${OUT:-$F/out}"
mkdir -p "$OUT"
fails=0
names=""
for f in "$F"/*.php; do
  b="$(basename "$f" .php)"
  "$ORACLE" -d error_reporting=E_ALL -d display_errors=1 -d log_errors=0 "$f" > "$OUT/$b.oracle" 2>&1
  env -u PHPR_REG_LOWER "$PHPR" "$f" > "$OUT/$b.on" 2>&1   # default = flag-on (env ASSENTE)
  PHPR_REG_LOWER=0 "$PHPR" "$f" > "$OUT/$b.off" 2>&1
  ok=1
  # Carve-out per NOME (§3.13, divergenza PRE-esistente identica nei 2 modi):
  # il diff DEVE essere ESATTAMENTE quello atteso, mai solo «non vuoto».
  exp="$F/$b.expected-divergence.diff"
  if [ -f "$exp" ]; then
    diff "$OUT/$b.oracle" "$OUT/$b.on"  > "$OUT/$b.on.diff"  2>&1
    diff "$OUT/$b.oracle" "$OUT/$b.off" > "$OUT/$b.off.diff" 2>&1
    cmp -s "$exp" "$OUT/$b.on.diff"  || { ok=0; names="$names $b:on"; }
    cmp -s "$exp" "$OUT/$b.off.diff" || { ok=0; names="$names $b:off"; }
  else
    cmp -s "$OUT/$b.oracle" "$OUT/$b.on"  || { ok=0; names="$names $b:on"; }
    cmp -s "$OUT/$b.oracle" "$OUT/$b.off" || { ok=0; names="$names $b:off"; }
  fi
  if [ "$ok" = 1 ]; then echo "PASS $b"; else echo "FAIL $b"; fails=$((fails+1)); fi
done
echo "fixtures_fail=$fails"
[ -n "$names" ] && echo "divergenti per NOME:$names"
[ "$fails" = 0 ]
