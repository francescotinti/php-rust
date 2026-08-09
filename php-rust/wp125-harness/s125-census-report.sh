#!/bin/bash
# s125-census-report.sh — ARBITRO del controllo ±zval (criterio p.4-6,
# committato PRIMA del run). Netting vs empty, 2 decimali, R=2 identici o
# NON ACQUISITA; predizioni e tolleranze dal criterio p.5.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp125-harness"
OUT="$H/census-out"
VERD="$H/s125-census-verdetto.out"
[ "$(cat "$OUT/run.rc" 2>/dev/null)" = 0 ] || { echo "run.rc != 0 — STOP" | tee -a "$VERD"; echo 1 > "$OUT/report.rc"; exit 1; }
python3 - "$OUT" >> "$VERD" <<'PY'
import re, sys
out = sys.argv[1]
N = {"arith": 50_000_000, "prop": 30_000_000, "calls": 20_000_000,
     "str": 4_000_000, "arr": 6_000_000, "re": 2_000_000}
CATS = ["arith", "prop", "calls", "str", "arr", "re"]
def counters(side, cat, r):
    for line in open(f"{out}/{side}-{cat}-r{r}.census"):
        if line.startswith("gacensus "):
            return {k: int(v) for k, v in
                    (kv.split("=") for kv in re.findall(r"\w+=\d+", line))}
    raise SystemExit(f"gacensus assente: {side}-{cat}-r{r}")
rc = 0
vals = {}   # (side,cat) -> (zvclone/it, alloc/it)
for side in ("A", "B"):
    for cat in CATS:
        per = []
        for r in (1, 2):
            c, e = counters(side, cat, r), counters(side, "empty", r)
            per.append((round((c["zvclone_all"]-e["zvclone_all"])/N[cat], 2),
                        round((c["galloc_n"]-e["galloc_n"])/N[cat], 2)))
        if per[0] != per[1]:
            print(f"{side} {cat}: R=2 NON identici {per} — cifra NON ACQUISITA"); rc = 1
        vals[(side, cat)] = per[0]
PRED_A = {"arith": 1.00, "prop": 3.00, "calls": 2.00, "str": 2.00, "arr": 5.08, "re": 3.00}
PRED_D = {"prop": 2.00}          # B-A zvclone atteso; altrove 0.00
TOL = {"arr": 0.10}              # default 0.05
print("== s125 controllo ±zval STESSO head (fusa A vs unfused B; zvclone/it e alloc/it nettati) ==")
grado = 0
for cat in CATS:
    za, aa = vals[("A", cat)]; zb, ab = vals[("B", cat)]
    dz, da = round(zb-za, 2), round(ab-aa, 2)
    tol = TOL.get(cat, 0.05)
    ok_a  = abs(za - PRED_A[cat]) <= tol
    ok_dz = abs(dz - PRED_D.get(cat, 0.0)) <= tol
    ok_da = abs(da) <= tol
    g = "OK" if (ok_a and ok_dz and ok_da) else "MANCATO"
    if g == "MANCATO": grado = 1
    print(f"{cat:6s} A: zv={za:.2f} alloc={aa:.2f} | B: zv={zb:.2f} alloc={ab:.2f} | B-A zv={dz:+.2f} alloc={da:+.2f} | pred A={PRED_A[cat]:.2f} Dzv={PRED_D.get(cat,0.0):+.2f} tol={tol:.2f} -> {g}")
if rc == 0 and grado == 0:
    print("VERDETTO: fusione CONFERMATA come MISURA (prop double-hit = B-A zvclone +2,00; inferenza S-123 chiusa)")
elif rc == 0:
    print("VERDETTO: grado MANCATO su una o piu' voci — diagnosi PRIMA di ogni uso (criterio p.6)")
sys.exit(1 if rc else (2 if grado else 0))  # 1=NON ACQUISITA, 2=grado mancato (esito valido registrato)
PY
prc=$?
echo "$prc" > "$OUT/report.rc"
exit "$prc"
