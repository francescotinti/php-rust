#!/bin/bash
# s144-parser-golden-test.sh — golden-test del parser census v3 (az.rev.
# S-143 #4: committato INSIEME al criterio, PRIMA della run). Le fixture
# contengono righe tag=exit_mi: un parser regredito al match-substring
# raddoppia le chiavi e il diff MORDE. Esito ESATTO, mai «diverso da»
# (feedback-forge-silent-failure).
set -u
H="$(cd "$(dirname "$0")" && pwd)"
RC=0
for case in case1 case2; do
  got="$("/usr/bin/python3" "$H/s144-census-parse.py" "$H/s144-parser-golden/$case")"
  if diff -u "$H/s144-parser-golden/$case/expected.txt" <(printf '%s\n' "$got"); then
    echo "GOLDEN $case PASS"
  else
    echo "GOLDEN $case FAIL"
    RC=1
  fi
done
exit $RC
