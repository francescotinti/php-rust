#!/bin/bash
# s135-ab.sh <BPATH> <BEXP8> <TAG> <R> [DSMOKE_OBJMAP] — LEVA AP1 fast-path
# (criterio s135-criterio-ap1.md, commit PRIMA di ogni run).
# COPIA DICHIARATA di wp134-harness/s134-ab.sh (collaudo: scripts/copia-gate.sh
# + manifest s135-ab-copia.diff) coi SOLI adattamenti del criterio p.3-6:
#  (a) A = stash pin s134 (phpr-s134, 61896da1);
#  (b) UN GIUDICE: objmap; soglia = max(4, rumore drop-1 del run,
#      SPREAD-BATCH objmap s134-submicro: 10.0);
#  (c) guardie micro-ORM con bande = spread s134-submicro phpr:
#      objalloc 3.3 · objdatains 30.0 · objchurn 16.7;
#  (d) UB FALSIFICABILE = 47.7 (walk-plumbing 38.4 + residuo-arm 9.3,
#      prezzi MISURATI dal modello tempo s135-tempo-verdetto.out):
#      D > UB+banda(10.0)=57.7 = FUORI BANDA dichiarato a verbale;
#      riconciliazione smoke-R5 con banda = spread-batch giudice (10.0);
#  (e) churn dichiarato: nessuno (tree B = leva sola sopra il pin s134).
# Tutto il resto INVARIATO: coppie ALTERNATE, user CPU /usr/bin/time -p,
# floor med3 per binario, guardie SOLO-REGRESSIONE, trange deterministico,
# verdetto = FILE NUOVO per tentativo. rc autoritativo = SOLO ab-out/<TAG>.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="$H/.."
M="$REPO/wp127-harness/micro-orm"
M97="$REPO/wp123-harness/scaled"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s134"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
BB="${1:?BPATH}"; BEXP="${2:?BEXP8}"; TAG="${3:?TAG}"; R="${4:?R}"; DSM="${5:-}"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s135-$TAG-verdetto.out"
if [ -e "$VERD" ]; then
  echo "verdetto $VERD ESISTE — tentativo nuovo = TAG nuovo (az.rev. S-131 #5)" >&2
  exit 7
fi
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
RC="$OUT/$TAG.rc"

"$QUIESCE" "$OUT/quiesce-$TAG.rc" > "$OUT/quiesce-$TAG.log" 2>&1
QRC=$(cat "$OUT/quiesce-$TAG.rc" 2>/dev/null || echo MANCANTE)
if [ "$QRC" != 0 ]; then
  echo "quiescenza FALLITA (rc=$QRC) — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1
fi

[ "$(shasum -a 256 "$A" | cut -c1-8)" = "61896da1" ] || { echo "A != pin s134 — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$BB" | cut -c1-8)" = "$BEXP" ]   || { echo "B != $BEXP — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }

ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }

src_of() { case "$1" in objalloc|objchurn|objmap|objdatains) echo "$M";; *) echo "$M97";; esac; }
n_fixed() { case "$1" in arith) echo 150000000;; prop) echo 90000000;; calls) echo 60000000;; str) echo 28000000;; arr) echo 30000000;; re) echo 12000000;; *) echo "";; esac; }
CATS_JUDGE="objmap"
CATS_GUARD="objalloc objdatains objchurn arith prop calls str arr re"

