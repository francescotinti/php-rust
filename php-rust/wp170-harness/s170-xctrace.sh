#!/bin/bash
# s170-xctrace.sh — XCTRACE-3 (criterio s170-criterio.md p.6): mutante c0 POSITIVO (arith-c0pos.php: array_sum L1, retiring puro) bilaterale + baseline p/o-dq fresche; c3 ANCHE in ns/iter (az.rev. S-169 #4). Derivato da s169-xctrace.sh. Testo originale S-169:
# s169-xctrace.sh — XCTRACE-2 (criterio s169-criterio.md p.3): mutante branch NON predicibile (rbm) + mutante c0 (crc32 L1) bilaterali. Derivato da s168-xctrace.sh. Testo originale S-168:
# s169-xctrace.sh — SANATURE az.rev.2+4 S-167 sul braccio (c) xctrace 'CPU
# Counters' (quote top-down a 4 colonne, per-campione): (i) REPLICHE R=3 per
# lato (p-dq, o-dq) e del mutante branch oracle (o-mut ×3: spiegare c1
# 0,031→0,004); (ii) MUTANTE BACKEND arith-memstall.php (un miss DRAM/iter, su
# ENTRAMBI i motori): la colonna che schizza = Processing/backend ⇒ c0 fissata
# per ESCLUSIONE (c1=discarded e c2=delivery già fissate in S-167).
# COPIA DICHIARATA di wp169-harness/s169-xctrace.sh (manifest s170-xctrace-copia-v3.diff
# + copia-gate v3 PER TOKEN). Le .trace si cancellano dopo l'export (spazio); gli .xml
# vanno FUORI repo a fine sessione. rc autoritativo = ab-out/xctrace.rc.
# S-170: lock per TOKEN, `[ -s ]` su driver/xml, GUARDIA DISCO Data ≥10G prima di
# ogni record (pre-flight lesson S-169), purge ktrace dopo ogni export.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
P=~/Claude/php-rust-output/release/phpr
O=/opt/homebrew/opt/php/bin/php
DQ="$H/../wp164-harness/arith-dq.php"
C0="$H/arith-c0pos.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s170-xctrace-verdetto.out"; RC="$OUT/xctrace.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE" >&2; exit 7; }
for f in "$DQ" "$C0" "$P" "$O"; do [ -s "$f" ] || { echo "file assente o VUOTO: $f" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }; done
grep -qw s170 /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock s170 assente (per TOKEN)" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
PM="$(shasum -a 256 "$P" | cut -c1-8)"
[ "$PM" = 092dcff4 ] || { echo "pin!=s166 ($PM)" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
{
echo "== s170 xctrace-3 (pin $PM; template CPU Counters; p/o-dq + mutante c0 POSITIVO array_sum L1 bilaterale; c3 anche in ns/iter) =="
# parità del mutante contro l'ATTESO oracle (az.rev. S-169 #3)
"$O" "$C0" > "$OUT/expected-arith-c0pos.out" 2>&1; "$P" "$C0" > "$OUT/xc-p-c0.parity.out" 2>&1
[ -s "$OUT/expected-arith-c0pos.out" ] && cmp -s "$OUT/expected-arith-c0pos.out" "$OUT/xc-p-c0.parity.out" || { echo "c0pos: output phpr ≠ atteso oracle — STOP"; echo 2 > "$RC"; exit 2; }
echo "parità c0pos: phpr==oracle ($(tr '\n' ' ' < "$OUT/expected-arith-c0pos.out"))"
for tag in p-dq o-dq p-c0 o-c0; do
  case "$tag" in o-*) B="$O";; p-*) B="$P";; esac
  case "$tag" in *-dq) D="$DQ";; *-c0) D="$C0";; esac
  FREE=$(df -g /System/Volumes/Data | awk 'NR>1{print $4}')
  [ "$FREE" -ge 10 ] || { echo "GUARDIA DISCO: Data ${FREE}G <10G prima di $tag — STOP"; echo 7 > "$RC"; exit 7; }
  T="$OUT/xc-$tag.trace"; rm -rf "$T"
  xctrace record --template 'CPU Counters' --output "$T" --launch -- "$B" "$D" > "$OUT/xc-$tag.rec.log" 2>&1 \
    || { echo "record FALLITO ($tag)"; echo 7 > "$RC"; exit 7; }
  xctrace export --input "$T" --xpath '//trace-toc/run[@number="1"]/data/table[@schema="CounterMetricByThread"]' > "$OUT/xc-$tag.xml" 2>/dev/null \
    || { echo "export FALLITO ($tag)"; echo 7 > "$RC"; exit 7; }
  [ -s "$OUT/xc-$tag.xml" ] || { echo "export VUOTO ($tag)"; echo 7 > "$RC"; exit 7; }
  rm -rf "$T"
  # EMENDA S-169 (ENOSPC #2): xctrace lascia `instruments*.ktrace` (0,4-4 GB l'uno)
  # in $DARWIN_USER_TEMP_DIR — 22 file = 10 GB dopo S-167/168/169: si purgano qui.
  rm -rf "$(getconf DARWIN_USER_TEMP_DIR)"/instruments*.ktrace
done
# ns/iter dei driver dq (1 run per lato, N=250M, floor non sottratto: lettura di riferimento per c3 in ns/iter)
NSP=$(ucpu "$P" "$DQ"); NSO=$(ucpu "$O" "$DQ")
echo "dq user s (1 run, per c3 in ns/iter): phpr=$NSP oracle=$NSO"
python3 - "$OUT" "$NSP" "$NSO" <<'PY'
import sys, re
out = sys.argv[1]; nsp = float(sys.argv[2])/250e6*1e9; nso = float(sys.argv[3])/250e6*1e9
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
for tag in ("p-dq","o-dq","p-c0","o-c0"):
    r = shares(tag)
    if r is None: print(f"{tag}: campioni insufficienti"); sys.exit(7)
    res[tag] = r[0]
    print(f"{tag}: quote c0..c3 = " + " ".join(f"{v:.3f}" for v in r[0]) + f" (campioni {r[1]})")
print(f"c3 (discarded) in ns/iter su dq: phpr {res['p-dq'][3]*nsp:.2f} (quota {res['p-dq'][3]:.3f} × {nsp:.1f}) vs oracle {res['o-dq'][3]*nso:.2f} (quota {res['o-dq'][3]:.3f} × {nso:.1f}); c2 (delivery): phpr {res['p-dq'][2]*nsp:.2f} vs oracle {res['o-dq'][2]*nso:.2f} ns/iter — c2 resta INDIZIATA (semantica non fissata)")
ok = True
for side in ("p","o"):
    d = [res[f"{side}-c0"][i]-res[f"{side}-dq"][i] for i in range(4)]
    ci = max(range(4), key=lambda i: d[i])
    verdict = "POSITIVO: c0 = useful/retiring FISSATA" if ci == 0 else f"FALLITO (sale c{ci}): c0 resta per esclusione"
    if ci != 0: ok = False
    print(f"MUTANTE c0pos {'phpr' if side=='p' else 'oracle'}: Δquote vs dq c0..c3 = " + " ".join(f"{x:+.3f}" for x in d) + f" -> colonna che sale = c{ci} (attesa c0) -> {verdict}")
print("ESITO: c0 fissata da mutante POSITIVO su ENTRAMBI i motori" if ok else "ESITO: mutante c0 positivo NON riuscito su almeno un motore — dichiarare")
sys.exit(0 if ok else 4)
PY
prc=$?
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
