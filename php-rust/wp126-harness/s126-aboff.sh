#!/bin/bash
# s126-aboff.sh — A/B full WP off-patch s123↔s124 stessa notte
# (criterio s126-criterio-aboff.md, committato PRIMA del run).
# 4 gambe intercalate A1 B1 A2 B2; ricetta pair109 phpr-only; gate contesa ictx.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
OUT="$H/aboff-out"; mkdir -p "$OUT"
VERD="$H/s126-aboff-verdetto.out"
WPDEV="/Users/francescotinti/Claude/wpdev"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
S="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
A="$S/phpr-s123-p0b"; HA_ATT="885d2c646ac7ff4c"
B="$S/phpr-s124";     HB_ATT="c5ba2573a23adf69"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/aboff.done"
[ "$(shasum -a 256 "$A" | cut -c1-16)" = "$HA_ATT" ] || { echo "rc=9 A!=s123-p0b" > "$OUT/aboff.done"; exit 9; }
[ "$(shasum -a 256 "$B" | cut -c1-16)" = "$HB_ATT" ] || { echo "rc=9 B!=s124" > "$OUT/aboff.done"; exit 9; }
{ echo "A=s123-p0b $HA_ATT  B=s124 $HB_ATT"
  echo "server=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/php-server" | cut -c1-16) (costante)"
  echo "epoch=$(date +%s)"; } > "$OUT/identity.txt"

reset_env(){
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || exit 3
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
}
names(){ tr -d '\0' < "$1" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u; }

"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=4 guard" > "$OUT/aboff.done"; exit 4; }
cd "$WPDEV" || exit 2
for leg in 1 2; do
  for side in A B; do
    E="$A"; [ "$side" = B ] && E="$B"
    L="$side$leg"
    p "media $L START"; reset_env
    MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER=1 /usr/bin/time -l "$E" vendor/bin/phpunit --group media \
      > "$OUT/media-$L.txt" 2> "$OUT/media-$L.time"
    p "media $L rc=$?"
    p "full $L START"; reset_env
    MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER=1 /usr/bin/time -l "$E" vendor/bin/phpunit \
      > "$OUT/full-$L.txt" 2> "$OUT/full-$L.time"
    p "full $L rc=$?"
    names "$OUT/full-$L.txt" > "$OUT/full-$L.failnames"
  done
done
find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$OUT/progress.txt" 2>&1

python3 - "$OUT" > "$VERD" 2>&1 <<'PY'
import re, sys, statistics
out = sys.argv[1]
def t(f):
    d = {}
    for l in open(f):
        m = re.search(r'([\d.]+)\s+user', l)
        if m: d['user'] = float(m.group(1))
        m = re.search(r'([\d.]+)\s+sys', l)
        if m: d['sys'] = float(m.group(1))
        m = re.search(r'^\s*(\d+)\s+peak memory footprint', l)
        if m: d['pf'] = int(m.group(1))
        m = re.search(r'^\s*(\d+)\s+involuntary context switches', l)
        if m: d['ictx'] = int(m.group(1))
    d['cpu'] = d.get('user', 0) + d.get('sys', 0)
    return d
legs = {L: t(f"{out}/full-{L}.time") for L in ("A1", "B1", "A2", "B2")}
med = {L: t(f"{out}/media-{L}.time") for L in ("A1", "B1", "A2", "B2")}
print("== s126 aboff: A=s123-p0b (pre-PhpStr) vs B=s124 (post) — full WP stessa notte ==")
print("grade=VERDICT  # derivazione meccanica dai .time; criterio s126-criterio-aboff.md")
for L in ("A1", "B1", "A2", "B2"):
    d = legs[L]
    print(f"full_{L}: cpu={d['cpu']:.2f} (user={d.get('user',0):.2f} sys={d.get('sys',0):.2f}) ictx={d.get('ictx',-1)} peak_MiB={d.get('pf',0)/1048576:.1f} media_cpu={med[L]['cpu']:.2f}")
ictx = {L: legs[L].get('ictx', -1) for L in legs}
m = statistics.median(ictx.values())
null_legs = [L for L, v in ictx.items() if v > 1.5 * m]
print(f"ictx_mediana={m:.0f} gambe_nulle(>1,5x)={null_legs or 'nessuna'}")
va = [legs[L]['cpu'] for L in ("A1", "A2") if L not in null_legs]
vb = [legs[L]['cpu'] for L in ("B1", "B2") if L not in null_legs]
sa = sorted(open(f"{out}/full-A1.failnames").read()) == sorted(open(f"{out}/full-A2.failnames").read())
sb = sorted(open(f"{out}/full-B1.failnames").read()) == sorted(open(f"{out}/full-B2.failnames").read())
print(f"failset_stabile: A={sa} B={sb}")
if not va or not vb:
    print("VERDETTO NULLO: un lato senza gambe valide"); sys.exit(0)
if not (sa and sb):
    print("VERDETTO NULLO: fail-set instabile su un lato"); sys.exit(0)
ma, mb = statistics.median(va), statistics.median(vb)
spread_a = (max(va) - min(va)) / ma if len(va) > 1 else 0.0
spread_b = (max(vb) - min(vb)) / mb if len(vb) > 1 else 0.0
delta = (mb - ma) / ma
print(f"A_med={ma:.2f} (spread {100*spread_a:.2f}%)  B_med={mb:.2f} (spread {100*spread_b:.2f}%)")
print(f"delta_B_vs_A={100*delta:+.2f}%  soglia_risoluzione=max spread intra-lato {100*max(spread_a,spread_b):.2f}%")
print("RISOLTO" if abs(delta) > max(spread_a, spread_b) and len(va) > 1 and len(vb) > 1 else "NON RISOLVIBILE anche same-night — voce chiusa (criterio p.5)")
PY
echo "rc=0 $(date +%T)" > "$OUT/aboff.done"
p "DONE"
