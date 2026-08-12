#!/bin/bash
# s132-ab.sh <BPATH> <BEXP8> <TAG> <R> [DSMOKE] — LEVA L-LO1 (criterio in
# s132-criterio-lookuponce.md, commit PRIMA di ogni run). COPIA DICHIARATA di
# wp131-harness/s131-ab.sh coi SOLI adattamenti del criterio p.3-6:
#  (a) A = stash pin s131 (phpr-s131, ff66cb84);
#  (b) soglia giudice az.rev. S-131 #1: max(4, rumore drop-1, SPREAD STORICO
#      TRA BATCH sul pin s131 = 10.0, SL prop 0.80);
#  (c) bande micro-ORM FONDATE sulle 7 gambe B committate S-131 (stesso
#      binario che qui e' A): objalloc 13.3 / objchurn 6.7 / objmap 10.0;
#  (d) trange con tie-break DETERMINISTICO (az.rev. #4: chiave (|x-m|, x));
#  (e) verdetto = FILE NUOVO per tentativo (az.rev. #5: rifiuto se esiste);
#  (f) riconciliazione az.rev. #2: con DSMOKE dato, il verdetto DICHIARA
#      |D_smoke-D_R| e D vs UB=30 (dichiarazioni, non gate).
# Tutto il resto INVARIATO: giudice objdatains, coppie ALTERNATE, user CPU
# /usr/bin/time -p, floor med3 per binario, guardie SOLO-REGRESSIONE.
# rc autoritativo = SOLO ab-out/<TAG>.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="$H/.."
M="$REPO/wp127-harness/micro-orm"
M97="$REPO/wp123-harness/scaled"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s131"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
BB="${1:?BPATH}"; BEXP="${2:?BEXP8}"; TAG="${3:?TAG}"; R="${4:?R}"; DSMOKE="${5:-}"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s132-$TAG-verdetto.out"
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

[ "$(shasum -a 256 "$A" | cut -c1-8)" = "ff66cb84" ] || { echo "A != pin s131 — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
[ "$(shasum -a 256 "$BB" | cut -c1-8)" = "$BEXP" ]   || { echo "B != $BEXP — STOP" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }

ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$2"); b=$(ucpu "$1" "$2"); c=$(ucpu "$1" "$2"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }

src_of() { case "$1" in objalloc|objchurn|objmap|objdatains) echo "$M";; *) echo "$M97";; esac; }
n_fixed() { case "$1" in arith) echo 150000000;; prop) echo 90000000;; calls) echo 60000000;; str) echo 28000000;; arr) echo 30000000;; re) echo 12000000;; *) echo "";; esac; }
CATS_JUDGE="objdatains"
CATS_GUARD="objalloc objchurn objmap arith prop calls str arr re"

{
echo "== s132-ab $TAG: A=ff66cb84(pin s131) B=$BEXP R=$R ordine ALTERNATO; giudice=objdatains; soglia=max(4, drop-1, spread-batch 10.0, SL prop); bande fondate objalloc 13.3/objchurn 6.7/objmap 10.0; quiescenza rc=$QRC (file: ab-out/quiesce-$TAG.rc); rc autoritativo = ab-out/$TAG.rc =="
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
python3 - "$TSV" "$R" "$DSMOKE" <<'PY'
import sys
tsv, R = sys.argv[1], int(sys.argv[2])
dsmoke = float(sys.argv[3]) if len(sys.argv) > 3 and sys.argv[3] else None
UB = 30.0            # criterio p.5: ~3 lookup x 10.4 dentro gli 81.9 non-resolve
SPREAD_BATCH = 10.0  # criterio p.3: batch ff66cb84 committati (1223.3/1230.0/1220.0)
rows = {}
for l in open(tsv):
    t = l.rstrip("\n").split("\t")
    rows.setdefault(t[0], []).append(t)
# SL storiche (max s123/s125, committate) per le sei; bande micro-ORM FONDATE
# sul pin s131 (criterio p.4: range drop-1 delle 7 gambe B committate S-131).
SL = {"arith":0.94,"prop":0.80,"calls":0.73,"str":2.89,"arr":2.49,"re":4.46}
BAND = {"objmap":10.0,"objalloc":13.3,"objchurn":6.7}
def med(v):
    s = sorted(v); n = len(s)
    return s[n//2] if n % 2 else (s[n//2-1]+s[n//2])/2
def trange(v):
    # az.rev. S-131 #4: tie-break deterministico — chiave (|x-m|, x), a parita'
    # di distanza si scarta il valore MAGGIORE (funzione del multiset).
    if len(v) < 4:
        return max(v) - min(v)
    m = med(v)
    w = sorted(v, key=lambda x: (abs(x - m), x))[:-1]
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
        thr = max(4.0, noise, SPREAD_BATCH, SL["prop"])
        ok = delta >= thr
        print(f"GIUDICE {cat}: A={ma:.1f} B={mb:.1f} ns/iter D={delta:+.1f} soglia={thr:.1f} (rumore drop-1: A'={ra:.1f} B'={rb:.1f}; spread-batch storico={SPREAD_BATCH}) -> {'PROMOSSA' if ok else 'SOTTO SOGLIA'}")
        # az.rev. S-131 #2: riconciliazione DICHIARATA (non gate)
        if dsmoke is not None:
            dd = abs(dsmoke - delta)
            print(f"riconciliazione: D_smoke={dsmoke:+.1f} D_R{R}={delta:+.1f} |diff|={dd:.1f} -> {'FUORI BANDA (>soglia)' if dd > thr else 'in banda'}")
        print(f"riconciliazione UB: D={delta:+.1f} vs UB modello={UB:.0f} -> {'FUORI BANDA (D>UB)' if delta > UB else 'dentro UB'}")
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
