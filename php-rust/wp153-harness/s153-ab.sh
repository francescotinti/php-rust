#!/bin/bash
# s153-ab.sh <BPATH> <BEXP8> <TAG> <R> [DSMOKE_OBJDROPDEF] — LEVA L-TD1
# teardown/sweep borrow unico (criterio s153-criterio-td1.md, commit PRIMA).
# COPIA DICHIARATA di wp136-harness/s136-ab.sh coi SOLI adattamenti:
#  (a) A = stash pin s150 (phpr-s150, cbbe7173);
#  (b) GIUDICE: objdropdef; soglia = max(4, rumore drop-1) — NESSUNA
#      spread-batch fondata per objdropdef (criterio p.3, dichiarato);
#  (c) UB FALSIFICABILE = 17.6 (4 borrow/iter x mock c2_borrow 4.41,
#      verdetto s152-sonda): D > UB+rumore = fuori banda a verbale;
#  (d) COMPANION FIRMATI (segno atteso +, non gate): objchurn, objdatains;
#      guardie SOLO-REGRESSIONE: objalloc 13.3 / objallocni 10.0 / objmap 3.3
#      (spread s135-submicro, da s136-ab.sh committato) + le sei (SL storiche);
#  (e) lock di finestra: procede SOLO se /private/tmp/phpr-measure.lock
#      esiste ed e' MIO (contiene "s153"); altrui => rc 9.
# Tutto il resto INVARIATO: coppie ALTERNATE, user CPU /usr/bin/time -p,
# floor med3 per binario, trange deterministico, verdetto = FILE NUOVO per
# tentativo. rc autoritativo = SOLO ab-out/<TAG>.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="$H/.."
M="$REPO/wp127-harness/micro-orm"
M97="$REPO/wp123-harness/scaled"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s150"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
LOCK=/private/tmp/phpr-measure.lock
BB="${1:?BPATH}"; BEXP="${2:?BEXP8}"; TAG="${3:?TAG}"; R="${4:?R}"; DSM="${5:-}"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s153-$TAG-verdetto.out"
if [ -e "$VERD" ]; then
  echo "verdetto $VERD ESISTE — tentativo nuovo = TAG nuovo (az.rev. S-131 #5)" >&2
  exit 7
fi
TSV="$OUT/$TAG-runs.tsv"; : > "$TSV"
RC="$OUT/$TAG.rc"

if ! grep -q "s153" "$LOCK" 2>/dev/null; then
  echo "lock di finestra assente o ALTRUI — STOP" | tee -a "$VERD"; echo 9 > "$RC"; exit 9
fi
"$QUIESCE" "$OUT/quiesce-$TAG.rc" > "$OUT/quiesce-$TAG.log" 2>&1
QRC=$(cat "$OUT/quiesce-$TAG.rc" 2>/dev/null || echo MANCANTE)
if [ "$QRC" != 0 ]; then
  echo "quiescenza FALLITA (rc=$QRC) — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1
fi

[ "$(shasum -a 256 "$A" | cut -c1-8)" = "cbbe7173" ] || { echo "A != pin s150 — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$BB" | cut -c1-8)" = "$BEXP" ]   || { echo "B != $BEXP — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }

ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }

src_of() { case "$1" in objalloc|objchurn|objmap|objdatains|objallocni|objdropdef) echo "$M";; *) echo "$M97";; esac; }
n_fixed() { case "$1" in arith) echo 150000000;; prop) echo 90000000;; calls) echo 60000000;; str) echo 28000000;; arr) echo 30000000;; re) echo 12000000;; *) echo "";; esac; }
CATS_JUDGE="objdropdef"
CATS_COMPANION="objchurn objdatains"
CATS_GUARD="objalloc objallocni objmap arith prop calls str arr re"

{
echo "== s153-ab $TAG: A=cbbe7173(pin s150) B=$BEXP R=$R ordine ALTERNATO; giudice=objdropdef; soglia=max(4, drop-1); UB falsificabile=17.6 (4 borrow x mock 4.41); companion firmati objchurn/objdatains; guardie objalloc 13.3/objallocni 10.0/objmap 3.3 + sei SL; quiescenza rc=$QRC; rc autoritativo = ab-out/$TAG.rc =="
for C in $CATS_JUDGE $CATS_COMPANION $CATS_GUARD; do
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
if len(sys.argv) > 3 and sys.argv[3]: dsmoke["objdropdef"] = float(sys.argv[3])
UB = 17.6            # criterio p.5: 4 borrow/iter x mock c2_borrow 4.41
                     # (s152-sonda-canali-verdetto.out); tetto per costruzione
rows = {}
for l in open(tsv):
    t = l.rstrip("\n").split("\t")
    rows.setdefault(t[0], []).append(t)
SL = {"arith":0.94,"prop":0.80,"calls":0.73,"str":2.89,"arr":2.49,"re":4.46}
BAND = {"objalloc":13.3,"objmap":3.3,"objallocni":10.0}
COMPANION = ("objchurn","objdatains")
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
JUDGES = ("objdropdef",)
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
    elif cat in COMPANION:
        print(f"companion {cat}: A={ma:.1f} B={mb:.1f} D={delta:+.1f} segno atteso + -> {'CONFERMA' if delta > 0 else 'SEGNO OPPOSTO (a verbale, non gate)'} [rumore drop-1 A'={ra:.1f} B'={rb:.1f}]")
    else:
        thr = max(4.0, BAND.get(cat, SL.get(cat, 0.0)), noise)
        reg = delta < -thr
        band = f"SL={SL[cat]}" if cat in SL else f"banda fondata={BAND[cat]}"
        print(f"guardia {cat}: A={ma:.1f} B={mb:.1f} D={delta:+.1f} soglia_reg={-thr:.1f} [{band}; rumore drop-1 A'={ra:.1f} B'={rb:.1f}] -> {'REGRESSIONE' if reg else 'ok'}")
        if reg: verdict_rc = 5
print("ESITO: " + {0:"SOPRA SOGLIA (giudice ok, guardie ok)",4:"SOTTO SOGLIA (giudice)",5:"GUARDIA MORDE"}[verdict_rc])
sys.exit(verdict_rc)
PY
prc=$?
echo "$prc" > "$RC"
exit "$prc"
} >> "$VERD" 2>&1
