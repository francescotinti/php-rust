#!/bin/bash
# s157-ab-al1.sh <BPATH> <BEXP8> <TAG> <R> [DSMOKE_MISSLOAD] — LEVA
# L-AL1 miss/autoload plumbing 0-alloc (criterio s157-criterio-al1.md).
# COPIA DICHIARATA di s156-ab-hd2host.sh coi SOLI adattamenti del criterio:
#  (a) GIUDICE: missload (m-missload, wp157-harness) con N=10000000
#      DICHIARATO (letterale nel sorgente; n_fixed lo pinna comunque);
#  (b) UB-alloc FALSIFICABILE = 20.7 (3 alloc/miss x miheap 6.9);
#  (c) guardie: hostargs (m-hostargs N=10M, NUOVA: hit-path class_exists) +
#      backtrace24 (N=2400000) + obj* + le sei; comparatore STRETTO
#      pre-registrato (az.rev. S-156 #1): morde sse D < -soglia;
#  (d) A = GEMELLO RICOSTRUITO sul tree s156 (atteso 42efea3e, criterio p.6);
#  (e) lock/verdetto/tag s157.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="$H/.."
M="$REPO/wp127-harness/micro-orm"
M97="$REPO/wp123-harness/scaled"
M149="$REPO/wp149-harness"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s156-gemelloA"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
LOCK=/private/tmp/phpr-measure.lock
BB="${1:?BPATH}"; BEXP="${2:?BEXP8}"; TAG="${3:?TAG}"; R="${4:?R}"; DSM="${5:-}"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s157-$TAG-verdetto.out"
if [ -e "$VERD" ]; then
  echo "verdetto $VERD ESISTE — tentativo nuovo = TAG nuovo (az.rev. S-131 #5)" >&2
  exit 7
fi
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
RC="$OUT/$TAG.rc"

if ! grep -qi "s-157\|s157" "$LOCK" 2>/dev/null; then
  echo "lock di finestra assente o ALTRUI — STOP" | tee -a "$VERD"; echo 9 > "$RC"; exit 9
fi
"$QUIESCE" "$OUT/quiesce-$TAG.rc" > "$OUT/quiesce-$TAG.log" 2>&1
QRC=$(cat "$OUT/quiesce-$TAG.rc" 2>/dev/null || echo MANCANTE)
if [ "$QRC" != 0 ]; then
  echo "quiescenza FALLITA (rc=$QRC) — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1
fi

[ "$(shasum -a 256 "$A" | cut -c1-8)" = "42efea3e" ] || { echo "A != gemello 42efea3e (ricostruito tree s156) — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$BB" | cut -c1-8)" = "$BEXP" ]   || { echo "B != $BEXP — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }

ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }

src_of() { case "$1" in missload|hostargs|backtrace) echo "$H";; objalloc|objchurn|objmap|objdatains|objallocni|objdropdef) echo "$M";; *) echo "$M97";; esac; }
php_of() { case "$1" in missload) echo "m-missload.php";; hostargs) echo "m-hostargs.php";; backtrace) echo "m-backtrace24.php";; *) echo "$1.php";; esac; }
n_fixed() { case "$1" in missload) echo 10000000;; hostargs) echo 10000000;; backtrace) echo 2400000;; arith) echo 150000000;; prop) echo 90000000;; calls) echo 60000000;; str) echo 28000000;; arr) echo 30000000;; re) echo 12000000;; *) echo "";; esac; }
CATS_JUDGE="missload"
CATS_GUARD="hostargs backtrace objdropdef objchurn objdatains objalloc objallocni objmap arith prop calls str arr re"

