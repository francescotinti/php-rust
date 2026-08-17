#!/bin/bash
# s150-orm-coppia.sh — coppia dbal+ORM sul pin s150 = GIUDIZIO della scommessa
# BT1 (criterio s150-criterio-orm.md; copia DICHIARATA di s147-orm-rimisura.sh,
# manifest s150-orm-copia.diff; adattamenti: pin s150 cbbe71735effb165, path
# wp150, blocco SCOMMESSA meccanico in coda al verdetto): ORM + dbal, N=2 per
# lato, oracle prima poi phpr per gamba, workspace ri-untarrato, watchdog,
# /usr/bin/time -l, pavimenti per-binario PER WORKSPACE, gate contesa ictx/s.
# ADATTAMENTO lock (ereditato): il lock misura è della SESSIONE — lo
# script lo VERIFICA soltanto (veto «lock di finestra con trap EXIT altrui»).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp150-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
SP="${MAPPA_SP:?MAPPA_SP (workdir APFS) richiesto}"
OUT="$H/orm-out"; mkdir -p "$OUT"
VERD="$H/s150-orm-coppia-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/rimisura.done"
[ "$(shasum -a 256 "$PHPR" | cut -c1-16)" = "cbbe71735effb165" ] || { echo "rc=9 pin!=s150" > "$OUT/rimisura.done"; exit 9; }
# lock della SESSIONE: si VERIFICA soltanto (niente creazione né trap).
LOCK=/private/tmp/phpr-measure.lock
[ -e "$LOCK" ] || { echo "rc=6 measure-lock ASSENTE (finestra non aperta)" > "$OUT/rimisura.done"; exit 6; }

names(){ tr -d '\0' < "$1" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u; }
summ(){ tr -d '\0' < "$1" | grep -E "^(Tests:|OK)" | tail -1; }

