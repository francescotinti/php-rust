#!/bin/bash
# s140-ab-hc1.sh <BPATH> <BEXP8> <TAG> <R> [SMOKE=0|1] [BANDA] [DSM] —
# LEVA HC1 hint-check senza clone (criterio s140-criterio-hc1.md, commit
# PRIMA di ogni run). COPIA DICHIARATA di wp138-harness/s138-ab-rmw.sh coi
# SOLI adattamenti:
#  (a) A = stash pin s138 (phpr-s138, fa17dabd); B = candidato HC1;
#  (b) UN GIUDICE: m-hintcall (wp140-harness, N=3e6 dal sorgente);
#      soglia = max(4, rumore drop-1 del run, BANDA arg6 = banda-layout
#      propria del giudice da s140-banda-verdetto.out);
#  (c) SMOKE=1: SOLO il giudice, NESSUNA guardia;
#  (d) guardie R=5 SOLO-REGRESSIONE: objdatains (13.3) + sei micro (SL) +
#      m-dimrmw/m-diminc (banda 5.0, conferma post-pin S-138);
#  (e) UB: D atteso 10-40 (criterio p.4, canale RECV non prezzato in A/B
#      proprio); D > 80 = FUORI MODELLO (sonda dovuta prima di promuovere).
# Tutto il resto INVARIATO. rc autoritativo = SOLO ab-out/<TAG>.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="$H/.."
M="$REPO/wp127-harness/micro-orm"
M97="$REPO/wp123-harness/scaled"
M138="$REPO/wp138-harness"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s138"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
BB="${1:?BPATH}"; BEXP="${2:?BEXP8}"; TAG="${3:?TAG}"; R="${4:?R}"
SMOKE="${5:-0}"; BANDA="${6:-0}"; DSM="${7:-}"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s140-$TAG-verdetto.out"
if [ -e "$VERD" ]; then
  echo "verdetto $VERD ESISTE — tentativo nuovo = TAG nuovo" >&2
  exit 7
fi
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
RC="$OUT/$TAG.rc"

"$QUIESCE" "$OUT/quiesce-$TAG.rc" > "$OUT/quiesce-$TAG.log" 2>&1
QRC=$(cat "$OUT/quiesce-$TAG.rc" 2>/dev/null || echo MANCANTE)
if [ "$QRC" != 0 ]; then
  echo "quiescenza FALLITA (rc=$QRC) — tentativo NULLO (criterio p.6)" | tee -a "$VERD"; echo 1 > "$RC"; exit 1
fi

[ "$(shasum -a 256 "$A" | cut -c1-8)" = "fa17dabd" ] || { echo "A != pin s138 — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$BB" | cut -c1-8)" = "$BEXP" ]   || { echo "B != $BEXP — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }

ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }

src_of() { case "$1" in m-hintcall) echo "$H";; m-dimrmw|m-diminc) echo "$M138";; objalloc|objchurn|objmap|objdatains|objallocni) echo "$M";; *) echo "$M97";; esac; }
n_fixed() { case "$1" in arith) echo 150000000;; prop) echo 90000000;; calls) echo 60000000;; str) echo 28000000;; arr) echo 30000000;; re) echo 12000000;; *) echo "";; esac; }
CATS_JUDGE="m-hintcall"
if [ "$SMOKE" = 1 ]; then
  CATS_GUARD=""
else
  CATS_GUARD="objdatains m-dimrmw m-diminc arith prop calls str arr re"
fi

