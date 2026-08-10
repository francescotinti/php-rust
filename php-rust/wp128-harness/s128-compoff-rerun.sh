#!/bin/bash
# s128-compoff-rerun.sh — rimisura compoff su pin s127b (criterio
# s128-criterio-compoff.md, committato PRIMA di questo codice — az.rev. #5).
# ATTENDE pair-intercal.done (run sequenziali), poi: rebuild tarball con
# composer-x + smoke bilaterale + 2 gambe per lato. Cifra canonica = ratio_NET;
# gate contesa in ictx/s (emenda S-127). rc autoritativo = SOLO
# compoff-out/compoff.done scritto QUI (az.rev. #2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
SP="${COMP_SP:?COMP_SP (workdir) richiesto}"
OUT="$H/compoff-out"; mkdir -p "$OUT"
VERD="$H/s128-compoff-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/compoff-progress.txt"; }
: > "$OUT/compoff-progress.txt"
rm -f "$OUT/compoff.done"
[ "$(shasum -a 256 "$PHPR" | cut -c1-16)" = "ccb63dcaf565cffc" ] || { echo "rc=9 pin!=s127b" > "$OUT/compoff.done"; exit 9; }

p "attesa pair-intercal.done"
i=0; while [ ! -f "$H/pair-out/pair-intercal.done" ] && [ $i -lt 1080 ]; do /bin/sleep 20; i=$((i+1)); done
[ -f "$H/pair-out/pair-intercal.done" ] || { echo "rc=9 timeout-attesa-pair" > "$OUT/compoff.done"; exit 9; }
p "pair conclusa: $(cat "$H/pair-out/pair-intercal.done")"

# rebuild: composer-x dentro il workspace, poi CONGELA il tarball prima delle gambe
mkdir -p "$SP"
rm -rf "$SP/compoff-work"; tar xzf "$GATES/compoff-work.tgz" -C "$SP" || { echo "rc=8 untar" > "$OUT/compoff.done"; exit 8; }
( cd "$SP/compoff-work" && rm -rf composer-x && \
  PHAR_SRC="$GATES/composer.phar" "$ORACLE" -r '$p = new Phar(getenv("PHAR_SRC")); $p->extractTo("composer-x", null, true);' ) \
  || { echo "rc=8 extract" > "$OUT/compoff.done"; exit 8; }
# smoke bilaterale rc=0 ESATTO (forge-silent-failure)
( cd "$SP/compoff-work" && "$ORACLE" composer-x/bin/composer --version > /dev/null 2>&1 ) || { echo "rc=8 smoke-oracle" > "$OUT/compoff.done"; exit 8; }
( cd "$SP/compoff-work" && "$PHPR" composer-x/bin/composer --version > /dev/null 2>&1 ) || { echo "rc=8 smoke-phpr" > "$OUT/compoff.done"; exit 8; }
tar czf "$GATES/compoff-work.tgz" -C "$SP" compoff-work || { echo "rc=8 retar" > "$OUT/compoff.done"; exit 8; }
p "tarball ricongelato con composer-x"

floor3(){ local E="$1"; local f1 f2 f3
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
  p "compoff $L START"
  local ML=""
  [ "$E" = "$ORACLE" ] && ML="-d memory_limit=-1"
  ( cd "$SP/compoff-work" && rm -rf vendor && \
    COMPOSER_DISABLE_NETWORK=1 COMPOSER_CACHE_DIR="$SP/compoff-work/ccache" COMPOSER_HOME="$SP/compoff-work/chome" \
    /usr/bin/time -l perl -e 'alarm 1200; exec @ARGV or die' -- "$E" $ML composer-x/bin/composer install --no-interaction --no-security-blocking \
      > "$OUT/compoff-$L.txt" 2> "$OUT/compoff-$L.time" )
  local rc=$?
  [ -f "$SP/compoff-work/vendor/autoload.php" ] && echo "vendor_ok rc=$rc" >> "$OUT/compoff-$L.txt"
  p "compoff $L rc=$rc"
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
        m = re.search(r'([\d.]+)\s+real', l)
        if m: d['real'] = float(m.group(1))
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
print("== s128 compoff RIMISURA su pin s127b (composer ESTRATTO; canonica=NET, emenda S-127) ==")
print("grade=VERDICT  # derivazione meccanica dai .time; rc autoritativo = compoff-out/compoff.done")
print(f"floor bin/composer: oracle={cfo:.3f} phpr={cfp:.3f}")
ictx_s = {}
for leg in (1, 2):
    o = t(f"{out}/compoff-oracle{leg}.time"); p_ = t(f"{out}/compoff-phpr{leg}.time")
    vo, vp = ok(f"{out}/compoff-oracle{leg}.txt"), ok(f"{out}/compoff-phpr{leg}.txt")
    for k, d in ((f"oracle{leg}", o), (f"phpr{leg}", p_)):
        ictx_s[k] = (d['ictx']/d['real']) if ('ictx' in d and d.get('real', 0) > 0) else -1.0
    if not (vo and vp):
        print(f"compoff_leg{leg}: NULLA (vendor_ok oracle={vo} phpr={vp})"); continue
    raw = p_['user']/o['user']; net = (p_['user']-cfp)/(o['user']-cfo)
    print(f"compoff_leg{leg}: oracle_user={o['user']:.2f} (sys={o.get('sys',0):.2f} real={o.get('real',0):.2f}) phpr_user={p_['user']:.2f} (sys={p_.get('sys',0):.2f} real={p_.get('real',0):.2f}) ratio_net={net:.3f} (raw={raw:.3f})")
vals = [v for v in ictx_s.values() if v >= 0]
med = statistics.median(vals) if vals else 0
flag = [k for k, v in ictx_s.items() if med > 0 and v > 1.5*med]
print("compoff_ictx_per_s: " + " ".join(f"{k}={v:.1f}" for k, v in ictx_s.items()) + (f"  SEGNALATE(>1,5x med ictx/s): {flag}" if flag else "  contesa ok"))
PY
echo "rc=0 $(date +%T)" > "$OUT/compoff.done"
p "DONE (fonte rc: $OUT/compoff.done)"
