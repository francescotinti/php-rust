#!/bin/bash
# s136-ab.sh <BPATH> <BEXP8> <TAG> <R> [DSMOKE_OBJDATAINS] — LEVA FD1 fast-path
# dim-write su prop (criterio s136-criterio-dimwrite.md, commit PRIMA di ogni run).
# COPIA DICHIARATA di wp135-harness/s135-ab.sh (collaudo: scripts/copia-gate.sh
# + manifest s136-ab-copia.diff) coi SOLI adattamenti del criterio p.3-7:
#  (a) A = stash pin s135 (phpr-s135, 6518a1e1);
#  (b) UN GIUDICE: objdatains; soglia = max(4, rumore drop-1 del run,
#      SPREAD-BATCH objdatains s135-submicro: 13.3);
#  (c) guardie micro-ORM con bande = spread s135-submicro phpr:
#      objalloc 13.3 · objmap 3.3 · objchurn 6.7 · objallocni 10.0;
#  (d) UB FALSIFICABILE = 69.6 (walk_driver 37.2 + resolve 6.7 + guardia 11.3
#      + prop_step_altro 14.4, prezzi MISURATI da s136-tempo-verdetto.out):
#      D > UB+banda(13.3)=82.9 = FUORI BANDA dichiarato a verbale;
#      riconciliazione smoke-R5 con banda = spread-batch giudice (13.3);
#  (e) churn dichiarato: il tree B porta ANCHE az.rev. S-135 #3 (contatore
#      AP1_BUSY su ramo dichiarato irraggiungibile — codice freddo).
# Tutto il resto INVARIATO: coppie ALTERNATE, user CPU /usr/bin/time -p,
# floor med3 per binario, guardie SOLO-REGRESSIONE, trange deterministico,
# verdetto = FILE NUOVO per tentativo. rc autoritativo = SOLO ab-out/<TAG>.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="$H/.."
M="$REPO/wp127-harness/micro-orm"
M97="$REPO/wp123-harness/scaled"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s135"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
BB="${1:?BPATH}"; BEXP="${2:?BEXP8}"; TAG="${3:?TAG}"; R="${4:?R}"; DSM="${5:-}"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s136-$TAG-verdetto.out"
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

[ "$(shasum -a 256 "$A" | cut -c1-8)" = "6518a1e1" ] || { echo "A != pin s135 — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$BB" | cut -c1-8)" = "$BEXP" ]   || { echo "B != $BEXP — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }

ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }

src_of() { case "$1" in objalloc|objchurn|objmap|objdatains|objallocni) echo "$M";; *) echo "$M97";; esac; }
n_fixed() { case "$1" in arith) echo 150000000;; prop) echo 90000000;; calls) echo 60000000;; str) echo 28000000;; arr) echo 30000000;; re) echo 12000000;; *) echo "";; esac; }
CATS_JUDGE="objdatains"
CATS_GUARD="objalloc objmap objchurn objallocni arith prop calls str arr re"

{
echo "== s136-ab $TAG: A=6518a1e1(pin s135) B=$BEXP R=$R ordine ALTERNATO; giudice=objdatains; soglia=max(4, drop-1, spread-batch s135 13.3); UB falsificabile=69.6 (+banda 13.3 => fuori banda sopra 82.9); guardie fondate objalloc 13.3/objmap 3.3/objchurn 6.7/objallocni 10.0; quiescenza rc=$QRC (file: ab-out/quiesce-$TAG.rc); rc autoritativo = ab-out/$TAG.rc =="
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
if len(sys.argv) > 3 and sys.argv[3]: dsmoke["objdatains"] = float(sys.argv[3])
UB = 69.6            # criterio p.3: walk_driver 37.2 + resolve 6.7 +
                     # guardia 11.3 + prop_step_altro 14.4 (prezzi MISURATI,
                     # s136-tempo-verdetto.out; tetto per costruzione)
SPREAD_BATCH = {"objdatains": 13.3}
                     # criterio p.5: spread s135-submicro objdatains 0.04 s @3e6
rows = {}
for l in open(tsv):
    t = l.rstrip("\n").split("\t")
    rows.setdefault(t[0], []).append(t)
# SL storiche (max s123/s125, committate) per le sei; bande micro-ORM =
# spread s135-submicro phpr (criterio p.7).
SL = {"arith":0.94,"prop":0.80,"calls":0.73,"str":2.89,"arr":2.49,"re":4.46}
BAND = {"objalloc":13.3,"objmap":3.3,"objchurn":6.7,"objallocni":10.0}
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
JUDGES = ("objdatains",)
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
        print(f"GIUDICE {cat}: A={ma:.1f} B={mb:.1f} ns/iter D={delta:+.1f} soglia={thr:.1f} (rumore drop-1: A'={ra:.1f} B'={rb:.1f}; spread-batch s135={SPREAD_BATCH[cat]}) -> {'SOPRA SOGLIA' if ok else 'SOTTO SOGLIA'}")
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
