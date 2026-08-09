#!/bin/bash
# s121-fx-preg-gate.sh — fixture preg L-RE1 a GATE FAIL-CLOSED (az. rev.
# S-120 #1, modello s105-fx21-gate.sh). 12 righe; le righe 2 e 11 sono le
# divergenze §3.18 A CATALOGO ((?J) nomi duplicati → false; nome utente col
# prefisso sintetico __phprbg nascosto). Il gate pinna QUATTRO golden IN repo:
#   fx-preg-riga{2,11}-phpr.golden   = righe phpr OGGI (divergenti) — alla
#                                      cura §3.18 il gate diventa ROSSO e la
#                                      cura DEVE aggiornare il golden nello
#                                      stesso commit.
#   fx-preg-riga{2,11}-oracle.golden = arbitro: se deriva, la fixture non
#                                      arbitra più ⇒ VOID.
# Le ALTRE 10 righe: byte-parity oracle<->phpr nei DUE modi, pena FAIL.
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

b="fx-preg-re1"
f="$F/$b.php"
[ -f "$f" ] || { echo "GATE VOID: manca $f"; exit 66; }
for g in 2 11; do
  [ -f "$F/fx-preg-riga$g-phpr.golden" ]   || { echo "GATE VOID: manca il golden phpr riga $g (si genera CONSAPEVOLMENTE, mai al volo)"; exit 66; }
  [ -f "$F/fx-preg-riga$g-oracle.golden" ] || { echo "GATE VOID: manca il golden oracle riga $g"; exit 66; }
done

"$ORACLE" -d error_reporting=E_ALL -d display_errors=1 -d log_errors=0 "$f" > "$OUT/$b.oracle" 2>&1
env -u PHPR_REG_LOWER "$PHPR" "$f" > "$OUT/$b.on" 2>&1
PHPR_REG_LOWER=0 "$PHPR" "$f" > "$OUT/$b.off" 2>&1

NL="$(wc -l < "$OUT/$b.oracle" | tr -d ' ')"
[ "$NL" = 12 ] || { echo "GATE VOID: oracle ha $NL righe (attese 12) — fixture non arbitra"; exit 66; }

for g in 2 11; do
  if ! cmp -s <(sed -n "${g}p" "$OUT/$b.oracle") "$F/fx-preg-riga$g-oracle.golden"; then
    echo "GATE VOID: riga $g oracle diversa dal golden arbitro"; exit 66
  fi
done

fails=0
for mode in on off; do
  if ! cmp -s <(sed '2d;11d' "$OUT/$b.oracle") <(sed '2d;11d' "$OUT/$b.$mode"); then
    echo "FAIL $b:$mode (byte-diff fuori dalle righe 2/11)"; fails=1
  fi
  for g in 2 11; do
    if ! cmp -s <(sed -n "${g}p" "$OUT/$b.$mode") "$F/fx-preg-riga$g-phpr.golden"; then
      echo "FAIL $b:$mode (riga $g != golden phpr — cura §3.18 atterrata o regressione: aggiornare il golden CONSAPEVOLMENTE nello stesso commit)"; fails=1
    fi
  done
done
if [ "$fails" = 0 ]; then echo "PASS $b (10 righe byte-id nei 2 modi; righe 2/11 = golden §3.18 pinnati)"; exit 0; fi
exit 1
