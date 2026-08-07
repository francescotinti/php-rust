#!/bin/bash
# s109-w9-fixtures.sh — azioni 3-4 revisore S-108: le 2 fixture del canale
# W9 (w9b TypeError con obj in pila; w9a ordine warning/eccezione) giudicate
# per BYTE-IDENTITA' oracle <-> phpr nei DUE modi. Set SEPARATO dai gate
# esistenti (hc1/move/recv/fx20/fx21), pinnato ai suoi 2 NOMI.
# In piu': PROBE DI FINESTRA — il dump ON della fixture deve contenere l'op
# fusa attesa (BinaryTCPropSetPop / PropGetSlotRecv), o la fixture non
# esercita il handler che dichiara di collaudare (GATE VOID).
# rc: 0 = identiche nei due modi; 1 = divergenza per NOME; 66 = VOID.
set -u
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
F="$H/w9-fixtures"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
OUT="${OUT:-$F/out}"
mkdir -p "$OUT"

# ---- hash FAIL-CLOSED (A-KL-104-2) ----
if [ -z "${PHPR_PIN_ATTESO:-}" ]; then
  echo "GATE VOID: PHPR_PIN_ATTESO assente (hash fail-closed A-KL-104-2)"; exit 66
fi
H_PHPR="$(shasum -a 256 "$PHPR" | cut -c1-16)"
if [ "$H_PHPR" != "$PHPR_PIN_ATTESO" ]; then
  echo "GATE VOID: phpr=$H_PHPR != atteso $PHPR_PIN_ATTESO (A-KL-104-2)"; exit 66
fi
echo "phpr_pin=$H_PHPR (verificato)"

# ---- MODE-PROBE (A-KL-104-3) ----
PROBE="$OUT/mode-probe.php"
printf '<?php\n$s=0; for ($i=0;$i<10;$i++) { $s = $s + $i; }\necho $s, "\\n";\n' > "$PROBE"
ON_REG=0;  env -u PHPR_REG_LOWER PHPR_DUMP_OPS=1 "$PHPR" "$PROBE" 2>&1 | grep -qE "BinarySS|BinarySC|BinaryDst|CmpJmpSC|CmpJmpSS" && ON_REG=1
OFF_REG=0; PHPR_REG_LOWER=0 PHPR_DUMP_OPS=1 "$PHPR" "$PROBE" 2>&1 | grep -qE "BinarySS|BinarySC|BinaryDst|CmpJmpSC|CmpJmpSS" && OFF_REG=1
if [ "$ON_REG" != 1 ] || [ "$OFF_REG" != 0 ]; then
  echo "GATE VOID: mode-probe fallita (on_reg=$ON_REG, off_reg=$OFF_REG)"; exit 66
fi
echo "mode-probe OK (on=registri, off=pila)"

# ---- PROBE DI FINESTRA: la fixture esercita l'op fusa che dichiara ----
probe_window() { # $1=file $2=op attesa
  local d="$OUT/dump-$(basename "$1" .php).txt"
  env -u PHPR_REG_LOWER PHPR_DUMP_OPS=1 "$PHPR" "$1" > "$d" 2>&1
  grep -q "$2" "$d"
}
if ! probe_window "$F/w9b-typeerror-obj-in-pila.php" "BinaryTCPropSetPop"; then
  echo "GATE VOID: w9b non emette BinaryTCPropSetPop nel dump ON"; exit 66
fi
if ! probe_window "$F/w9a-undef-get-lancia.php" "PropGetSlotRecv"; then
  echo "GATE VOID: w9a non emette PropGetSlotRecv nel dump ON"; exit 66
fi
echo "probe-finestra OK (w9b=BinaryTCPropSetPop, w9a=PropGetSlotRecv)"

EXPECTED_NAMES="w9a-undef-get-lancia w9b-typeerror-obj-in-pila"
seen=""
fails=0
names=""
for f in "$F"/*.php; do
  b="$(basename "$f" .php)"
  seen="$seen $b"
  "$ORACLE" -d error_reporting=E_ALL -d display_errors=1 -d log_errors=0 "$f" > "$OUT/$b.oracle" 2>&1
  env -u PHPR_REG_LOWER "$PHPR" "$f" > "$OUT/$b.on" 2>&1
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
echo "w9_fixtures_fail=$fails"
[ -n "$names" ] && echo "divergenti per NOME:$names"
[ "$fails" = 0 ]
