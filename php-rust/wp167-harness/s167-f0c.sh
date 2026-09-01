#!/bin/bash
# s167-f0c.sh — FETTA 0 braccio (c): counters CPU bilaterali via xctrace
# (criterio s167-criterio-f0.md, ADATTAMENTO DICHIARATO: il template 'CPU
# Counters' di Instruments 16 espone QUOTE top-down per-campione
# (uint64-array a 4 colonne; legenda del template: Cycles/Delivery/
# Discarded/Processing-Useful) e NON branch-miss grezzi ⇒ (i) la semantica
# di colonna si fissa EMPIRICAMENTE col MUTANTE branch (la colonna che
# schizza su branchmut = scarti da mispredict); (ii) collaudo strumento:
# quella colonna deve crescere >=2x di quota su phpr-branchmut vs phpr-dq,
# altrimenti (c) INDISPONIBILE dichiarato; (iii) firma front-end: quota
# (delivery+discarded) phpr >= 2x oracle su arith-dq.
# rc autoritativo = f0-out/f0c.rc; verdetto s167-f0c-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
P=~/Claude/php-rust-output/release/phpr
O=/opt/homebrew/opt/php/bin/php
DQ="$H/../wp164-harness/arith-dq.php"
MU="$H/arith-branchmut.php"
OUT="$H/f0-out"; mkdir -p "$OUT"
VERD="$H/s167-f0c-verdetto.out"; RC="$OUT/f0c.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE" >&2; exit 7; }
grep -qi "s167\|s-167" /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock assente" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
PM="$(shasum -a 256 "$P" | cut -c1-8)"
[ "$PM" = 092dcff4 ] || { echo "pin!=s166 ($PM)" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
{
echo "== s167 F0 braccio (c) xctrace bottleneck bilaterale (pin $PM; template CPU Counters; adattamento quote DICHIARATO in testa allo script) =="
for tag in o-dq o-mut p-dq p-mut; do
  case "$tag" in o-*) B="$O";; p-*) B="$P";; esac
  case "$tag" in *-dq) D="$DQ";; *-mut) D="$MU";; esac
  T="$OUT/f0c-$tag.trace"; rm -rf "$T"
  xctrace record --template 'CPU Counters' --output "$T" --launch -- "$B" "$D" > "$OUT/f0c-$tag.rec.log" 2>&1 \
    || { echo "record FALLITO ($tag)"; echo 7 > "$RC"; exit 7; }
  xctrace export --input "$T" --xpath '//trace-toc/run[@number="1"]/data/table[@schema="CounterMetricByThread"]' > "$OUT/f0c-$tag.xml" 2>/dev/null \
    || { echo "export FALLITO ($tag)"; echo 7 > "$RC"; exit 7; }
  rm -rf "$T"   # spazio: i .trace pesano; i dati vivono negli .xml
done
python3 - "$OUT" <<'PY'
import sys, re
out = sys.argv[1]
def shares(tag):
    rows = []
    for m in re.finditer(r'<duration[^>]*>(\d+)</duration>.*?<uint64-array[^>]*fmt="[^"]*">([\d ]+)</uint64-array>', open(f"{out}/f0c-{tag}.xml").read(), re.S):
        dur = int(m.group(1)); vals = [int(x) for x in m.group(2).split()]
        if len(vals) == 4 and sum(vals) > 0: rows.append((dur, vals))
    if len(rows) < 10: return None
    rows = rows[len(rows)//10 : -max(1, len(rows)//10)]  # scarto 10% ai bordi
    tot = [0.0]*4; W = 0
    for dur, vals in rows:
        s = sum(vals)
        for i in range(4): tot[i] += dur*vals[i]/s
        W += dur
    return [t/W for t in tot], len(rows)
res = {}
for tag in ("o-dq","o-mut","p-dq","p-mut"):
    r = shares(tag)
    if r is None: print(f"{tag}: campioni insufficienti"); sys.exit(7)
    res[tag] = r[0]
    print(f"{tag}: quote colonne c0..c3 = " + " ".join(f"{v:.3f}" for v in r[0]) + f" (campioni {r[1]})")
# mutante: colonna che cresce di piu' su phpr mut vs dq
deltas = [res["p-mut"][i]-res["p-dq"][i] for i in range(4)]
ci = max(range(4), key=lambda i: deltas[i])
ratio = res["p-mut"][ci]/max(res["p-dq"][ci], 1e-9)
print(f"MUTANTE: colonna mispredict-indiziata = c{ci} (Δquota {deltas[ci]:+.3f}, quota mut/dq = {ratio:.2f}x)")
if ratio < 2.0:
    print("STRUMENTO NON MORDE (ratio<2) — (c) INDISPONIBILE dichiarato"); sys.exit(5)
# firma front-end su dq: quota c_i (mispredict) + eventuale colonna delivery
fe_p, fe_o = res["p-dq"][ci], res["o-dq"][ci]
print(f"firma front-end (colonna c{ci} su arith-dq): phpr={fe_p:.3f} oracle={fe_o:.3f} rapporto={fe_p/max(fe_o,1e-9):.2f}x (soglia criterio: >=2x)")
print("quote INTERE dq a verbale per la lettura congiunta (delivery/processing): vedi righe sopra")
sys.exit(0)
PY
prc=$?
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
