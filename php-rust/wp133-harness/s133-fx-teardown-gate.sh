#!/bin/bash
# s133-fx-teardown-gate.sh — fixture destructor-window (az. rev. S-132 #1) a
# GATE FAIL-CLOSED, modello s121-fx-preg-gate.sh. Sette vettori mirati alla
# finestra di teardown GC (detach props a layout vuoto, vm/mod.rs destroy
# phase): dtor che riscrive la propria prop dichiarata leaf→non-leaf (v1),
# la prop del peer (v2), gc annidata nel dtor (v3), resurrezione (v4),
# dtor-walk di fine script (v5), WeakReference post-destroy (v6), doppio
# gc annidato con dtor del peer pendente (v7). In S-133 tutti e sette sono
# BYTE-IDENTICI oracle==phpr nei due modi (s131 E s132): il gate pinna la
# parità piena, senza golden di divergenza. Se una cura futura sposta uno di
# questi esiti, il gate diventa ROSSO e la cura lo emenda DICHIARANDO.
# rc: 0 = verde; 1 = FAIL; 66 = VOID. Pin BILATERALE fail-closed.
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

FIXTURES="v1-self-dtor v2-other-dtor v3-nested-gc v4-resurrect v5-shutdown v6-weakref v7-nested-peer"
fails=0
n=0
for b in $FIXTURES; do
  f="$F/$b.php"
  [ -f "$f" ] || { echo "GATE VOID: manca $f"; exit 66; }
  "$ORACLE" -d error_reporting=E_ALL -d display_errors=1 -d log_errors=0 "$f" > "$OUT/$b.oracle" 2>&1
  env -u PHPR_REG_LOWER "$PHPR" "$f" > "$OUT/$b.on" 2>&1
  PHPR_REG_LOWER=0 "$PHPR" "$f" > "$OUT/$b.off" 2>&1
  for mode in on off; do
    if ! cmp -s "$OUT/$b.oracle" "$OUT/$b.$mode"; then
      echo "FAIL $b:$mode (byte-diff vs oracle)"; fails=1
    fi
  done
  n=$((n + 1))
done
[ "$n" = 7 ] || { echo "GATE VOID: eseguite $n fixture (attese 7)"; exit 66; }
if [ "$fails" = 0 ]; then echo "PASS teardown (7 fixture byte-id nei 2 modi)"; exit 0; fi
exit 1
