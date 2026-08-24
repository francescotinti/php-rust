#!/bin/bash
# s157-orm-replica3.sh — INDAGINE regressione-segnalata attesa-HD2 (criterio
# s157-criterio-orm.md p.5: «indagine PRIMA di ogni altra leva»). DERIVATO
# DICHIARATO di s157-orm-coppia.sh: UNA terza gamba ORM (oracle poi phpr),
# stessa meccanica (untar fresco, floors med3, watchdog, memory_limit oracle),
# in finestra CERTIFICATA da quiescenza (il run notturno non la aveva per
# gamba). NON tocca il verdetto della coppia: scrive il proprio .out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp157-harness"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
SP="${MAPPA_SP:?MAPPA_SP richiesto}"
OUT="$H/orm-out"; mkdir -p "$OUT"
VERD="$H/s157-orm-replica3-verdetto.out"
p(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress-replica3.txt"; }
: > "$OUT/progress-replica3.txt"
rm -f "$OUT/replica3.done"
[ "$(shasum -a 256 "$PHPR" | cut -c1-16)" = "42efea3e34feb390" ] || { echo "rc=9 pin!=s156" > "$OUT/replica3.done"; exit 9; }
[ -e /private/tmp/phpr-measure.lock ] || { echo "rc=6 lock assente" > "$OUT/replica3.done"; exit 6; }
"$QUIESCE" "$OUT/quiesce-replica3.rc" > "$OUT/quiesce-replica3.log" 2>&1
QRC=$(cat "$OUT/quiesce-replica3.rc" 2>/dev/null || echo MANCANTE)
[ "$QRC" = 0 ] || { echo "rc=8 quiescenza=$QRC" > "$OUT/replica3.done"; exit 8; }
rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=8 untar" > "$OUT/replica3.done"; exit 8; }
floor3(){ local E="$1" P="$2" f1 f2 f3
  f1=$( { /usr/bin/time -l "$E" "$P" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f2=$( { /usr/bin/time -l "$E" "$P" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  f3=$( { /usr/bin/time -l "$E" "$P" --version > /dev/null; } 2>&1 | awk '/[0-9.]+ *user/{for(i=1;i<=NF;i++) if($i=="user"){print $(i-1)}}' )
  printf '%s\n%s\n%s\n' "$f1" "$f2" "$f3" | sort -n | awk 'NR==2'
}
FO=$(floor3 "$ORACLE" "$SP/orm-work/vendor/bin/phpunit"); FP=$(floor3 "$PHPR" "$SP/orm-work/vendor/bin/phpunit")
p "floors o=$FO p=$FP"
run_leg(){ # ENGINE LABEL ML
  local E="$1" L="$2" ML="$3"
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || return 7
  p "orm $L START"
  ( cd "$SP/orm-work" && "$WD" -t 3600 -s 600 -p "$OUT/orm-$L.txt" -o "$OUT" -- \
      /usr/bin/time -l "$E" $ML vendor/bin/phpunit --no-coverage > "$OUT/orm-$L.txt" 2> "$OUT/orm-$L.time" )
  p "orm $L rc=$?"
}
run_leg "$ORACLE" "oracle3" "-d memory_limit=-1"
run_leg "$PHPR"   "phpr3"   ""
LC_ALL=C tr -d '\0' < "$OUT/orm-phpr3.txt" | LC_ALL=C sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | LC_ALL=C sort -u > "$OUT/orm-phpr3.failnames"
RC=0
diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/orm-phpr3.failnames" > /dev/null || RC=1
python3 - "$OUT" "$FO" "$FP" "$RC" "$QRC" > "$VERD" <<'PY'
import re, sys
out, fo, fp, rc, qrc = sys.argv[1], float(sys.argv[2]), float(sys.argv[3]), sys.argv[4], sys.argv[5]
def t(f):
    d={}
    for l in open(f):
        for k,pat in (("real",r'([\d.]+)\s+real'),("user",r'([\d.]+)\s+user'),("sys",r'([\d.]+)\s+sys')):
            m=re.search(pat,l)
            if m: d[k]=float(m.group(1))
        m=re.search(r'^\s*(\d+)\s+involuntary context switches',l)
        if m: d['ictx']=int(m.group(1))
    return d
o=t(f"{out}/orm-oracle3.time"); p_=t(f"{out}/orm-phpr3.time")
print("== s157 orm replica3 (INDAGINE attesa-HD2 REGRESSIONE SEGNALATA; finestra certificata quiescenza rc=%s; parita' rc=%s) ==" % (qrc, rc))
print(f"floors: o={fo} p={fp}")
print(f"orm_leg3: oracle_user={o['user']:.2f} (sys={o.get('sys',0):.2f}) phpr_user={p_['user']:.2f} (sys={p_.get('sys',0):.2f}) ratio_raw={p_['user']/o['user']:.3f} ratio_net={(p_['user']-fp)/(o['user']-fo):.3f}")
print(f"ictx: oracle3={o.get('ictx',-1)} ({o.get('ictx',0)/o.get('real',1):.1f}/s) phpr3={p_.get('ictx',-1)} ({p_.get('ictx',0)/p_.get('real',1):.1f}/s)")
print(f"phpr_net3={p_['user']-fp:.2f} oracle_net3={o['user']-fo:.2f}  # confronto: notte [34.82;34.91]/[4.94] · ref s154-window [34.30;34.35]/[4.87;4.92]")
PY
echo "rc=$RC $(date +%T)" > "$OUT/replica3.done"
p "DONE rc=$RC"