{
echo "== s157-ab-al1 $TAG: A=42efea3e(gemello RICOSTRUITO tree s156) B=$BEXP R=$R ordine ALTERNATO; giudice=missload N=10000000 dichiarato; soglia=max(4, drop-1); UB-alloc falsificabile=20.7 (3 alloc/miss x miheap 6.9); guardie hostargs(N=10M) + backtrace24(N=2,4M) + obj* + sei, comparatore STRETTO; quiescenza rc=$QRC; rc autoritativo = ab-out/$TAG.rc =="
for C in $CATS_JUDGE $CATS_GUARD; do
  D=$(src_of "$C"); P=$(php_of "$C")
  N=$(n_fixed "$C")
  [ -n "$N" ] || N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$D/$P")
  "$A"  "$D/$P" > "$OUT/$TAG-$C-A.out" 2>&1
  "$BB" "$D/$P" > "$OUT/$TAG-$C-B.out" 2>&1
  if ! diff -q "$OUT/$TAG-$C-A.out" "$OUT/$TAG-$C-B.out" > /dev/null; then
    echo "$C: output DIVERGE — STOP LEVA"; echo 2 > "$RC"; exit 2
  fi
  FA=$(floor3 "$A" "$D/empty.php"); FB=$(floor3 "$BB" "$D/empty.php")
  echo "cat=$C N=$N floor_A=$FA floor_B=$FB"
  for i in $(seq 1 "$R"); do
    if [ $((i % 2)) -eq 1 ]; then
      TA=$(ucpu "$A" "$D/$P"); TB=$(ucpu "$BB" "$D/$P"); ord=AB
    else
      TB=$(ucpu "$BB" "$D/$P"); TA=$(ucpu "$A" "$D/$P"); ord=BA
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$C" "$N" "$i" "$TA" "$TB" "$FA" "$FB" "$ord" >> "$TSV"
    echo "  coppia$i [$ord]: rawA=$TA rawB=$TB"
  done
done
python3 - "$TSV" "$R" "$DSM" <<'PY'
import sys
tsv, R = sys.argv[1], int(sys.argv[2])
dsmoke = {}
if len(sys.argv) > 3 and sys.argv[3]: dsmoke["missload"] = float(sys.argv[3])
UB = 20.7            # criterio al1 p.3: 3 alloc/miss x miheap 6.9
rows = {}
for l in open(tsv):
    t = l.rstrip("\n").split("\t")
    rows.setdefault(t[0], []).append(t)
SL = {"arith":0.94,"prop":0.80,"calls":0.73,"str":2.89,"arr":2.49,"re":4.46}
BAND = {"objalloc":13.3,"objmap":3.3,"objallocni":10.0,"objchurn":6.7}
def med(v):
    s = sorted(v); n = len(s)
    return s[n//2] if n % 2 else (s[n//2-1]+s[n//2])/2
def trange(v):
    if len(v) < 4:
        return max(v) - min(v)
    m = med(v)
    w = sorted(v, key=lambda x: (abs(x - m), x))[:-1]
    return max(w) - min(w)
verdict_rc = 0
JUDGES = ("missload",)
for cat, rs in rows.items():
    n = float(rs[0][1])
    na = [(float(t[3])-float(t[5]))/n*1e9 for t in rs]
    nb = [(float(t[4])-float(t[6]))/n*1e9 for t in rs]
    ma, mb = med(na), med(nb)
    delta = ma - mb                      # + = B (leva) piu' veloce
    ra, rb = trange(na), trange(nb)
    noise = max(ra, rb)
    if cat in JUDGES:
        thr = max(4.0, noise)
        ok = delta >= thr
        print(f"GIUDICE {cat}: A={ma:.1f} B={mb:.1f} ns/iter D={delta:+.1f} soglia={thr:.1f} (rumore drop-1: A'={ra:.1f} B'={rb:.1f}) -> {'SOPRA SOGLIA' if ok else 'SOTTO SOGLIA'}")
        if cat in dsmoke:
            band = max(4.0, noise)
            dd = abs(dsmoke[cat] - delta)
            print(f"riconciliazione {cat}: D_smoke={dsmoke[cat]:+.1f} D_R{R}={delta:+.1f} |diff|={dd:.1f} banda={band:.1f} -> {'FUORI BANDA' if dd > band else 'in banda'}")
        fb = delta > UB + noise
        print(f"riconciliazione UB {cat}: D={delta:+.1f} vs UB={UB:.1f}+rumore={noise:.1f} -> {'FUORI BANDA SOPRA (reperto a verbale, sonda dovuta)' if fb else 'dentro la banda del modello'}")
        if not ok: verdict_rc = 4
    else:
        thr = max(4.0, BAND.get(cat, SL.get(cat, 0.0)), noise)
        reg = delta < -thr
        band = f"SL={SL[cat]}" if cat in SL else (f"banda fondata={BAND[cat]}" if cat in BAND else "max(4, drop-1)")
        print(f"guardia {cat}: A={ma:.1f} B={mb:.1f} D={delta:+.1f} soglia_reg={-thr:.1f} [{band}; rumore drop-1 A'={ra:.1f} B'={rb:.1f}] -> {'REGRESSIONE' if reg else 'ok'}")
        if reg: verdict_rc = 5
print("ESITO: " + {0:"SOPRA SOGLIA (giudice ok, guardie ok)",4:"SOTTO SOGLIA (giudice)",5:"GUARDIA MORDE"}[verdict_rc])
sys.exit(verdict_rc)
PY
prc=$?
echo "$prc" > "$RC"
exit "$prc"
} >> "$VERD" 2>&1
