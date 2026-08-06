#!/bin/bash
# s105-fx21-gate.sh — fx21-args-window a GATE FAIL-CLOSED (S-106-D-14 ≡
# A-KL-107-3 ∘ T-PS-107-6, modello s104-fx20-gate.sh).
#
# La fixture ha 8 righe di output. La RIGA 5 (vref, variadic by-ref) è la
# divergenza §3.15 A CATALOGO: phpr aliasa solo il 1° argomento del pack.
# Il gate pinna DUE golden IN repo:
#   fx21-riga5-phpr.golden   = riga 5 di phpr OGGI (divergente) — alla cura
#                              §3.15 il gate diventa ROSSO e la cura DEVE
#                              aggiornare il golden nello stesso commit
#                              (attesa: diventerà = golden oracle; D-8/
#                              KS-ST-107-2: toccare UNA sola copia della
#                              semantica di bind ⇒ questo gate è il dente).
#   fx21-riga5-oracle.golden = riga 5 dell'oracle (arbitro): se cambia,
#                              la fixture non arbitra più ⇒ VOID.
# Le ALTRE 7 righe: byte-parity oracle<->phpr nei DUE modi, pena FAIL.
# rc: 0 = verde; 1 = FAIL; 66 = VOID. Pin BILATERALE fail-closed (PIN-105).
set -u
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
F="$H/fixtures"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
OUT="${OUT:-$F/out}"
mkdir -p "$OUT"

# ---- pin bilaterale fail-closed ----
if [ -z "${PHPR_PIN_ATTESO:-}" ] || [ -z "${ORACLE_PIN_ATTESO:-}" ]; then
  echo "GATE VOID: PHPR_PIN_ATTESO/ORACLE_PIN_ATTESO assente (fail-closed)"; exit 66
fi
H_PHPR="$(shasum -a 256 "$PHPR" | cut -c1-16)"
H_ORACLE="$(shasum -a 256 "$ORACLE" | cut -c1-16)"
[ "$H_PHPR" = "$PHPR_PIN_ATTESO" ] || { echo "GATE VOID: phpr=$H_PHPR != atteso $PHPR_PIN_ATTESO"; exit 66; }
[ "$H_ORACLE" = "$ORACLE_PIN_ATTESO" ] || { echo "GATE VOID: oracle=$H_ORACLE != atteso $ORACLE_PIN_ATTESO"; exit 66; }
echo "phpr_pin=$H_PHPR oracle_pin=$H_ORACLE (bilaterale, verificato)"

# ---- mode-probe (ereditata da fx20) ----
PROBE="$OUT/mode-probe.php"
printf '<?php\n$s=0; for ($i=0;$i<10;$i++) { $s = $s + $i; }\necho $s, "\\n";\n' > "$PROBE"
ON_REG=0;  env -u PHPR_REG_LOWER PHPR_DUMP_OPS=1 "$PHPR" "$PROBE" 2>&1 | grep -qE "BinarySS|BinarySC|BinaryDst|CmpJmpSC|CmpJmpSS" && ON_REG=1
OFF_REG=0; PHPR_REG_LOWER=0 PHPR_DUMP_OPS=1 "$PHPR" "$PROBE" 2>&1 | grep -qE "BinarySS|BinarySC|BinaryDst|CmpJmpSC|CmpJmpSS" && OFF_REG=1
if [ "$ON_REG" != 1 ] || [ "$OFF_REG" != 0 ]; then
  echo "GATE VOID: mode-probe fallita (on_reg=$ON_REG, off_reg=$OFF_REG)"; exit 66
fi
echo "mode-probe OK (on=registri, off=pila)"

b="fx21-args-window"
f="$F/$b.php"
GP="$F/fx21-riga5-phpr.golden"
GO="$F/fx21-riga5-oracle.golden"
[ -f "$f" ]  || { echo "GATE VOID: manca $f"; exit 66; }
[ -f "$GP" ] || { echo "GATE VOID: manca il golden $GP (fail-closed: si genera CONSAPEVOLMENTE, mai al volo)"; exit 66; }
[ -f "$GO" ] || { echo "GATE VOID: manca il golden $GO"; exit 66; }

"$ORACLE" -d error_reporting=E_ALL -d display_errors=1 -d log_errors=0 "$f" > "$OUT/$b.oracle" 2>&1
env -u PHPR_REG_LOWER "$PHPR" "$f" > "$OUT/$b.on" 2>&1
PHPR_REG_LOWER=0 "$PHPR" "$f" > "$OUT/$b.off" 2>&1

# l'oracle deve avere esattamente 8 righe o la fixture non arbitra
NL="$(wc -l < "$OUT/$b.oracle" | tr -d ' ')"
[ "$NL" = 8 ] || { echo "GATE VOID: oracle ha $NL righe (attese 8) — fixture non arbitra"; exit 66; }

fails=0
# riga 5 oracle = golden arbitro (VOID se deriva)
if ! cmp -s <(sed -n '5p' "$OUT/$b.oracle") "$GO"; then
  echo "GATE VOID: riga 5 oracle diversa dal golden arbitro"; exit 66
fi
for mode in on off; do
  # righe 1-4 e 6-8: byte-parity con l'oracle
  if ! cmp -s <(sed '5d' "$OUT/$b.oracle") <(sed '5d' "$OUT/$b.$mode"); then
    echo "FAIL $b:$mode (byte-diff fuori dalla riga 5)"; fails=1
  fi
  # riga 5: pinnata al golden phpr (§3.15 a catalogo; ROSSO alla cura)
  if ! cmp -s <(sed -n '5p' "$OUT/$b.$mode") "$GP"; then
    echo "FAIL $b:$mode (riga 5 != golden phpr — cura §3.15 atterrata o regressione: aggiornare il golden CONSAPEVOLMENTE nello stesso commit)"; fails=1
  fi
done
if [ "$fails" = 0 ]; then echo "PASS $b (7 righe byte-id nei 2 modi; riga 5 = golden §3.15 pinnato)"; exit 0; fi
exit 1