{
echo "== s135-ab $TAG: A=61896da1(pin s134) B=$BEXP R=$R ordine ALTERNATO; giudice=objmap; soglia=max(4, drop-1, spread-batch s134 10.0); UB falsificabile=47.7 (+banda 10.0 => fuori banda sopra 57.7); guardie fondate objalloc 3.3/objdatains 30.0/objchurn 16.7; quiescenza rc=$QRC (file: ab-out/quiesce-$TAG.rc); rc autoritativo = ab-out/$TAG.rc =="
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
python3 - "$TSV" "$R" "$DSM" <<'PY'
import sys
tsv, R = sys.argv[1], int(sys.argv[2])
dsmoke = {}
if len(sys.argv) > 3 and sys.argv[3]: dsmoke["objmap"] = float(sys.argv[3])
UB = 47.7            # criterio p.4: walk-plumbing 38.4 + residuo-arm 9.3
                     # (prezzi MISURATI, s135-tempo-verdetto.out)
SPREAD_BATCH = {"objmap": 10.0}
                     # criterio p.4: spread s134-submicro objmap 0.03 s @3e6
rows = {}
for l in open(tsv):
    t = l.rstrip("\n").split("\t")
    rows.setdefault(t[0], []).append(t)
# SL storiche (max s123/s125, committate) per le sei; bande micro-ORM =
# spread s134-submicro phpr (criterio p.6).
SL = {"arith":0.94,"prop":0.80,"calls":0.73,"str":2.89,"arr":2.49,"re":4.46}
BAND = {"objalloc":3.3,"objdatains":30.0,"objchurn":16.7}
def med(v):
    s = sorted(v); n = len(s)
    return s[n//2] if n % 2 else (s[n//2-1]+s[n//2])/2
def trange(v):
    # az.rev. S-131 #4: tie-break deterministico — chiave (|x-m|, x).
    if len(v) < 4:
        return max(v) - min(v)
    m = med(v)
    w = sorted(v, key=lambda x: (abs(x - m), x))[:-1]
    return max(w) - min(w)
verdict_rc = 0
JUDGES = ("objmap",)
for cat, rs in rows.items():
    n = float(rs[0][1])
    na = [(float(t[3])-float(t[5]))/n*1e9 for t in rs]
    nb = [(float(t[4])-float(t[6]))/n*1e9 for t in rs]
    ma, mb = med(na), med(nb)
    delta = ma - mb                      # + = B (leva) piu' veloce
    if cat in JUDGES:
        ra, rb = trange(na), trange(nb)
        noise = max(ra, rb)
        thr = max(4.0, noise, SPREAD_BATCH[cat])
        ok = delta >= thr
        print(f"GIUDICE {cat}: A={ma:.1f} B={mb:.1f} ns/iter D={delta:+.1f} soglia={thr:.1f} (rumore drop-1: A'={ra:.1f} B'={rb:.1f}; spread-batch s134={SPREAD_BATCH[cat]}) -> {'SOPRA SOGLIA' if ok else 'SOTTO SOGLIA'}")
        if cat in dsmoke:
            dd = abs(dsmoke[cat] - delta)
            print(f"riconciliazione {cat}: D_smoke={dsmoke[cat]:+.1f} D_R{R}={delta:+.1f} |diff|={dd:.1f} banda={SPREAD_BATCH[cat]} -> {'FUORI BANDA' if dd > SPREAD_BATCH[cat] else 'in banda'}")
        fb = delta > UB + SPREAD_BATCH[cat]
        print(f"riconciliazione UB {cat}: D={delta:+.1f} vs UB={UB:.1f}+banda={SPREAD_BATCH[cat]} -> {'FUORI BANDA SOPRA (reperto a verbale, sonda dovuta)' if fb else 'dentro la banda del modello'}")
        if not ok: verdict_rc = 4
    else:
        # EMENDA criterio p.6 (rev. S-112, dopo ab-ap1): stessa formula del
        # giudice — la soglia include il rumore drop-1 del run stesso.
        ra, rb = trange(na), trange(nb)
        noise = max(ra, rb)
        thr = max(4.0, BAND.get(cat, SL.get(cat, 0.0)), noise)
        reg = delta < -thr
        band = f"SL={SL[cat]}" if cat in SL else f"banda fondata={BAND[cat]}"
        print(f"guardia {cat}: A={ma:.1f} B={mb:.1f} D={delta:+.1f} soglia_reg={-thr:.1f} [{band}; rumore drop-1 A'={ra:.1f} B'={rb:.1f}] -> {'REGRESSIONE' if reg else 'ok'}")
        if reg: verdict_rc = 5
print("ESITO: " + {0:"PROMOSSA (giudice sopra soglia, guardie ok)",4:"NON PROMOSSA (giudice sotto soglia)",5:"GUARDIA MORDE"}[verdict_rc])
sys.exit(verdict_rc)
PY
prc=$?
echo "$prc" > "$RC"
exit "$prc"
} >> "$VERD" 2>&1
