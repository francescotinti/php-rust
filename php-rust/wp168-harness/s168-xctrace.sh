#!/bin/bash
# s168-xctrace.sh — SANATURE az.rev.2+4 S-167 sul braccio (c) xctrace 'CPU
# Counters' (quote top-down a 4 colonne, per-campione): (i) REPLICHE R=3 per
# lato (p-dq, o-dq) e del mutante branch oracle (o-mut ×3: spiegare c1
# 0,031→0,004); (ii) MUTANTE BACKEND arith-memstall.php (un miss DRAM/iter, su
# ENTRAMBI i motori): la colonna che schizza = Processing/backend ⇒ c0 fissata
# per ESCLUSIONE (c1=discarded e c2=delivery già fissate in S-167).
# COPIA DICHIARATA di wp167-harness/s167-f0c.sh (manifest s168-xctrace-copia.diff
# + copia-gate v2). Le .trace si cancellano dopo l'export (spazio); gli .xml
# vanno FUORI repo a fine sessione. rc autoritativo = ab-out/xctrace.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
P=~/Claude/php-rust-output/release/phpr
O=/opt/homebrew/opt/php/bin/php
DQ="$H/../wp164-harness/arith-dq.php"
MU="$H/../wp167-harness/arith-branchmut.php"
MS="$H/arith-memstall.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s168-xctrace-verdetto.out"; RC="$OUT/xctrace.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE" >&2; exit 7; }
grep -qi "s168\|s-168" /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock assente" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
PM="$(shasum -a 256 "$P" | cut -c1-8)"
[ "$PM" = 092dcff4 ] || { echo "pin!=s166 ($PM)" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
{
echo "== s168 xctrace sanature (pin $PM; template CPU Counters; repliche R=3 p-dq/o-dq/o-mut + mutante backend memstall bilaterale) =="
for tag in p-dq-1 o-dq-1 o-mut-1 p-dq-2 o-dq-2 o-mut-2 p-dq-3 o-dq-3 o-mut-3 p-ms o-ms; do
  case "$tag" in o-*) B="$O";; p-*) B="$P";; esac
  case "$tag" in *-dq-*) D="$DQ";; *-mut-*) D="$MU";; *-ms) D="$MS";; esac
  T="$OUT/xc-$tag.trace"; rm -rf "$T"
  xctrace record --template 'CPU Counters' --output "$T" --launch -- "$B" "$D" > "$OUT/xc-$tag.rec.log" 2>&1 \
    || { echo "record FALLITO ($tag)"; echo 7 > "$RC"; exit 7; }
  xctrace export --input "$T" --xpath '//trace-toc/run[@number="1"]/data/table[@schema="CounterMetricByThread"]' > "$OUT/xc-$tag.xml" 2>/dev/null \
    || { echo "export FALLITO ($tag)"; echo 7 > "$RC"; exit 7; }
  rm -rf "$T"
done
python3 - "$OUT" <<'PY'
import sys, re
out = sys.argv[1]
def shares(tag):
    rows = []
    for m in re.finditer(r'<duration[^>]*>(\d+)</duration>.*?<uint64-array[^>]*fmt="[^"]*">([\d ]+)</uint64-array>', open(f"{out}/xc-{tag}.xml").read(), re.S):
        dur = int(m.group(1)); vals = [int(x) for x in m.group(2).split()]
        if len(vals) == 4 and sum(vals) > 0: rows.append((dur, vals))
    if len(rows) < 10: return None
    rows = rows[len(rows)//10 : -max(1, len(rows)//10)]
    tot = [0.0]*4; W = 0
    for dur, vals in rows:
        s = sum(vals)
        for i in range(4): tot[i] += dur*vals[i]/s
        W += dur
    return [t/W for t in tot], len(rows)
res = {}
for tag in ("p-dq-1","p-dq-2","p-dq-3","o-dq-1","o-dq-2","o-dq-3","o-mut-1","o-mut-2","o-mut-3","p-ms","o-ms"):
    r = shares(tag)
    if r is None: print(f"{tag}: campioni insufficienti"); sys.exit(7)
    res[tag] = r[0]
    print(f"{tag}: quote c0..c3 = " + " ".join(f"{v:.3f}" for v in r[0]) + f" (campioni {r[1]})")
def spread(pref):
    rs = [res[f"{pref}-{k}"] for k in (1,2,3)]
    return [ (sum(r[i] for r in rs)/3, max(r[i] for r in rs)-min(r[i] for r in rs)) for i in range(4)]
for pref in ("p-dq","o-dq","o-mut"):
    sp = spread(pref)
    print(f"REPLICHE {pref}: media±spread c0..c3 = " + " ".join(f"{m:.3f}±{s:.3f}" for m,s in sp))
odq, omut = spread("o-dq"), spread("o-mut")
print(f"o-mut c1 vs o-dq c1: {omut[1][0]:.3f} vs {odq[1][0]:.3f} (spread {omut[1][1]:.3f}/{odq[1][1]:.3f}) -> {'ANOMALIA CONFERMATA (riproducibile)' if omut[1][0] < odq[1][0]-max(odq[1][1],omut[1][1]) else 'NON riproducibile: era rumore N=1'}")
pdq = spread("p-dq")
for side, ms, base in (("phpr", res["p-ms"], [m for m,_ in pdq]), ("oracle", res["o-ms"], [m for m,_ in odq])):
    d = [ms[i]-base[i] for i in range(4)]
    ci = max(range(4), key=lambda i: d[i])
    print(f"MUTANTE BACKEND {side}: Δquote memstall−dq c0..c3 = " + " ".join(f"{x:+.3f}" for x in d) + f" -> colonna che sale = c{ci}")
sys.exit(0)
PY
prc=$?
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
