#!/bin/bash
# s100-assignop-oracle.sh — gamba a DUE MOTORI delle sette trappole AssignOp
# (KS-ST-100-2: «per NOME su ENTRAMBI i motori»). Ogni fixture gira su
# phpr flag-off, phpr flag-on (valori ESPLICITI, contratto S-100) e oracle.
# Le divergenze oracle PRE-ESISTENTI (di motore, identiche nei due modi,
# NON introdotte dal pass) sono ATTESE PER NOME qui sotto e catalogate in
# PHPR_DIVERGENCES_FROM_PHP.md (§S-100): una fixture attesa-divergente che
# COMBACIA o una nuova divergenza fanno FALLIRE il gate — mai un verde che
# copre un cambiamento.
#   b-typed-ref: Zend AZZERA il typed-ref dopo AssignOp fallito, phpr
#     conserva il valore; e-undef-warning-order: phpr non emette il warning
#     "Undefined variable" sul lhs undef di un AssignOp.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp100-harness"
T="$H/assignop-traps"
OUT="$H/assignop-oracle-out"
mkdir -p "$OUT"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
ORACLE=/opt/homebrew/opt/php/bin/php
EXPECTED_DIVERGENT="b-typed-ref e-undef-warning-order"
FAILS=0
for f in "$T"/*.php; do
  n="$(basename "$f" .php)"
  PHPR_REG_LOWER=0 "$PHPR" "$f" > "$OUT/$n-off.out" 2>&1
  PHPR_REG_LOWER=1 "$PHPR" "$f" > "$OUT/$n-on.out" 2>&1
  "$ORACLE" -d error_log= "$f" 2>&1 | grep -v '^PHP Warning:' | sed '/^$/d' > "$OUT/$n-oracle.out"
  sed '/^$/d' "$OUT/$n-off.out" > "$OUT/$n-off.norm"
  if ! cmp -s "$OUT/$n-off.out" "$OUT/$n-on.out"; then
    echo "$n: off!=on — BLOCCA IL FLIP (KS-ST-101-1)"; FAILS=$((FAILS+1)); continue
  fi
  case " $EXPECTED_DIVERGENT " in
    *" $n "*)
      if cmp -s "$OUT/$n-off.norm" "$OUT/$n-oracle.out"; then
        echo "$n: ATTESO DIVERGENTE ma ora COMBACIA con l'oracle — aggiornare lista e catalogo"; FAILS=$((FAILS+1))
      else
        echo "$n: divergenza oracle ATTESA (a catalogo), off==on OK"
      fi ;;
    *)
      if cmp -s "$OUT/$n-off.norm" "$OUT/$n-oracle.out"; then
        echo "$n: OK (due modi + oracle)"
      else
        echo "$n: DIVERGENZA ORACLE NUOVA — vedi diff:"; diff "$OUT/$n-off.norm" "$OUT/$n-oracle.out" | head -10; FAILS=$((FAILS+1))
      fi ;;
  esac
done
echo "ASSIGNOP-ORACLE DONE fails=$FAILS"
exit "$FAILS"
