#!/bin/bash
# s171-xctrace.sh <BPATH> <BEXP8> — az.rev. S-170 #2: xctrace 'CPU Counters' (quote top-down c0..c3) su arith-dq per TRE bracci:
# p-dq = pin s166 (stesso sorgente di m0 36d73812, altro path di build: dichiarato), b-dq = candidato L-SL1, m-dq = tetto m13
# (stash phpr-s170-m13). La colonna che CALA da p a m13 (e da p a B) fissa la semantica di c2 senza circolarità: se cala c2 il
# −22 del mock è FRONT-END (delivery: corpi grossi = icache/BTB) e la leva deve tenere il ramo lento fuori linea; se cala c0 è
# lavoro retiring. Nessun mutante c0 positivo qui (quesito separato). COPIA DICHIARATA di wp170-harness/s170-xctrace.sh
# (manifest s171-xctrace-copia-v3.diff + copia-gate v3 PER TOKEN) coi SOLI adattamenti: bracci p/b/m su dq (niente c0pos,
# niente oracle), lock per TOKEN s171, N dal driver. Le .trace si cancellano dopo l'export; purge ktrace dopo ogni export;
# GUARDIA DISCO Data ≥10G prima di ogni record. rc autoritativo = ab-out/xctrace.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
P=~/Claude/php-rust-output/release/phpr
BB="${1:?BPATH}"; BEXP="${2:?BEXP8}"
M="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s170-m13"
DQ="$H/../wp164-harness/arith-dq.php"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s171-xctrace-verdetto.out"; RC="$OUT/xctrace.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE" >&2; exit 7; }
for f in "$DQ" "$P" "$BB" "$M"; do [ -s "$f" ] || { echo "file assente o VUOTO: $f" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }; done
grep -qw s171 /private/tmp/phpr-measure.lock 2>/dev/null || { echo "lock s171 assente (per TOKEN)" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
PM="$(shasum -a 256 "$P" | cut -c1-8)"; BM="$(shasum -a 256 "$BB" | cut -c1-8)"; MM="$(shasum -a 256 "$M" | cut -c1-8)"
[ "$PM" = 092dcff4 ] || { echo "pin!=s166 ($PM)" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$BM" = "$BEXP" ] || { echo "B misurato $BM != atteso $BEXP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$MM" = 85adbfc0 ] || { echo "m13!=85adbfc0 ($MM)" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
NDQ=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$DQ")
[ -n "$NDQ" ] || { echo "N non leggibile dal driver" | tee -a "$VERD"; echo 7 > "$RC"; exit 7; }
ucpu(){ { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$@" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
{
echo "== s171 xctrace az.rev.#2 (p=pin $PM, b=candidato $BM, m=m13 $MM; template CPU Counters; arith-dq N=$NDQ) =="
for tag in p-dq b-dq m-dq; do
  case "$tag" in p-*) B="$P";; b-*) B="$BB";; m-*) B="$M";; esac
  FREE=$(df -g /System/Volumes/Data | awk 'NR>1{print $4}')
  [ "$FREE" -ge 10 ] || { echo "GUARDIA DISCO: Data ${FREE}G <10G prima di $tag — STOP"; echo 7 > "$RC"; exit 7; }
  T="$OUT/xc-$tag.trace"; rm -rf "$T"
  xctrace record --template 'CPU Counters' --output "$T" --launch -- "$B" "$DQ" > "$OUT/xc-$tag.rec.log" 2>&1 \
    || { echo "record FALLITO ($tag)"; echo 7 > "$RC"; exit 7; }
  xctrace export --input "$T" --xpath '//trace-toc/run[@number="1"]/data/table[@schema="CounterMetricByThread"]' > "$OUT/xc-$tag.xml" 2>/dev/null \
    || { echo "export FALLITO ($tag)"; echo 7 > "$RC"; exit 7; }
  [ -s "$OUT/xc-$tag.xml" ] || { echo "export VUOTO ($tag)"; echo 7 > "$RC"; exit 7; }
  rm -rf "$T"
  rm -rf "$(getconf DARWIN_USER_TEMP_DIR)"/instruments*.ktrace
done
NSP=$(ucpu "$P" "$DQ"); NSB=$(ucpu "$BB" "$DQ"); NSM=$(ucpu "$M" "$DQ")
echo "dq user s (1 run, per quote in ns/iter; floor non sottratto): pin=$NSP B=$NSB m13=$NSM"
python3 - "$OUT" "$NSP" "$NSB" "$NSM" "$NDQ" <<'PY'
import sys, re
out = sys.argv[1]; n = float(sys.argv[5])
ns = {"p-dq": float(sys.argv[2])/n*1e9, "b-dq": float(sys.argv[3])/n*1e9, "m-dq": float(sys.argv[4])/n*1e9}
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
for tag in ("p-dq","b-dq","m-dq"):
    r = shares(tag)
    if r is None: print(f"{tag}: campioni insufficienti"); sys.exit(7)
    res[tag] = r[0]
    print(f"{tag}: quote c0..c3 = " + " ".join(f"{v:.3f}" for v in r[0]) + f" (campioni {r[1]}); in ns/iter = " + " ".join(f"{v*ns[tag]:.2f}" for v in r[0]) + f" (× {ns[tag]:.1f})")
rc = 0
for tag, lab in (("m-dq","m13"),("b-dq","B")):
    d = [res[tag][i]*ns[tag]-res["p-dq"][i]*ns["p-dq"] for i in range(4)]
    ci = min(range(4), key=lambda i: d[i])
    print(f"Δ ns/iter {lab}−pin per colonna c0..c3 = " + " ".join(f"{x:+.2f}" for x in d) + f" -> colonna che CALA di più = c{ci} ({'FRONT-END/delivery: c2 = fetch/decode/resteer' if ci == 2 else 'retiring: c0 = lavoro utile' if ci == 0 else 'backend' if ci == 1 else 'discarded'})")
c2m = res["m-dq"][2]*ns["m-dq"]-res["p-dq"][2]*ns["p-dq"]; c0m = res["m-dq"][0]*ns["m-dq"]-res["p-dq"][0]*ns["p-dq"]
print(f"LETTURA: su m13 cala c2 di {c2m:+.2f} e c0 di {c0m:+.2f} ns/iter -> {'il taglio del mock è prevalentemente FRONT-END (c2 = delivery fissata dal contrasto)' if abs(c2m) > abs(c0m) else 'il taglio del mock è prevalentemente RETIRING (c0): c2 NON fissata da questo contrasto'}")
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
