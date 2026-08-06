#!/bin/bash
# s104-fx20-gate.sh — la fixture NUOVA del gate H-C2 (KS-BA-105-1):
# fx20 stringhe-in-Pop, l'arbitro del leak che il giudice prop non vede.
# Byte-identita' oracle <-> phpr nei DUE modi. GATE PINNATO al NOME fx20.
# REGOLA PIN-105 (A-KL-105-3): pin BILATERALE fail-closed — anche l'oracle.
# rc: 0 = identico nei due modi; 1 = diverge; 66 = VOID.
set -u
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
F="$H/fixtures"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
OUT="${OUT:-$F/out}"
mkdir -p "$OUT"

# ---- pin bilaterale fail-closed (PIN-105) ----
if [ -z "${PHPR_PIN_ATTESO:-}" ] || [ -z "${ORACLE_PIN_ATTESO:-}" ]; then
  echo "GATE VOID: PHPR_PIN_ATTESO/ORACLE_PIN_ATTESO assente (PIN-105 fail-closed)"; exit 66
fi
H_PHPR="$(shasum -a 256 "$PHPR" | cut -c1-16)"
H_ORACLE="$(shasum -a 256 "$ORACLE" | cut -c1-16)"
if [ "$H_PHPR" != "$PHPR_PIN_ATTESO" ]; then
  echo "GATE VOID: phpr=$H_PHPR != atteso $PHPR_PIN_ATTESO"; exit 66
fi
if [ "$H_ORACLE" != "$ORACLE_PIN_ATTESO" ]; then
  echo "GATE VOID: oracle=$H_ORACLE != atteso $ORACLE_PIN_ATTESO"; exit 66
fi
echo "phpr_pin=$H_PHPR oracle_pin=$H_ORACLE (bilaterale, verificato)"

# ---- mode-probe (A-KL-104-3, ereditata) ----
PROBE="$OUT/mode-probe.php"
printf '<?php\n$s=0; for ($i=0;$i<10;$i++) { $s = $s + $i; }\necho $s, "\\n";\n' > "$PROBE"
ON_REG=0;  env -u PHPR_REG_LOWER PHPR_DUMP_OPS=1 "$PHPR" "$PROBE" 2>&1 | grep -qE "BinarySS|BinarySC|BinaryDst|CmpJmpSC|CmpJmpSS" && ON_REG=1
OFF_REG=0; PHPR_REG_LOWER=0 PHPR_DUMP_OPS=1 "$PHPR" "$PROBE" 2>&1 | grep -qE "BinarySS|BinarySC|BinaryDst|CmpJmpSC|CmpJmpSS" && OFF_REG=1
if [ "$ON_REG" != 1 ] || [ "$OFF_REG" != 0 ]; then
  echo "GATE VOID: mode-probe fallita (on_reg=$ON_REG, off_reg=$OFF_REG)"; exit 66
fi
echo "mode-probe OK (on=registri, off=pila)"

b="fx20-stringhe-in-pop"
f="$F/$b.php"
[ -f "$f" ] || { echo "GATE VOID: manca $f"; exit 66; }
"$ORACLE" -d error_reporting=E_ALL -d display_errors=1 -d log_errors=0 "$f" > "$OUT/$b.oracle" 2>&1
env -u PHPR_REG_LOWER "$PHPR" "$f" > "$OUT/$b.on" 2>&1
PHPR_REG_LOWER=0 "$PHPR" "$f" > "$OUT/$b.off" 2>&1
fails=0
cmp -s "$OUT/$b.oracle" "$OUT/$b.on"  || { echo "FAIL $b:on (byte-diff)";  fails=1; }
cmp -s "$OUT/$b.oracle" "$OUT/$b.off" || { echo "FAIL $b:off (byte-diff)"; fails=1; }
# l'oracle deve arbitrare l'output atteso (1x bool(true) + done) o VOID.
if [ "$(grep -c "bool(true)" "$OUT/$b.oracle")" != 1 ] || ! grep -q "fx20 done" "$OUT/$b.oracle"; then
  echo "GATE VOID: output oracle inatteso — la fixture non arbitra"; exit 66
fi
# ---- braccio RSS (KS-MA-105-1): il verdetto di LEAK vive QUI, non
# in-script (memory_get_usage di phpr e' uno stub costante — rosso del
# mutation-check archiviato in denti-rossi/). ~1M Str trattenute ≈
# +250 MiB: CAP pre-registrato 150 MiB (misure: clean ~<60, leaky 301).
# S-105 (A-MA-106-3, R-MA-106-3): il cap fisso 150 lasciava passare un leak
# PARZIALE (~+25-30 MiB, un solo sentiero Pop/overwrite ⇒ rss ~80). Banda a
# due soglie: PASS solo sotto GUARDIA=CAP/2 (75); [75,150) = EROSIONE della
# banda di guardia (FAIL: o leak parziale o pavimento clean cresciuto — in
# entrambi i casi si guarda, non si archivia); ≥150 = LEAK capitale.
# ⚠️ portabilità documentata: /usr/bin/time -l e ru_maxrss in BYTE sono
# macOS-only — su altro OS rss esce vuoto ⇒ GATE VOID (fail-closed VOLUTO).
CAP_MIB=150
GUARDIA_MIB=$((CAP_MIB / 2))
for mode in on off; do
  if [ "$mode" = on ]; then
    rss=$({ /usr/bin/time -l env -u PHPR_REG_LOWER "$PHPR" "$f" >/dev/null; } 2>&1 | awk '/maximum resident/{printf "%d", $1/1048576}')
  else
    rss=$({ /usr/bin/time -l env PHPR_REG_LOWER=0 "$PHPR" "$f" >/dev/null; } 2>&1 | awk '/maximum resident/{printf "%d", $1/1048576}')
  fi
  [ -n "$rss" ] || { echo "GATE VOID: RSS non misurabile ($mode)"; exit 66; }
  echo "rss_${mode}_MiB=$rss (guardia $GUARDIA_MIB, cap $CAP_MIB)"
  if [ "$rss" -ge "$CAP_MIB" ]; then
    echo "FAIL $b:$mode (RSS $rss >= cap $CAP_MIB: LEAK)"; fails=1
  elif [ "$rss" -ge "$GUARDIA_MIB" ]; then
    echo "FAIL $b:$mode (RSS $rss >= guardia $GUARDIA_MIB: EROSIONE — leak parziale o pavimento cresciuto)"; fails=1
  fi
done
if [ "$fails" = 0 ]; then echo "PASS $b (2 modi byte-id + RSS sotto cap, oracle arbitro valido)"; exit 0; fi
exit 1
