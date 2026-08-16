#!/bin/bash
# s149-parse-golden-test.sh — golden del parser s149-parse.py (criterio p.6:
# parser committato + golden PRIMA del run). Fixture in golden/ con numeri
# calcolati A MANO: identita' per pid 70+30==100 (r1, 2 pid) e 100==100 (r2);
# str_repeat n=(40+30+70)/2=70 attr=30 other=40 su hostcall.other medio 60
# (66,67%); sprintf other=17 (28,33%); implode other=3 (5,00%); crosscheck
# s148tag == s149sum; DICHIARA vs riferimento s148 (fixture minuscola);
# repliche identiche => worst 0,000%. rc=0 solo se TUTTE le righe ESATTE.
set -u
H="$(cd "$(dirname "$0")" && pwd)"
OUT=$(python3 "$H/s149-parse.py" "$H/golden") || { echo "GOLDEN rc=9 (parser errore)"; exit 9; }
RC=0
while IFS= read -r want; do
  if ! grep -qF "$want" <<< "$OUT"; then
    echo "GOLDEN MANCA: $want"; RC=1
  fi
done <<'EOF'
r1_identita: hostcall_n=100 sum_name_n=100 unnamed_n=0 overflow=0 OK
r2_identita: hostcall_n=100 sum_name_n=100 unnamed_n=0 overflow=0 OK
r1_crosscheck_s148tag: hostcall.n=100 OK
r2_crosscheck_s148tag: hostcall.n=100 OK
r1_hostcall_n: ora=100 s148=325416908 delta=100.000% DICHIARA scarto>1% vs s148
replica_worst_head: - 0.000%
nomi totali=3, stampati top 30 per other
name str_repeat                   n=         70 attr=         30 other=         40 (66.67% di hostcall.other) b=2100
name sprintf                      n=         25 attr=          8 other=         17 (28.33% di hostcall.other) b=600
other str_repeat                            40 (40.00% di hostcall.n)  sotto soglia (solo per FAMIGLIA)
other implode                                3 ( 3.00% di hostcall.n)  sotto soglia (solo per FAMIGLIA)
EOF
[ "$RC" = 0 ] && echo "GOLDEN PASS (11 righe attese, tutte esatte)"
echo "GOLDEN rc=$RC"
exit "$RC"
