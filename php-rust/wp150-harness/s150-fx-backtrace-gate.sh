#!/bin/bash
# s150-fx-backtrace-gate.sh — fixture BT1 a GATE FAIL-CLOSED nella catena s109
# (az. NEXT §S-150 p.1: «fx-backtrace nel set»; modello s105-fx21-gate /
# s121-fx-preg-gate, ramo semplice: byte-parity PIENA oracle<->phpr nei DUE
# modi — la leva BT1 ha chiuso la divergenza options/limit, quindi nessuna
# riga a catalogo). Fixture: wp149-harness/fx-backtrace.php (bilaterale,
# giudicata BYTE-ID su B nell'A/B s149-ab-bt1-verdetto.out).
# rc: 0 = verde; 1 = FAIL; 66 = VOID. Pin BILATERALE fail-closed.
set -u
R="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
F="$R/wp149-harness/fx-backtrace.php"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
OUT="${OUT:-$R/wp150-harness/fx-out}"
mkdir -p "$OUT"

if [ -z "${PHPR_PIN_ATTESO:-}" ] || [ -z "${ORACLE_PIN_ATTESO:-}" ]; then
  echo "GATE VOID: PHPR_PIN_ATTESO/ORACLE_PIN_ATTESO assente (fail-closed)"; exit 66
fi
H_PHPR="$(shasum -a 256 "$PHPR" | cut -c1-16)"
H_ORACLE="$(shasum -a 256 "$ORACLE" | cut -c1-16)"
[ "$H_PHPR" = "$PHPR_PIN_ATTESO" ] || { echo "GATE VOID: phpr=$H_PHPR != atteso $PHPR_PIN_ATTESO"; exit 66; }
[ "$H_ORACLE" = "$ORACLE_PIN_ATTESO" ] || { echo "GATE VOID: oracle=$H_ORACLE != atteso $ORACLE_PIN_ATTESO"; exit 66; }
[ -f "$F" ] || { echo "GATE VOID: manca $F"; exit 66; }
echo "phpr_pin=$H_PHPR oracle_pin=$H_ORACLE (bilaterale, verificato)"

"$ORACLE" -d error_reporting=E_ALL -d display_errors=1 -d log_errors=0 "$F" > "$OUT/fx-backtrace.oracle" 2>&1
env -u PHPR_REG_LOWER "$PHPR" "$F" > "$OUT/fx-backtrace.on" 2>&1
PHPR_REG_LOWER=0 "$PHPR" "$F" > "$OUT/fx-backtrace.off" 2>&1

fails=0
for mode in on off; do
  if cmp -s "$OUT/fx-backtrace.oracle" "$OUT/fx-backtrace.$mode"; then
    :
  else
    echo "FAIL fx-backtrace:$mode (byte-diff vs oracle)"
    diff "$OUT/fx-backtrace.oracle" "$OUT/fx-backtrace.$mode" | head -10
    fails=1
  fi
done
if [ "$fails" = 0 ]; then echo "PASS fx-backtrace (byte-id vs oracle nei 2 modi)"; exit 0; fi
exit 1
