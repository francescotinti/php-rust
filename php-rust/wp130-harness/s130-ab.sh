#!/bin/bash
# s130-ab.sh <BPATH> <BEXP8> <TAG> <R> — RIENTRO F4 (criterio in
# s130-criterio-f4.md, commit PRIMA di ogni run). COPIA DICHIARATA di
# wp129-harness/s129-ab.sh con le SOLE emende 3-4 del criterio:
#  (a) rumore giudice = trimmed drop-1 SIMMETRICO su A e B (R>=4; R<=3 range
#      pieno), rumore = max(rangeA', rangeB');
#  (b) bande micro-ORM FONDATE (range drop-1 delle 7 gambe A committate S-129):
#      objmap 6.7 / objalloc 6.7 / objchurn 13.3 — niente default 4.
# Tutto il resto INVARIATO: A = pin s127b (stash), giudice objdatains, coppie
# ALTERNATE (dispari A->B, pari B->A), user CPU /usr/bin/time -p, floor med3
# per binario, guardie SOLO-REGRESSIONE, sei storiche su SL committate.
# rc autoritativo = SOLO ab-out/<TAG>.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="$H/.."
M="$REPO/wp127-harness/micro-orm"
M97="$REPO/wp123-harness/scaled"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s127b"
BB="${1:?BPATH}"; BEXP="${2:?BEXP8}"; TAG="${3:?TAG}"; R="${4:?R}"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s130-$TAG-verdetto.out"
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
RC="$OUT/$TAG.rc"

[ "$(shasum -a 256 "$A" | cut -c1-8)" = "ccb63dca" ] || { echo "A != pin s127b — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$BB" | cut -c1-8)" = "$BEXP" ]   || { echo "B != $BEXP — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }

ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }

src_of() { case "$1" in objalloc|objchurn|objmap|objdatains) echo "$M";; *) echo "$M97";; esac; }
n_fixed() { case "$1" in arith) echo 150000000;; prop) echo 90000000;; calls) echo 60000000;; str) echo 28000000;; arr) echo 30000000;; re) echo 12000000;; *) echo "";; esac; }
CATS_JUDGE="objdatains"
CATS_GUARD="objalloc objchurn objmap arith prop calls str arr re"

{
echo "== s130-ab $TAG: A=ccb63dca(pin s127b) B=$BEXP R=$R ordine ALTERNATO; giudice=objdatains; rumore trimmed drop-1 simmetrico; bande fondate objmap 6.7/objalloc 6.7/objchurn 13.3; rc autoritativo = ab-out/$TAG.rc =="
for C in $CATS_JUDGE $CATS_GUARD; do
  D=$(src_of "$C")
  N=$(n_fixed "$C")
  [ -n "$N" ] || N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$D/$C.php")
  "$A"  "$D/$C.php" > "$OUT/$TAG-$C-A.out" 2>&1
  "$BB" "$D/$C.php" > "$OUT/$TAG-$C-B.out" 2>&1
  if ! diff -q "$OUT/$TAG-$C-A.out" "$OUT/$TAG-$C-B.out" > /dev/null; then
    echo "$C: output DIVERGE — STOP LEVA"; echo 2 > "$RC"; exit 2
  fi
  FA=$(floor3 "$A" "$D/empty.php"); FB=$(floor3 "$BB" "$D/empty.php")
  echo "cat=$C N=$N floor_A=$FA floor_B=$FB"
  for i in $(seq 1 "$R"); do
    if [ $((i % 2)) -eq 1 ]; then
      TA=$(ucpu "$A" "$D/$C.php"); TB=$(ucpu "$BB" "$D/$C.php"); ord=AB
    else
      TB=$(ucpu "$BB" "$D/$C.php"); TA=$(ucpu "$A" "$D/$C.php"); ord=BA
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$C" "$N" "$i" "$TA" "$TB" "$FA" "$FB" "$ord" >> "$TSV"
    echo "  coppia$i [$ord]: rawA=$TA rawB=$TB"
  done
done
python3 - "$TSV" "$R" <<'PY'
import sys
tsv, R = sys.argv[1], int(sys.argv[2])
rows = {}
for l in open(tsv):
    t = l.rstrip("\n").split("\t")
    rows.setdefault(t[0], []).append(t)
# SL storiche (max s123/s125, committate) per le sei; bande micro-ORM FONDATE
# (criterio s130 p.4: range drop-1 delle 7 gambe A committate S-129).
SL = {"arith":0.94,"prop":0.80,"calls":0.73,"str":2.89,"arr":2.49,"re":4.46}
BAND = {"objmap":6.7,"objalloc":6.7,"objchurn":13.3}
def med(v):
    s = sorted(v); n = len(s)
    return s[n//2] if n % 2 else (s[n//2-1]+s[n//2])/2
def trange(v):
    # emenda (a): con R>=4 scarta il singolo campione piu' lontano dalla
    # mediana della serie, range dei restanti; con R<=3 range pieno
    if len(v) < 4:
        return max(v) - min(v)
    m = med(v)
    w = sorted(v, key=lambda x: abs(x - m))[:-1]
    return max(w) - min(w)
verdict_rc = 0
for cat, rs in rows.items():
    n = float(rs[0][1])
    na = [(float(t[3])-float(t[5]))/n*1e9 for t in rs]
    nb = [(float(t[4])-float(t[6]))/n*1e9 for t in rs]
    ma, mb = med(na), med(nb)
    delta = ma - mb                      # + = B (leva) piu' veloce
    if cat == "objdatains":
        ra, rb = trange(na), trange(nb)
        noise = max(ra, rb)
        thr = max(4.0, noise, SL["prop"])
        ok = delta >= thr
        print(f"GIUDICE {cat}: A={ma:.1f} B={mb:.1f} ns/iter D={delta:+.1f} soglia={thr:.1f} (rumore drop-1: A'={ra:.1f} B'={rb:.1f}) -> {'PROMOSSA' if ok else 'SOTTO SOGLIA'}")
        if not ok: verdict_rc = 4
    else:
        thr = max(4.0, BAND.get(cat, SL.get(cat, 0.0)))
        reg = delta < -thr
        band = f"SL={SL[cat]}" if cat in SL else f"banda fondata={BAND[cat]}"
        print(f"guardia {cat}: A={ma:.1f} B={mb:.1f} D={delta:+.1f} soglia_reg={-thr:.1f} [{band}] -> {'REGRESSIONE' if reg else 'ok'}")
        if reg: verdict_rc = 5
print("ESITO: " + {0:"PROMOSSA (giudice sopra soglia, guardie ok)",4:"NON PROMOSSA (sotto soglia)",5:"GUARDIA MORDE"}[verdict_rc])
sys.exit(verdict_rc)
PY
prc=$?
echo "$prc" > "$RC"
exit "$prc"
} >> "$VERD" 2>&1
