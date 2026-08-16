#!/bin/bash
# s148-parse-golden-test.sh — golden del parser s148-parse.py (criterio p.6:
# parser committato + golden PRIMA del run). Fixture in golden/ con numeri
# calcolati A MANO: identita' 70+30==100 per pid sommati; none n=60 attr=20
# other=40 (40%); frame other=20; ranking decrescente; hist arrgrow <=48.
# rc=0 solo se TUTTE le righe attese compaiono ESATTE.
set -u
H="$(cd "$(dirname "$0")" && pwd)"
OUT=$(python3 "$H/s148-parse.py" "$H/golden") || { echo "GOLDEN rc=9 (parser errore)"; exit 9; }
RC=0
while IFS= read -r want; do
  if ! grep -qF "$want" <<< "$OUT"; then
    echo "GOLDEN MANCA: $want"; RC=1
  fi
done <<'EOF'
r1_identita: galloc_n=100 sum_tag_n=100 OK
r2_identita: galloc_n=100 sum_tag_n=100 OK
DICHIARA scarto>1% vs s144 su s143.galloc_n: ora=100 s144=471263413
replica none.n: r1=60 r2=60 delta=0.000%
replica_worst: - 0.000%
tag none      n=          60 (60.00%) attr=          20 other=          40 (40.00%) bytes=960
tag arrgrow   n=          10 (10.00%) attr=           6 other=           4 ( 4.00%) bytes=200
other none                40 (40.00% di galloc_n)  NON-bersaglio-solo (sotto soglia INDIZIO)
other frame               20 (20.00% di galloc_n)  NON-bersaglio-solo (sotto soglia INDIZIO)
hist arrgrow   0,0,10,0,0,0,0,0,0,0,0  top: <=48:10
EOF
[ "$RC" = 0 ] && echo "GOLDEN PASS (10 righe attese, tutte esatte)"
echo "GOLDEN rc=$RC"
exit "$RC"
