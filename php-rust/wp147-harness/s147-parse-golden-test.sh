#!/bin/bash
# s147-parse-golden-test.sh — golden del parser s147-parse.py (criterio p.5:
# parser committato + golden PRIMA del run). Fixture in golden/ con numeri
# calcolati A MANO: soglia 0,007x(40,10-0,10)=0,280 s; coerenza 15==15;
# ponte 15=100%; prezzi (mv-cal): scalar 3,0 str 4,0; kill ZERO codice.
# rc=0 solo se TUTTE le righe attese compaiono ESATTE.
set -u
H="$(cd "$(dirname "$0")" && pwd)"
OUT=$(python3 "$H/s147-parse.py" "$H/golden" "$H/golden/prezzi.out" "$H/golden/rimisura.out") || { echo "GOLDEN rc=9 (parser errore)"; exit 9; }
RC=0
while IFS= read -r want; do
  if ! grep -qF "$want" <<< "$OUT"; then
    echo "GOLDEN MANCA: $want"; RC=1
  fi
done <<'EOF'
soglia KS-146-1 = 0,7% x orm_net 40.00 s = 0.280 s
r1_coerenza: s147mv_tot=15 s145_clone_tot=15 OK
r2_coerenza: s147mv_tot=15 s145_clone_tot=15 OK
replica tot_mv: r1=15 r2=15 delta=0.000%
ponte_mv (s147mv, lista ['LoadSlot', 'LoadVar']): 15 = 100.00% del tot_mv; sec=0.000
slot_reads (convenzione census): 15  slot_reads_rc=5
dg Sweep->LoadSlot: 15
take_str: n=2 pieno=0.000s incdec=0.000s
take_safe_tot: n=4 (rc=2)
banda_borrow_first=0.000s soglia=0.280s rapporto=0.00 => ZERO codice sul bersaglio (banda < 1x soglia)
EOF
[ "$RC" = 0 ] && echo "GOLDEN PASS (10 righe attese, tutte esatte)"
echo "GOLDEN rc=$RC"
exit "$RC"
