#!/bin/bash
# s126-compoff-rerun.sh — rimisura compoff con composer ESTRATTO (criterio
# mappa2 p.7, committato PRIMA del run). ATTENDE aboff.done (run sequenziali),
# poi: rebuild tarball con composer-x + smoke bilaterale + 2 gambe per lato.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
SP="${COMP_SP:?COMP_SP (workdir) richiesto}"
OUT="$H/mappa2-out"; mkdir -p "$OUT"
VERD="$H/s126-compoff-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/compoff2-progress.txt"; }
: > "$OUT/compoff2-progress.txt"
rm -f "$OUT/compoff2.done"
[ "$(shasum -a 256 "$PHPR" | cut -c1-16)" = "002e6cc12047ab9f" ] || { echo "rc=9 pin!=s125" > "$OUT/compoff2.done"; exit 9; }

p "attesa aboff.done"
i=0; while [ ! -f "$H/aboff-out/aboff.done" ] && [ $i -lt 1080 ]; do /bin/sleep 20; i=$((i+1)); done
[ -f "$H/aboff-out/aboff.done" ] || { echo "rc=9 timeout-attesa-aboff" > "$OUT/compoff2.done"; exit 9; }
p "aboff concluso: $(cat "$H/aboff-out/aboff.done")"

# rebuild: composer-x dentro il workspace, poi CONGELA il tarball prima delle gambe
rm -rf "$SP/compoff-work"; tar xzf "$GATES/compoff-work.tgz" -C "$SP" || { echo "rc=8 untar" > "$OUT/compoff2.done"; exit 8; }
( cd "$SP/compoff-work" && rm -rf composer-x && \
  PHAR_SRC="$GATES/composer.phar" "$ORACLE" -r '$p = new Phar(getenv("PHAR_SRC")); $p->extractTo("composer-x", null, true);' ) \
  || { echo "rc=8 extract" > "$OUT/compoff2.done"; exit 8; }
# smoke bilaterale rc=0 ESATTO (forge-silent-failure)
( cd "$SP/compoff-work" && "$ORACLE" composer-x/bin/composer --version > /dev/null 2>&1 ) || { echo "rc=8 smoke-oracle" > "$OUT/compoff2.done"; exit 8; }
( cd "$SP/compoff-work" && "$PHPR" composer-x/bin/composer --version > /dev/null 2>&1 ) || { echo "rc=8 smoke-phpr" > "$OUT/compoff2.done"; exit 8; }
tar czf "$GATES/compoff-work.tgz" -C "$SP" compoff-work || { echo "rc=8 retar" > "$OUT/compoff2.done"; exit 8; }
p "tarball ricongelato con composer-x"

floor3(){ local E="$1"; local v f1 f2 f3
  f1=$( { /usr/bin/time -l "$E" "$SP/compoff-work/composer-x/bin/composer" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f2=$( { /usr/bin/time -l "$E" "$SP/compoff-work/composer-x/bin/composer" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f3=$( { /usr/bin/time -l "$E" "$SP/compoff-work/composer-x/bin/composer" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}
CFO=$(floor3 "$ORACLE"); CFP=$(floor3 "$PHPR")
p "floors bin/composer: oracle=$CFO phpr=$CFP"

run_leg(){ # ENGINE LABEL
  local E="$1" L="$2"
  rm -rf "$SP/compoff-work"; tar xzf "$GATES/compoff-work.tgz" -C "$SP" || return 7
  p "compoff2 $L START"
  local ML=""
  [ "$E" = "$ORACLE" ] && ML="-d memory_limit=-1"
  ( cd "$SP/compoff-work" && rm -rf vendor && \
    COMPOSER_DISABLE_NETWORK=1 COMPOSER_CACHE_DIR="$SP/compoff-work/ccache" COMPOSER_HOME="$SP/compoff-work/chome" \
    /usr/bin/time -l perl -e 'alarm 1200; exec @ARGV or die' -- "$E" $ML composer-x/bin/composer install --no-interaction --no-security-blocking \
      > "$OUT/compoff2-$L.txt" 2> "$OUT/compoff2-$L.time" )
  local rc=$?
  [ -f "$SP/compoff-work/vendor/autoload.php" ] && echo "vendor_ok rc=$rc" >> "$OUT/compoff2-$L.txt"
  p "compoff2 $L rc=$rc"
}
for leg in 1 2; do
  run_leg "$ORACLE" "oracle$leg"
  run_leg "$PHPR"   "phpr$leg"
done

python3 - "$OUT" "$CFO" "$CFP" > "$VERD" 2>&1 <<'PY'
import re, sys, statistics
out, cfo, cfp = sys.argv[1], float(sys.argv[2]), float(sys.argv[3])
def t(f):
    d = {}
    for l in open(f):
        m = re.search(r'([\d.]+)\s+user', l)
        if m: d['user'] = float(m.group(1))
        m = re.search(r'([\d.]+)\s+sys', l)
        if m: d['sys'] = float(m.group(1))
        m = re.search(r'^\s*(\d+)\s+involuntary context switches', l)
        if m: d['ictx'] = int(m.group(1))
    return d
def ok(f):
    try: return 'vendor_ok' in open(f, errors='replace').read()
    except OSError: return False
print("== s126 compoff RIMISURA (composer ESTRATTO, criterio mappa2 p.7) ==")
print("grade=VERDICT  # derivazione meccanica dai .time")
print(f"floor bin/composer: oracle={cfo:.3f} phpr={cfp:.3f}")
ictx = {}
for leg in (1, 2):
    o = t(f"{out}/compoff2-oracle{leg}.time"); p_ = t(f"{out}/compoff2-phpr{leg}.time")
    vo, vp = ok(f"{out}/compoff2-oracle{leg}.txt"), ok(f"{out}/compoff2-phpr{leg}.txt")
    ictx[f"oracle{leg}"] = o.get('ictx', -1); ictx[f"phpr{leg}"] = p_.get('ictx', -1)
    if not (vo and vp):
        print(f"compoff2_leg{leg}: NULLA (vendor_ok oracle={vo} phpr={vp})"); continue
    raw = p_['user']/o['user']; net = (p_['user']-cfp)/(o['user']-cfo)
    print(f"compoff2_leg{leg}: oracle_user={o['user']:.2f} (sys={o.get('sys',0):.2f}) phpr_user={p_['user']:.2f} (sys={p_.get('sys',0):.2f}) ratio_raw={raw:.3f} ratio_net={net:.3f}")
vals = [v for v in ictx.values() if v >= 0]
med = statistics.median(vals) if vals else 0
flag = [k for k, v in ictx.items() if med > 0 and v > 1.5*med]
print("compoff2_ictx: " + " ".join(f"{k}={v}" for k, v in ictx.items()) + (f"  SEGNALATE(>1,5x med): {flag}" if flag else "  contesa ok"))
PY
echo "rc=0 $(date +%T)" > "$OUT/compoff2.done"
p "DONE"
