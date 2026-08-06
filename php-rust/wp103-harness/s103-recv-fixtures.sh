#!/bin/bash
# s103-recv-fixtures.sh — S-103 punto 2: le 2 fixture del PACCHETTO
# RICEVITORE (19a soglia-esatta-slot-held, 19b base=1) giudicate per
# BYTE-IDENTITA' oracle <-> phpr nei DUE modi. Set SEPARATO dalle 13
# (hc1-fixtures.sh) e dalle 5 MOVE (s102-move-fixtures.sh): quei gate
# restano pinnati ai loro NOMI (A-KL-103-2).
# rc: 0 = tutte identiche nei due modi; 1 = almeno una diverge (per NOME).
# GATE PINNATO: esattamente i 2 NOMI attesi o VOID (mai un conteggio).
set -u
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
F="$H/recv-fixtures"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
OUT="${OUT:-$F/out}"
mkdir -p "$OUT"
EXPECTED_NAMES="19a-soglia-esatta-slot-held 19b-base1-ricevitore-temporaneo"
seen=""
fails=0
names=""
for f in "$F"/*.php; do
  b="$(basename "$f" .php)"
  seen="$seen $b"
  "$ORACLE" -d error_reporting=E_ALL -d display_errors=1 -d log_errors=0 "$f" > "$OUT/$b.oracle" 2>&1
  env -u PHPR_REG_LOWER "$PHPR" "$f" > "$OUT/$b.on" 2>&1   # default = flag-on (env ASSENTE)
  PHPR_REG_LOWER=0 "$PHPR" "$f" > "$OUT/$b.off" 2>&1
  ok=1
  cmp -s "$OUT/$b.oracle" "$OUT/$b.on"  || { ok=0; names="$names $b:on"; }
  cmp -s "$OUT/$b.oracle" "$OUT/$b.off" || { ok=0; names="$names $b:off"; }
  if [ "$ok" = 1 ]; then echo "PASS $b"; else echo "FAIL $b"; fails=$((fails+1)); fi
done
if [ "$(echo $seen | tr ' ' '\n' | sort | tr '\n' ' ' | sed 's/ $//')" != "$(echo $EXPECTED_NAMES | tr ' ' '\n' | sort | tr '\n' ' ' | sed 's/ $//')" ]; then
  echo "GATE VOID: set fixture != i 2 NOMI pinnati (visti:$seen)"
  fails=$((fails+1))
fi
echo "recv_fixtures_fail=$fails"
[ -n "$names" ] && echo "divergenti per NOME:$names"
[ "$fails" = 0 ]
