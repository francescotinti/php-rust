#!/bin/bash
# s109-fixture-chain.sh — le 5 fixture storiche + le 2 W9 (azioni 3-4 revisore
# S-108) + il gate preg §3.18 (az. rev. S-121 #4, cablato S-122) sul PIN.
# rc di OGNI gate DAL COMANDO (mai da pipe); marker finale rc=somma.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
R="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
export PHPR_PIN_ATTESO="${PHPR_PIN_ATTESO:?serve il pin atteso}"
export ORACLE_PIN_ATTESO="${ORACLE_PIN_ATTESO:-07b0df8d63247695}"
TOT=0
run_gate() { # $1=etichetta $2=script
  echo "== gate $1 =="
  "$2"
  local rc=$?
  echo "-- $1 rc=$rc"
  TOT=$((TOT + rc))
}
run_gate hc1  "$R/wp101-harness/hc1-fixtures.sh"
run_gate move "$R/wp102-harness/s102-move-fixtures.sh"
run_gate recv "$R/wp103-harness/s103-recv-fixtures.sh"
run_gate fx20 "$R/wp104-harness/s104-fx20-gate.sh"
run_gate fx21 "$R/wp105-harness/s105-fx21-gate.sh"
run_gate w9   "$R/wp109-harness/s109-w9-fixtures.sh"
run_gate preg "$R/wp121-harness/s121-fx-preg-gate.sh"
echo "FIXTURE-CHAIN rc_totale=$TOT (0 = tutte verdi)"
exit "$TOT"
