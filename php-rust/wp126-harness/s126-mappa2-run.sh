#!/bin/bash
# s126-mappa2-run.sh — MAPPA perf v2 (criterio s126-criterio-mappa2.md):
# dbal + http-foundation + collections + composer-install-OFFLINE.
# N=2 per lato, oracle prima (memory_limit=-1), workspace ri-untarrato per gamba.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
SP="${MAPPA_SP:?MAPPA_SP (workdir) richiesto}"
OUT="$H/mappa2-out"; mkdir -p "$OUT"
VERD="$H/s126-mappa2-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/mappa2.done"
[ "$(shasum -a 256 "$PHPR" | cut -c1-16)" = "002e6cc12047ab9f" ] || { echo "rc=9 pin!=s125" > "$OUT/mappa2.done"; exit 9; }

names(){ tr -d '\0' < "$1" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u; }
summ(){ tr -d '\0' < "$1" | grep -E "^(Tests:|OK)" | tail -1; }

# pavimenti per-binario: phpunit --version nel workspace dbal; composer --version
rm -rf "$SP/dbal-work"; tar xzf "$GATES/dbal-work.tgz" -C "$SP" || { echo "rc=8 untar-dbal" > "$OUT/mappa2.done"; exit 8; }
floor3(){ local E="$1"; shift; local f1 f2 f3
  f1=$( { /usr/bin/time -l "$E" "$@" > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f2=$( { /usr/bin/time -l "$E" "$@" > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f3=$( { /usr/bin/time -l "$E" "$@" > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}
FO=$(floor3 "$ORACLE" "$SP/dbal-work/vendor/bin/phpunit" --version)
FP=$(floor3 "$PHPR" "$SP/dbal-work/vendor/bin/phpunit" --version)
CFO=$(floor3 "$ORACLE" "$GATES/composer.phar" --version)
CFP=$(floor3 "$PHPR" "$GATES/composer.phar" --version)
echo "floor phpunit oracle=$FO phpr=$FP · composer oracle=$CFO phpr=$CFP" > "$OUT/floors.txt"
p "floors: $(cat "$OUT/floors.txt")"

run_suite(){ # W DIR T ENGINE LABEL
  local W="$1" DIR="$2" T="$3" E="$4" L="$5"
  rm -rf "$SP/$DIR"; tar xzf "$GATES/$DIR.tgz" -C "$SP" || return 7
  p "$W $L START"
  local ML=""
  [ "$E" = "$ORACLE" ] && ML="-d memory_limit=-1"
  ( cd "$SP/$DIR" && "$WD" -t "$T" -s 600 -p "$OUT/$W-$L.txt" -o "$OUT" -- \
      /usr/bin/time -l "$E" $ML vendor/bin/phpunit --no-coverage > "$OUT/$W-$L.txt" 2> "$OUT/$W-$L.time" )
  p "$W $L rc=$?"
}
run_compoff(){ # ENGINE LABEL
  local E="$1" L="$2"
  rm -rf "$SP/compoff-work"; tar xzf "$GATES/compoff-work.tgz" -C "$SP" || return 7
  p "compoff $L START"
  local ML=""
  [ "$E" = "$ORACLE" ] && ML="-d memory_limit=-1"
  ( cd "$SP/compoff-work" && rm -rf vendor && \
    COMPOSER_DISABLE_NETWORK=1 COMPOSER_CACHE_DIR="$SP/compoff-work/ccache" COMPOSER_HOME="$SP/compoff-work/chome" \
    /usr/bin/time -l perl -e 'alarm 1200; exec @ARGV or die' -- "$E" $ML "$GATES/composer.phar" install --no-interaction --no-audit \
      > "$OUT/compoff-$L.txt" 2> "$OUT/compoff-$L.time" )
  local rc=$?
  [ -f "$SP/compoff-work/vendor/autoload.php" ] && echo "vendor_ok" >> "$OUT/compoff-$L.txt"
  p "compoff $L rc=$rc"
}

for leg in 1 2; do
  run_suite dbal dbal-work 3600 "$ORACLE" "oracle$leg"
  run_suite dbal dbal-work 3600 "$PHPR"   "phpr$leg"
  run_suite hf   hf-work   1800 "$ORACLE" "oracle$leg"
  run_suite hf   hf-work   1800 "$PHPR"   "phpr$leg"
  run_suite coll coll-work 1800 "$ORACLE" "oracle$leg"
  run_suite coll coll-work 1800 "$PHPR"   "phpr$leg"
  run_compoff "$ORACLE" "oracle$leg"
  run_compoff "$PHPR"   "phpr$leg"
done

for W in dbal hf coll; do
  for leg in 1 2; do
    names "$OUT/$W-oracle$leg.txt" > "$OUT/$W-oracle$leg.failnames"
    names "$OUT/$W-phpr$leg.txt"   > "$OUT/$W-phpr$leg.failnames"
  done
done

python3 - "$OUT" "$FO" "$FP" "$CFO" "$CFP" > "$OUT/ratios.txt" 2>&1 <<'PY'
import re, sys, statistics, os
out, fo, fp, cfo, cfp = sys.argv[1], *map(float, sys.argv[2:6])
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
def failset(f):
    return set(x.strip() for x in open(f) if x.strip()) if os.path.exists(f) else set()
print("mappa2 (user CPU; pavimenti: phpunit o=%.3f p=%.3f, composer o=%.3f p=%.3f)" % (fo, fp, cfo, cfp))
print("grade=VERDICT  # derivazione meccanica dai .time")
for w, flo, flp in (("dbal", fo, fp), ("hf", fo, fp), ("coll", fo, fp), ("compoff", cfo, cfp)):
    ictx = {}
    for leg in (1, 2):
        try:
            o = t(f"{out}/{w}-oracle{leg}.time"); p_ = t(f"{out}/{w}-phpr{leg}.time")
            raw = p_['user']/o['user']
            net = (p_['user']-flp)/(o['user']-flo)
            ictx[f"oracle{leg}"] = o.get('ictx', -1); ictx[f"phpr{leg}"] = p_.get('ictx', -1)
            print(f"{w}_leg{leg}: oracle_user={o['user']:.2f} (sys={o.get('sys',0):.2f}) phpr_user={p_['user']:.2f} (sys={p_.get('sys',0):.2f}) ratio_raw={raw:.3f} ratio_net={net:.3f}")
        except Exception as e:
            print(f"{w}_leg{leg}: ESTRAZIONE FALLITA ({e}) — gamba NULLA")
    vals = [v for v in ictx.values() if v >= 0]
    med = statistics.median(vals) if vals else 0
    flag = [k for k, v in ictx.items() if med > 0 and v > 1.5*med]
    print(f"{w}_ictx: " + " ".join(f"{k}={v}" for k, v in ictx.items()) + (f"  SEGNALATE(>1,5x med): {flag}" if flag else "  contesa ok"))
    if w != "compoff":
        s1, s2 = failset(f"{out}/{w}-phpr1.failnames"), failset(f"{out}/{w}-phpr2.failnames")
        o1 = failset(f"{out}/{w}-oracle1.failnames")
        stab = (s1 == s2)
        diff = sorted(s1 - o1)
        print(f"{w}_parita': phpr_failset_stabile={stab} |phpr_fail|={len(s1)} |diff_vs_oracle|={len(diff)}")
        if diff[:20]: print(f"{w}_diff_nomi: " + " | ".join(diff[:20]))
PY

{ echo "== s126 mappa perf v2 (pin s125 002e6cc1 vs oracle 8.5.7; criterio s126-criterio-mappa2.md) =="
  cat "$OUT/floors.txt"
  cat "$OUT/ratios.txt"
  for w in dbal hf coll; do
    for leg in 1 2; do
      echo "$w oracle$leg: $(summ "$OUT/$w-oracle$leg.txt")"
      echo "$w phpr$leg:   $(summ "$OUT/$w-phpr$leg.txt")"
    done
  done
  for leg in 1 2; do
    echo "compoff oracle$leg: $(tail -2 "$OUT/compoff-oracle$leg.txt" | tr '\n' ' ')"
    echo "compoff phpr$leg:   $(tail -2 "$OUT/compoff-phpr$leg.txt" | tr '\n' ' ')"
  done
} > "$VERD"
echo "rc=0 $(date +%T)" > "$OUT/mappa2.done"
p "DONE"
