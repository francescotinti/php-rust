#!/bin/bash
# s124-classifica-report.sh — ADMISSION del criterio s124-criterio-phpstr.md p.3:
# confronto DATATO stesso-albero-±-patch: v124 (head 43d7610, census S-124) vs
# v123 (head 65b3385, raw wp123-harness/classifica-out — unico delta di sorgente
# tra i due head = la patch single-alloc, verificato con git diff --stat).
# PREDIZIONI (criterio p.3): Δalloc/iter str −2,00 · re −3,00 · arr −2,04,
# resto 0. «In grado» = |scarto| ≤ 0,25 (0,30 per arr: inferenza aritmetica).
# R=2 phpr deve essere IDENTICO o la voce è NON ACQUISITA.
# rc: 0=ADMISSION-PASSA (A/B autorizzato) 1=predizione mancata o R=2 instabile — STOP.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp124-harness"
OUT="$H/classifica-out"
OLD="$REPO/wp123-harness/classifica-out"
VERD="$H/s124-classifica-verdetto.out"

[ "$(cat "$OUT/run.rc" 2>/dev/null)" = 0 ] || { echo "run.rc != 0 — STOP" | tee -a "$VERD"; echo 1 > "$OUT/report.rc"; exit 1; }

python3 - "$OUT" "$OLD" > "$OUT/report.txt" <<'PY'
import sys, re, os
out, old = sys.argv[1], sys.argv[2]
N = {"arith": 50_000_000, "prop": 30_000_000, "calls": 20_000_000,
     "str": 4_000_000, "arr": 6_000_000, "re": 2_000_000}
CATS = ["arith", "prop", "calls", "str", "arr", "re"]
PRED = {"str": -2.00, "re": -3.00, "arr": -2.04,
        "arith": 0.00, "prop": 0.00, "calls": 0.00}
TOL = {"arr": 0.30}

def read_ga(path):
    txt = open(path, errors="replace").read().replace("\0", "")
    m = re.search(r"gacensus .*galloc_n=(\d+) gfree_n=(\d+) realloc_n=(\d+) zvclone_all=(\d+) zvclone_rc=(\d+)", txt)
    if not m:
        raise SystemExit(f"gacensus assente in {path}")
    return tuple(int(x) for x in m.groups())

def read_zend(path):
    txt = open(path, errors="replace").read().replace("\0", "")
    m = re.search(r"countinterpose .*malloc_n=(\d+) calloc_n=(\d+) realloc_n=(\d+) free_n=(\d+)", txt)
    if not m:
        raise SystemExit(f"countinterpose assente in {path}")
    ma, ca, ra, fr = (int(x) for x in m.groups())
    return ma + ca, fr

def net(vals_cat, vals_empty, n):
    return tuple(round((c - e) / n, 2) for c, e in zip(vals_cat, vals_empty))

rows, non_acq, missed = [], [], []
for c in CATS:
    vr = [read_ga(os.path.join(out, f"phpr-{c}-r{r}.census")) for r in (1, 2)]
    er = [read_ga(os.path.join(out, f"phpr-empty-r{r}.census")) for r in (1, 2)]
    if vr[0] != vr[1] or er[0] != er[1]:
        non_acq.append(c)
    v124 = net(vr[0], er[0], N[c])
    v123 = net(read_ga(os.path.join(old, f"phpr-{c}-r1.census")),
               read_ga(os.path.join(old, "phpr-empty-r1.census")), N[c])
    za, _ = read_zend(os.path.join(out, f"zend-{c}-r1.count"))
    ea, _ = read_zend(os.path.join(out, "zend-empty-r1.count"))
    o_alloc = round((za - ea) / N[c], 2)
    d = round(v124[0] - v123[0], 2)
    tol = TOL.get(c, 0.25)
    ok = abs(d - PRED[c]) <= tol
    if not ok:
        missed.append(f"{c}(Δ{d:+.2f} vs pred {PRED[c]:+.2f})")
    rows.append((c, v124, v123, o_alloc, d, ok))

print(f"{'cat':6} {'alloc/it v124':>13} {'v123':>7} {'Δpatch':>7} {'pred':>6} {'esito':>6} {'oracle':>7} {'Δv124-or':>8} {'realloc v124':>12}")
for c, v124, v123, o, d, ok in rows:
    print(f"{c:6} {v124[0]:13.2f} {v123[0]:7.2f} {d:+7.2f} {PRED[c]:+6.2f} {'OK' if ok else 'MANCA':>6} {o:7.2f} {v124[0]-o:+8.2f} {v124[2]:12.2f}")
if non_acq:
    print(f"NON ACQUISITE (R=2 non identico): {non_acq}")
if missed:
    print(f"PREDIZIONI MANCATE: {missed}")
rc = 1 if (non_acq or missed) else 0
print("ADMISSION " + ("PASSA — A/B autorizzato" if rc == 0 else "FALLITA — STOP e ridiagnosi (criterio p.3)"))
print(f"RCV={rc}")
PY
prc=$?
[ "$prc" = 0 ] || { echo "report python rc=$prc — STOP" | tee -a "$VERD"; echo 1 > "$OUT/report.rc"; exit 1; }
sed '/^RCV=/d' "$OUT/report.txt"
RC=$(awk -F= '/^RCV=/{print $2}' "$OUT/report.txt")
echo "$RC" > "$OUT/report.rc"
{
  echo ""
  echo "ADMISSION S-124 (rc=$RC da classifica-out/report.rc; v123 datata head 65b3385, v124 head CON patch):"
  sed '/^RCV=/d' "$OUT/report.txt"
} >> "$VERD"
exit "$RC"
