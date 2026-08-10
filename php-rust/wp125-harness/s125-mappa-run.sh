#!/bin/bash
# s125-mappa-run.sh — MAPPA perf v1 (criterio s125-criterio-mappa.md):
# ORM + http-kernel, N=2 per lato, oracle prima poi phpr per gamba,
# workspace ri-untarrato, watchdog, /usr/bin/time -l, pavimenti per-binario.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp125-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
SP="${MAPPA_SP:?MAPPA_SP (workdir APFS) richiesto}"
OUT="$H/mappa-out"; mkdir -p "$OUT"
VERD="$H/s125-mappa-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/mappa.done"
[ "$(shasum -a 256 "$PHPR" | cut -c1-16)" = "002e6cc12047ab9f" ] || { echo "rc=9 pin!=s125" > "$OUT/mappa.done"; exit 9; }

names(){ tr -d '\0' < "$1" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u; }
summ(){ tr -d '\0' < "$1" | grep -E "^(Tests:|OK)" | tail -1; }

# pavimenti per-binario (med3 di `phpunit --version` nel workspace ORM)
rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=8 untar" > "$OUT/mappa.done"; exit 8; }
floor3(){ local E="$1" f1 f2 f3
  f1=$( { /usr/bin/time -l "$E" "$SP/orm-work/vendor/bin/phpunit" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f2=$( { /usr/bin/time -l "$E" "$SP/orm-work/vendor/bin/phpunit" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f3=$( { /usr/bin/time -l "$E" "$SP/orm-work/vendor/bin/phpunit" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}
FO=$(floor3 "$ORACLE"); FP=$(floor3 "$PHPR")
echo "floor_user_med3 oracle=$FO phpr=$FP (phpunit --version, pavimento PARZIALE dichiarato)" > "$OUT/floors.txt"
p "floors: oracle=$FO phpr=$FP"

run_leg(){ # W TGZ DIR TIMEOUT ENGINE LABEL
  local W="$1" TGZ="$2" DIR="$3" T="$4" E="$5" L="$6"
  rm -rf "$SP/$DIR"; tar xzf "$GATES/$TGZ" -C "$SP" || return 7
  p "$W $L START"
  # EMENDA criterio p.7: l'oracle gira con memory_limit=-1 (phpr non applica
  # il limite, §3.14 stub — parità di condizioni; il default 128M uccideva
  # la gamba ORM a metà suite).
  local ML=""
  [ "$E" = "$ORACLE" ] && ML="-d memory_limit=-1"
  ( cd "$SP/$DIR" && "$WD" -t "$T" -s 600 -p "$OUT/$W-$L.txt" -o "$OUT" -- \
      /usr/bin/time -l "$E" $ML vendor/bin/phpunit --no-coverage > "$OUT/$W-$L.txt" 2> "$OUT/$W-$L.time" )
  local rc=$?
  p "$W $L rc=$rc"
  return 0
}

WLS="${WORKLOADS:-orm hk}"
for leg in 1 2; do
  case " $WLS " in *" orm "*)
    run_leg orm orm-work.tgz orm-work 3600 "$ORACLE" "oracle$leg"
    run_leg orm orm-work.tgz orm-work 3600 "$PHPR"   "phpr$leg" ;;
  esac
  case " $WLS " in *" hk "*)
    run_leg hk  hk-work.tgz  hk-work  1800 "$ORACLE" "oracle$leg"
    run_leg hk  hk-work.tgz  hk-work  1800 "$PHPR"   "phpr$leg" ;;
  esac
done

# parità: ORM phpr per NOME vs baseline; hk phpr 0E/0F
RC=0
for leg in 1 2; do
  names "$OUT/orm-phpr$leg.txt" > "$OUT/orm-phpr$leg.failnames"
  if ! diff -q "$H/orm-baseline-failnames.txt" "$OUT/orm-phpr$leg.failnames" > /dev/null; then
    p "orm phpr$leg: fail-set DIVERGE — cifra NULLA"; RC=1
  fi
  HS=$(summ "$OUT/hk-phpr$leg.txt")
  HE=$(printf '%s' "$HS" | sed -n 's/.*Errors: \([0-9]*\).*/\1/p'); HE=${HE:-0}
  HF=$(printf '%s' "$HS" | sed -n 's/.*Failures: \([0-9]*\).*/\1/p'); HF=${HF:-0}
  if [ "$HE" != 0 ] || [ "$HF" != 0 ]; then p "hk phpr$leg: E=$HE F=$HF — cifra NULLA"; RC=1; fi
done

python3 - "$OUT" "$FO" "$FP" > "$OUT/ratios.txt" <<'PY'
import re, sys, statistics
out, fo, fp = sys.argv[1], float(sys.argv[2]), float(sys.argv[3])
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
print("mappa-v1 (user CPU; pavimento PARZIALE phpunit --version: oracle %.3f phpr %.3f)" % (fo, fp))
print("grade=VERDICT  # derivazione meccanica dai .time")
for w in ("orm", "hk"):
    ictx = {}
    for leg in (1, 2):
        o = t(f"{out}/{w}-oracle{leg}.time"); p_ = t(f"{out}/{w}-phpr{leg}.time")
        raw = p_['user']/o['user']
        net = (p_['user']-fp)/(o['user']-fo)
        ictx[f"oracle{leg}"] = o.get('ictx', -1); ictx[f"phpr{leg}"] = p_.get('ictx', -1)
        print(f"{w}_leg{leg}: oracle_user={o['user']:.2f} (sys={o.get('sys',0):.2f}) phpr_user={p_['user']:.2f} (sys={p_.get('sys',0):.2f}) ratio_raw={raw:.3f} ratio_net={net:.3f}")
    vals = [v for v in ictx.values() if v >= 0]
    med = statistics.median(vals) if vals else 0
    flag = [k for k, v in ictx.items() if med > 0 and v > 1.5*med]
    print(f"{w}_ictx: " + " ".join(f"{k}={v}" for k, v in ictx.items()) + (f"  SEGNALATE(>1,5x med): {flag}" if flag else "  contesa ok"))
PY

{ echo "== s125 mappa perf v1 (pin s125 002e6cc1 vs oracle 8.5.7; criterio s125-criterio-mappa.md) =="
  cat "$OUT/floors.txt"
  cat "$OUT/ratios.txt"
  echo "parita': orm phpr per NOME vs baseline 16 · hk phpr 0E/0F — rc=$RC (0=tutte valide)"
  for leg in 1 2; do
    echo "orm oracle$leg: $(summ "$OUT/orm-oracle$leg.txt")"
    echo "orm phpr$leg:   $(summ "$OUT/orm-phpr$leg.txt")"
    echo "hk  oracle$leg: $(summ "$OUT/hk-oracle$leg.txt")"
    echo "hk  phpr$leg:   $(summ "$OUT/hk-phpr$leg.txt")"
  done
} > "$VERD"
echo "rc=$RC $(date +%T)" > "$OUT/mappa.done"
p "DONE rc=$RC"
