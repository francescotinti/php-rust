#!/bin/bash
# s169-xctrace.sh — XCTRACE-2 (criterio s169-criterio.md p.3): mutante branch NON predicibile (rbm) + mutante c0 (crc32 L1) bilaterali. Derivato da s168-xctrace.sh. Testo originale S-168:
# s169-xctrace.sh — SANATURE az.rev.2+4 S-167 sul braccio (c) xctrace 'CPU
# Counters' (quote top-down a 4 colonne, per-campione): (i) REPLICHE R=3 per
# lato (p-dq, o-dq) e del mutante branch oracle (o-mut ×3: spiegare c1
# 0,031→0,004); (ii) MUTANTE BACKEND arith-memstall.php (un miss DRAM/iter, su
# ENTRAMBI i motori): la colonna che schizza = Processing/backend ⇒ c0 fissata
# per ESCLUSIONE (c1=discarded e c2=delivery già fissate in S-167).
# COPIA DICHIARATA di wp167-harness/s167-f0c.sh (manifest s169-xctrace-copia.diff
# + copia-gate v2). Le .trace si cancellano dopo l'export (spazio); gli .xml
# vanno FUORI repo a fine sessione. rc autoritativo = ab-out/xctrace.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
P=~/Claude/php-rust-output/release/phpr
O=/opt/homebrew/opt/php/bin/php
DQ="$H/../wp164-harness/arith-dq.php"
RB="$H/../wp168-harness/arith-rbranchmut.php"
C0="$H/arith-c0mut.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s169-xctrace-verdetto.out"; RC="$OUT/xctrace.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE" >&2; exit 7; }
grep -qi "s169\|s-168" /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock assente" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
PM="$(shasum -a 256 "$P" | cut -c1-8)"
[ "$PM" = 092dcff4 ] || { echo "pin!=s166 ($PM)" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
{
echo "== s169 xctrace sanature (pin $PM; template CPU Counters; repliche R=3 p-dq/o-dq/o-mut + mutante backend memstall bilaterale) =="
for tag in p-dq o-dq p-rbm o-rbm p-c0 o-c0; do
  case "$tag" in o-*) B="$O";; p-*) B="$P";; esac
  case "$tag" in *-dq) D="$DQ";; *-rbm) D="$RB";; *-c0) D="$C0";; esac
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
for tag in ("p-dq","o-dq","p-rbm","o-rbm","p-c0","o-c0"):
    r = shares(tag)
    if r is None: print(f"{tag}: campioni insufficienti"); sys.exit(7)
    res[tag] = r[0]
    print(f"{tag}: quote c0..c3 = " + " ".join(f"{v:.3f}" for v in r[0]) + f" (campioni {r[1]})")
for side in ("p","o"):
    for mut, att in (("rbm","c3 (discarded) se il mutante branch casuale morde"),("c0","c0 (useful) se c0 è lavoro retiring")):
        d = [res[f"{side}-{mut}"][i]-res[f"{side}-dq"][i] for i in range(4)]
        ci = max(range(4), key=lambda i: d[i])
        print(f"MUTANTE {mut} {'phpr' if side=='p' else 'oracle'}: Δquote vs dq c0..c3 = " + " ".join(f"{x:+.3f}" for x in d) + f" -> colonna che sale = c{ci} (attesa: {att})")
sys.exit(0)
PY
prc=$?
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
