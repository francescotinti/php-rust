#!/bin/bash
# s123-classifica-report.sh — criterio s123-criterio-classifica.md p.5-6:
# tabella-v2 (fusione VIVA) vs v1 (s119, fusione SPENTA) vs oracle, N OMOGENEI
# (arr = 6M op); R=2 phpr deve essere IDENTICO o la voce è NON ACQUISITA.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp123-harness"
OUT="$H/classifica-out"
OLD="$REPO/wp119-harness/clite-out"
VERD="$H/s123-classifica-verdetto.out"

[ "$(cat "$OUT/run.rc" 2>/dev/null)" = 0 ] || { echo "run.rc != 0 — STOP" | tee -a "$VERD"; echo 1 > "$OUT/report.rc"; exit 1; }

python3 - "$OUT" "$OLD" > "$OUT/report.txt" <<'PY'
import sys, re, os
out, old = sys.argv[1], sys.argv[2]
N = {"arith": 50_000_000, "prop": 30_000_000, "calls": 20_000_000,
     "str": 4_000_000, "arr": 6_000_000, "re": 2_000_000}
CATS = ["arith", "prop", "calls", "str", "arr", "re"]

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

rows = []
non_acq = []
for c in CATS:
    v2r = [read_ga(os.path.join(out, f"phpr-{c}-r{r}.census")) for r in (1, 2)]
    e2r = [read_ga(os.path.join(out, f"phpr-empty-r{r}.census")) for r in (1, 2)]
    if v2r[0] != v2r[1] or e2r[0] != e2r[1]:
        non_acq.append(c)
    v2 = net(v2r[0], e2r[0], N[c])           # (alloc, free, realloc, zvall, zvrc)/iter
    v1 = net(read_ga(os.path.join(old, f"phpr-{c}-r1.census")),
             read_ga(os.path.join(old, "phpr-empty-r1.census")), N[c])
    za, zf = read_zend(os.path.join(out, f"zend-{c}-r1.count"))
    ea, ef = read_zend(os.path.join(out, "zend-empty-r1.count"))
    o_alloc = round((za - ea) / N[c], 2)
    rows.append((c, v2, v1, o_alloc))

print(f"{'cat':6} {'alloc/it v2':>11} {'v1(unfused)':>11} {'oracle':>7} {'Δv2-or':>7} {'zvclone v2':>10} {'v1':>6} {'zvrc v2':>8} {'v1':>6}")
for c, v2, v1, o in sorted(rows, key=lambda r: -(r[1][0] - r[3])):
    print(f"{c:6} {v2[0]:11.2f} {v1[0]:11.2f} {o:7.2f} {v2[0]-o:+7.2f} {v2[3]:10.2f} {v1[3]:6.2f} {v2[4]:8.2f} {v1[4]:6.2f}")
for c, v2, v1, o in rows:
    dirn = "CROLLA" if v2[3] < v1[3] - 0.5 else ("=" if abs(v2[3]-v1[3]) <= 0.5 else "SALE")
    print(f"{c}: v1->v2 zvclone {v1[3]:.2f}->{v2[3]:.2f} [{dirn}] alloc {v1[0]:.2f}->{v2[0]:.2f}")
if non_acq:
    print(f"NON ACQUISITE (R=2 non identico, criterio p.4): {non_acq}")
print(f"RCV={1 if non_acq else 0}")
PY
sed '/^RCV=/d' "$OUT/report.txt"
RC=$(awk -F= '/^RCV=/{print $2}' "$OUT/report.txt")
echo "$RC" > "$OUT/report.rc"
{
  echo ""
  echo "CLASSIFICA-V2 (fusione VIVA, rc=$RC da classifica-out/report.rc):"
  sed '/^RCV=/d' "$OUT/report.txt"
} >> "$VERD"
exit "$RC"
