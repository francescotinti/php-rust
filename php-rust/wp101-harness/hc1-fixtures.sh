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

fails=0
names=""
# A-KL-103-2 (S-102): il set e' PINNATO ai 13 NOMI o il gate e' VOID — un
# conteggio nudo non vede una fixture sparita E una comparsa nello stesso run.
EXPECTED_NAMES="01-specie-int 02-specie-string 03-specie-array 04-alias-interleaved 05-alias-magic-hook 06-alias-lazy 07-alias-ref-slot 08-alias-reassign-intra 09-unset-during-read 10-readonly-typed-uninit 11-stack-inplace 12-destruct-order 13-gc-cycle-prop"
seen=""
for f in "$F"/*.php; do
  b="$(basename "$f" .php)"
  seen="$seen $b"
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
# gate pinnato per NOME o VOID (A-KL-103-2)
if [ "$(echo $seen | tr ' ' '\n' | sort | tr '\n' ' ' | sed 's/ $//')" != "$(echo $EXPECTED_NAMES | tr ' ' '\n' | sort | tr '\n' ' ' | sed 's/ $//')" ]; then
  echo "GATE VOID: set fixture != i 13 NOMI pinnati (visti:$seen)"
  fails=$((fails+1))
fi
echo "fixtures_fail=$fails"
[ -n "$names" ] && echo "divergenti per NOME:$names"
[ "$fails" = 0 ]
