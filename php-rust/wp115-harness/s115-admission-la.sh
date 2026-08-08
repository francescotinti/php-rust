#!/bin/bash
# s115-admission-la.sh — criterio s115-criterio-la.md punto 6 (e): admission
# runtime mirata sul candidato CONSERVATO phpr-s114-la vs stash pin phpr-s112.
# (1) parita' stdout+stderr per hit + 4 miss; (2) dump {main} candidato vs pin
# IDENTICO per ogni caso (leva runtime-only); (3) bigramma PropGetSlotRecv+
# BinaryTCPropSetPop adiacente nel dump del PIN: gate su hit.php, a verbale
# sugli altri. rc DAL COMANDO, mai da pipe.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s112"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s114-la"
OUT="$H/admission-out"; mkdir -p "$OUT"
CASES="hit miss-magic miss-undef miss-ref miss-str"
RC=0

main_only() { awk '/^-- \{main\} /{f=1;next} /^-- /{f=0} f' "$1"; }
bigram() { awk '/PropGetSlotRecv/{p=1;next} p&&/BinaryTCPropSetPop/{print "SI";exit} {p=0}' "$1"; }

echo "A(pin)=$(shasum -a 256 "$A" | cut -c1-8)  B(cand)=$(shasum -a 256 "$B" | cut -c1-8)"
echo "== (1) parita' stdout+stderr candidato vs pin =="
for c in $CASES; do
  "$A" "$H/admission-cases/$c.php" > "$OUT/$c-pin.out" 2>&1
  ra=$?
  "$B" "$H/admission-cases/$c.php" > "$OUT/$c-cand.out" 2>&1
  rb=$?
  diff -q "$OUT/$c-pin.out" "$OUT/$c-cand.out" > /dev/null
  d=$?
  if [ "$d" -eq 0 ] && [ "$ra" -eq "$rb" ]; then
    echo "   $c: output IDENTICO (rc $ra=$rb)"
  else
    echo "   $c: DIVERGE (diff_rc=$d rc_pin=$ra rc_cand=$rb) — VIOLAZIONE"; RC=1
  fi
done
echo "== (2) dump {main} candidato vs pin IDENTICO + (3) bigramma nel pin =="
for c in $CASES; do
  PHPR_DUMP_OPS=1 "$A" "$H/admission-cases/$c.php" 2> "$OUT/$c-pin.dump" > /dev/null
  PHPR_DUMP_OPS=1 "$B" "$H/admission-cases/$c.php" 2> "$OUT/$c-cand.dump" > /dev/null
  main_only "$OUT/$c-pin.dump" > "$OUT/$c-pin.main"
  main_only "$OUT/$c-cand.dump" > "$OUT/$c-cand.main"
  cmp -s "$OUT/$c-pin.main" "$OUT/$c-cand.main"
  m=$?
  bg=$(bigram "$OUT/$c-pin.main"); [ -n "$bg" ] || bg=NO
  if [ "$m" -ne 0 ]; then
    echo "   $c: {main} DIVERGE tra candidato e pin — VIOLAZIONE (leva runtime-only)"; RC=1
  else
    echo "   $c: {main} identico · bigramma_nel_pin=$bg"
  fi
  if [ "$c" = hit ] && [ "$bg" != SI ]; then
    echo "   hit: bigramma ASSENTE nel {main} del pin — GATE FALLITO (il caso non esercita il sentiero fuso)"; RC=1
  fi
done
echo "admission_rc=$RC"
exit "$RC"
