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

# ---- A-KL-104-2: hash FAIL-CLOSED — il gate si rifiuta senza pin atteso ----
if [ -z "${PHPR_PIN_ATTESO:-}" ]; then
  echo "GATE VOID: PHPR_PIN_ATTESO assente (hash fail-closed A-KL-104-2)"; exit 66
fi
H_PHPR="$(shasum -a 256 "$PHPR" | cut -c1-16)"
if [ "$H_PHPR" != "$PHPR_PIN_ATTESO" ]; then
  echo "GATE VOID: phpr=$H_PHPR != atteso $PHPR_PIN_ATTESO (A-KL-104-2)"; exit 66
fi
echo "phpr_pin=$H_PHPR (verificato)"

# ---- A-KL-104-3: MODE-PROBE — i due bracci PROVANO il loro modo o VOID ----
PROBE="$OUT/mode-probe.php"
printf '<?php\n$s=0; for ($i=0;$i<10;$i++) { $s = $s + $i; }\necho $s, "\\n";\n' > "$PROBE"
ON_REG=0;  env -u PHPR_REG_LOWER PHPR_DUMP_OPS=1 "$PHPR" "$PROBE" 2>&1 | grep -qE "BinarySS|BinarySC|BinaryDst|CmpJmpSC|CmpJmpSS" && ON_REG=1
OFF_REG=0; PHPR_REG_LOWER=0 PHPR_DUMP_OPS=1 "$PHPR" "$PROBE" 2>&1 | grep -qE "BinarySS|BinarySC|BinaryDst|CmpJmpSC|CmpJmpSS" && OFF_REG=1
if [ "$ON_REG" != 1 ] || [ "$OFF_REG" != 0 ]; then
  echo "GATE VOID: mode-probe fallita (on_reg=$ON_REG atteso 1, off_reg=$OFF_REG atteso 0) (A-KL-104-3)"; exit 66
fi
echo "mode-probe OK (on=registri, off=pila)"

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