# pavimenti per-binario (med3 di `phpunit --version`) PER WORKSPACE — MISURATI
# su QUESTO pin (az.rev.4: mai ereditati).
rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=8 untar" > "$OUT/rimisura.done"; exit 8; }
rm -rf "$SP/dbal-work"; tar xzf "$GATES/dbal-work.tgz" -C "$SP" || { echo "rc=8 untar-dbal" > "$OUT/rimisura.done"; exit 8; }
floor3(){ local E="$1" P="$2" f1 f2 f3
  f1=$( { /usr/bin/time -l "$E" "$P" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f2=$( { /usr/bin/time -l "$E" "$P" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f3=$( { /usr/bin/time -l "$E" "$P" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}
FO=$(floor3 "$ORACLE" "$SP/orm-work/vendor/bin/phpunit"); FP=$(floor3 "$PHPR" "$SP/orm-work/vendor/bin/phpunit")
DFO=$(floor3 "$ORACLE" "$SP/dbal-work/vendor/bin/phpunit"); DFP=$(floor3 "$PHPR" "$SP/dbal-work/vendor/bin/phpunit")
echo "floor_user_med3 orm: oracle=$FO phpr=$FP · dbal: oracle=$DFO phpr=$DFP (phpunit --version, pavimento PARZIALE dichiarato)" > "$OUT/floors.txt"
p "floors: orm o=$FO p=$FP · dbal o=$DFO p=$DFP"

run_leg(){ # W TGZ DIR TIMEOUT ENGINE LABEL
  local W="$1" TGZ="$2" DIR="$3" T="$4" E="$5" L="$6"
  rm -rf "$SP/$DIR"; tar xzf "$GATES/$TGZ" -C "$SP" || return 7
  p "$W $L START"
  # EMENDA ereditata (s147 p.7): oracle con memory_limit=-1 (§3.14, parità
  # di condizioni).
  local ML=""
  [ "$E" = "$ORACLE" ] && ML="-d memory_limit=-1"
  ( cd "$SP/$DIR" && "$WD" -t "$T" -s 600 -p "$OUT/$W-$L.txt" -o "$OUT" -- \
      /usr/bin/time -l "$E" $ML vendor/bin/phpunit --no-coverage > "$OUT/$W-$L.txt" 2> "$OUT/$W-$L.time" )
  local rc=$?
  p "$W $L rc=$rc"
  return 0
}

WLS="${WORKLOADS:-orm dbal}"
for leg in 1 2; do
  case " $WLS " in *" orm "*)
    run_leg orm orm-work.tgz orm-work 3600 "$ORACLE" "oracle$leg"
    run_leg orm orm-work.tgz orm-work 3600 "$PHPR"   "phpr$leg" ;;
  esac
  case " $WLS " in *" dbal "*)
    run_leg dbal dbal-work.tgz dbal-work 3600 "$ORACLE" "oracle$leg"
    run_leg dbal dbal-work.tgz dbal-work 3600 "$PHPR"   "phpr$leg" ;;
  esac
done

# parità: ORM phpr per NOME vs baseline 16; dbal fail-set phpr STABILE tra gambe
RC=0
for leg in 1 2; do
  names "$OUT/orm-phpr$leg.txt" > "$OUT/orm-phpr$leg.failnames"
  if ! diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/orm-phpr$leg.failnames" > /dev/null; then
    p "orm phpr$leg: fail-set DIVERGE — cifra NULLA"; RC=1
  fi
  names "$OUT/dbal-phpr$leg.txt"   > "$OUT/dbal-phpr$leg.failnames"
  names "$OUT/dbal-oracle$leg.txt" > "$OUT/dbal-oracle$leg.failnames"
done
if ! diff -q "$OUT/dbal-phpr1.failnames" "$OUT/dbal-phpr2.failnames" > /dev/null; then
  p "dbal: fail-set phpr NON stabile tra gambe — cifra NULLA"; RC=1
fi

python3 - "$OUT" "$FO" "$FP" "$DFO" "$DFP" "$RC" > "$OUT/ratios.txt" <<'PY'
import re, sys, statistics, os
out = sys.argv[1]
fo, fp, dfo, dfp = map(float, sys.argv[2:6])
parita_rc = int(sys.argv[6])
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
def failset(f):
    return set(x.strip() for x in open(f) if x.strip()) if os.path.exists(f) else set()
print("coppia-s150 (user CPU; pavimenti PARZIALI phpunit --version: orm o=%.3f p=%.3f · dbal o=%.3f p=%.3f)" % (fo, fp, dfo, dfp))
print("grade=VERDICT  # derivazione meccanica dai .time")
new_net = {}
for w, flo, flp in (("orm", fo, fp), ("dbal", dfo, dfp)):
    rate = {}; absi = {}
    for leg in (1, 2):
        o = t(f"{out}/{w}-oracle{leg}.time"); p_ = t(f"{out}/{w}-phpr{leg}.time")
        raw = p_['user']/o['user']
        net = (p_['user']-flp)/(o['user']-flo)
        new_net.setdefault(w, []).append(p_['user']-flp)
        for k, d in ((f"oracle{leg}", o), (f"phpr{leg}", p_)):
            absi[k] = d.get('ictx', -1)
            rate[k] = (d['ictx']/d['real']) if ('ictx' in d and d.get('real', 0) > 0) else -1
        print(f"{w}_leg{leg}: oracle_user={o['user']:.2f} (sys={o.get('sys',0):.2f}) phpr_user={p_['user']:.2f} (sys={p_.get('sys',0):.2f}) ratio_raw={raw:.3f} ratio_net={net:.3f}")
    vals = [v for v in rate.values() if v >= 0]
    med = statistics.median(vals) if vals else 0
    flag = [k for k, v in rate.items() if med > 0 and v > 1.5*med]
    print(f"{w}_ictx_abs: " + " ".join(f"{k}={absi[k]}" for k in absi))
    print(f"{w}_ictx/s (gate emenda S-127): " + " ".join(f"{k}={rate[k]:.1f}" for k in rate) + (f"  SEGNALATE(>1,5x med): {flag}" if flag else "  contesa ok"))
s1, s2 = failset(f"{out}/dbal-phpr1.failnames"), failset(f"{out}/dbal-phpr2.failnames")
o1 = failset(f"{out}/dbal-oracle1.failnames")
diff = sorted(s1 - o1)
print(f"dbal_parita': phpr_failset_stabile={s1 == s2} |phpr_fail|={len(s1)} |diff_vs_oracle|={len(diff)}")
if diff[:20]: print("dbal_diff_nomi: " + " | ".join(diff[:20]))
# --- SCOMMESSA BT1 (criterio s150-criterio-orm.md p.4, aritmetica meccanica) ---
REF_MIN, REF_MAX = 41.60, 42.22   # phpr ORM net @ s145 (s147 verdetto, floor 0,07)
RES = 0.293                        # risoluzione KS-146-1
ATT_LO, ATT_HI = 0.8, 3.1          # attesa pre-registrata
nn = sorted(new_net.get("orm", []))
if len(nn) == 2:
    d_min = REF_MIN - nn[-1]; d_max = REF_MAX - nn[0]
    print(f"scommessa: phpr_orm_net_new=[{nn[0]:.2f}; {nn[1]:.2f}] ref=[{REF_MIN}; {REF_MAX}] Delta=[{d_min:+.2f}; {d_max:+.2f}] s")
    if parita_rc != 0:
        print("scommessa: CIFRA NULLA (parita' rotta) — nessun giudizio")
    elif d_min >= RES:
        centro = "CENTRATA (dentro l'attesa 0,8-3,1)" if (d_max >= ATT_LO and d_min <= ATT_HI) else ("OLTRE-ATTESA (>3,1: dichiarare)" if d_min > ATT_HI else "VINTA ma SOTTO l'attesa 0,8 (dichiarare)")
        print(f"scommessa: VINTA (Delta_min {d_min:.2f} >= {RES}) — {centro}")
    elif d_max < RES:
        print(f"scommessa: PERSA (Delta_max {d_max:.2f} < {RES}: fermo nel rumore o su)")
    else:
        print(f"scommessa: NON RISOLTA (intervallo Delta a cavallo di {RES}) — replica dovuta")
PY

{ echo "== s150 coppia dbal+ORM (pin s150 cbbe7173 vs oracle 8.5.7; criterio s150-criterio-orm.md) =="
  cat "$OUT/floors.txt"
  cat "$OUT/ratios.txt"
  echo "parita': orm phpr per NOME vs baseline 16 · dbal fail-set stabile tra gambe — rc=$RC (0=tutte valide)"
  for leg in 1 2; do
    echo "orm  oracle$leg: $(summ "$OUT/orm-oracle$leg.txt")"
    echo "orm  phpr$leg:   $(summ "$OUT/orm-phpr$leg.txt")"
    echo "dbal oracle$leg: $(summ "$OUT/dbal-oracle$leg.txt")"
    echo "dbal phpr$leg:   $(summ "$OUT/dbal-phpr$leg.txt")"
  done
} > "$VERD"
echo "rc=$RC $(date +%T)" > "$OUT/rimisura.done"
p "DONE rc=$RC"