{
echo "== s140-ab-hc1 $TAG: A=fa17dabd(pin s138) B=$BEXP R=$R SMOKE=$SMOKE ordine ALTERNATO; giudice=m-hintcall; soglia=max(4, drop-1, banda-layout $BANDA); UB: D>80 FUORI MODELLO; guardie $([ "$SMOKE" = 1 ] && echo NESSUNA-allo-smoke || echo "objdatains+m-dimrmw+m-diminc+sei micro, solo a R=$R"); quiescenza rc=$QRC; rc autoritativo = ab-out/$TAG.rc =="
for C in $CATS_JUDGE $CATS_GUARD; do
  D=$(src_of "$C")
  N=$(n_fixed "$C")
  [ -n "$N" ] || N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$D/$C.php")
  "$A"  "$D/$C.php" > "$OUT/$TAG-$C-A.out" 2>&1
  "$BB" "$D/$C.php" > "$OUT/$TAG-$C-B.out" 2>&1
  if ! diff -q "$OUT/$TAG-$C-A.out" "$OUT/$TAG-$C-B.out" > /dev/null; then
    echo "$C: output DIVERGE — STOP LEVA"; echo 2 > "$RC"; exit 2
  fi
  FA=$(floor3 "$A" "$H/empty.php"); FB=$(floor3 "$BB" "$H/empty.php")
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
python3 - "$TSV" "$R" "$BANDA" "$DSM" <<'PY'
import sys
tsv, R, banda = sys.argv[1], int(sys.argv[2]), float(sys.argv[3])
dsmoke = float(sys.argv[4]) if len(sys.argv) > 4 and sys.argv[4] else None
FUORI_MODELLO = 80.0   # criterio p.4
RICONC_BANDA = max(banda, 5.0)
rows = {}
for l in open(tsv):
    t = l.rstrip("\n").split("\t")
    rows.setdefault(t[0], []).append(t)
SL = {"arith":0.94,"prop":0.80,"calls":0.73,"str":2.89,"arr":2.49,"re":4.46}
BAND = {"objdatains":13.3,"m-dimrmw":5.0,"m-diminc":5.0}
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
JUDGES = ("m-hintcall",)
for cat in [c for c in JUDGES if c in rows] + [c for c in rows if c not in JUDGES]:
    rs = rows[cat]
    n = float(rs[0][1])
    na = [(float(t[3])-float(t[5]))/n*1e9 for t in rs]
    nb = [(float(t[4])-float(t[6]))/n*1e9 for t in rs]
    ma, mb = med(na), med(nb)
    delta = ma - mb                      # + = B (leva) piu' veloce
    if cat in JUDGES:
        ra, rb = trange(na), trange(nb)
        noise = max(ra, rb)
        thr = max(4.0, noise, banda)
        ok = delta >= thr
        print(f"GIUDICE {cat}: A={ma:.1f} B={mb:.1f} ns/iter D={delta:+.1f} soglia={thr:.1f} (rumore drop-1: A'={ra:.1f} B'={rb:.1f}; banda-layout={banda}) -> {'SOPRA SOGLIA' if ok else 'SOTTO SOGLIA'}")
        if dsmoke is not None:
            dd = abs(dsmoke - delta)
            print(f"riconciliazione {cat}: D_smoke={dsmoke:+.1f} D_R{R}={delta:+.1f} |diff|={dd:.1f} banda={RICONC_BANDA} -> {'FUORI BANDA' if dd > RICONC_BANDA else 'in banda'}")
        fm = delta > FUORI_MODELLO
        print(f"vs modello {cat}: D={delta:+.1f} atteso 10-40 -> {'FUORI MODELLO: sonda dovuta prima di promuovere' if fm else 'dentro il modello dichiarato'}")
        if not ok: verdict_rc = 4
        if fm: verdict_rc = 6
    else:
        ra, rb = trange(na), trange(nb)
        noise = max(ra, rb)
        thr = max(4.0, BAND.get(cat, SL.get(cat, 0.0)), noise)
        reg = delta < -thr
        band = f"SL={SL[cat]}" if cat in SL else f"banda fondata={BAND[cat]}"
        print(f"guardia {cat}: A={ma:.1f} B={mb:.1f} D={delta:+.1f} soglia_reg={-thr:.1f} [{band}; rumore drop-1 A'={ra:.1f} B'={rb:.1f}] -> {'REGRESSIONE' if reg else 'ok'}")
        if reg: verdict_rc = 5
print("ESITO: " + {0:"PROMOSSA (giudice sopra soglia, guardie ok)",4:"NON PROMOSSA (giudice sotto soglia)",5:"GUARDIA MORDE",6:"FUORI MODELLO (sonda dovuta)"}[verdict_rc])
sys.exit(verdict_rc)
PY
prc=$?
echo "$prc" > "$RC"
exit "$prc"
} >> "$VERD" 2>&1
