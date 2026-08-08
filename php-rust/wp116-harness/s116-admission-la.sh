#!/bin/bash
# s116-admission-la.sh — criterio s116-criterio-la.md p.6 (INVARIATO da S-115
# p.6) + p.5: rc scritto in FILE (admission-out/rc), esito appeso al verbale
# s116-la-verdetto.out DALLO script a esito acquisito. Gate identita' binari.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s112"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s114-la"
CASES_DIR="$H/../wp115-harness/admission-cases"
OUT="$H/admission-out"; mkdir -p "$OUT"
VERD="$H/s116-la-verdetto.out"
CASES="hit miss-magic miss-undef miss-ref miss-str"
RC=0

HA=$(shasum -a 256 "$A" | cut -c1-8); HB=$(shasum -a 256 "$B" | cut -c1-8)
echo "A(pin)=$HA  B(cand)=$HB"
if [ "$HA" != f71abd2a ] || [ "$HB" != 052ea417 ]; then
  echo "IDENTITA' BINARI DIVERSA DAL CRITERIO — STOP"; echo 4 > "$OUT/rc"; exit 4
fi

main_only() { awk '/^-- \{main\} /{f=1;next} /^-- /{f=0} f' "$1"; }
bigram() { awk '/PropGetSlotRecv/{p=1;next} p&&/BinaryTCPropSetPop/{print "SI";exit} {p=0}' "$1"; }

echo "== (1) parita' stdout+stderr candidato vs pin =="
for c in $CASES; do
  "$A" "$CASES_DIR/$c.php" > "$OUT/$c-pin.out" 2>&1
  ra=$?
  "$B" "$CASES_DIR/$c.php" > "$OUT/$c-cand.out" 2>&1
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
BIGRAMS=""
for c in $CASES; do
  PHPR_DUMP_OPS=1 "$A" "$CASES_DIR/$c.php" 2> "$OUT/$c-pin.dump" > /dev/null
  PHPR_DUMP_OPS=1 "$B" "$CASES_DIR/$c.php" 2> "$OUT/$c-cand.dump" > /dev/null
  main_only "$OUT/$c-pin.dump" > "$OUT/$c-pin.main"
  main_only "$OUT/$c-cand.dump" > "$OUT/$c-cand.main"
  cmp -s "$OUT/$c-pin.main" "$OUT/$c-cand.main"
  m=$?
  bg=$(bigram "$OUT/$c-pin.main"); [ -n "$bg" ] || bg=NO
  BIGRAMS="$BIGRAMS $c=$bg"
  if [ "$m" -ne 0 ]; then
    echo "   $c: {main} DIVERGE tra candidato e pin — VIOLAZIONE (leva runtime-only)"; RC=1
  else
    echo "   $c: {main} identico · bigramma_nel_pin=$bg"
  fi
  if [ "$c" = hit ] && [ "$bg" != SI ]; then
    echo "   hit: bigramma ASSENTE nel {main} del pin — GATE FALLITO"; RC=1
  fi
done
echo "$RC" > "$OUT/rc"
echo "admission_rc=$RC (scritto in admission-out/rc)"
{
  echo "admission (p.6, rc=$RC da admission-out/rc): A=$HA B=$HB · parita' output e dump {main} x5: $([ "$RC" -eq 0 ] && echo TUTTI IDENTICI || echo VIOLAZIONE) · bigramma:$BIGRAMS · batteria RIUSATA 1742/0/2 S-114 (byte-identico — deroga dichiarata)"
} >> "$VERD"
exit "$RC"
