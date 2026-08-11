#!/bin/bash
# s128-ab.sh <BPATH> <BEXP8> <TAG> <R> — A/B L-OL1-F2 (criterio in
# s128-admission-lettura.md c9f5950, PRIMA del codice della forma): A = pin
# s127b (stash), B = leva keys-scratch. Giudice objdatains R=<R> coppie
# ALTERNATE (dispari A->B, pari B->A), user CPU /usr/bin/time -p, floor med3
# per binario. Guardie SOLO-REGRESSIONE: objalloc objchurn objmap + sei
# storiche. Bande: categorie micro-ORM SENZA banda storica = «default 4 ns»
# DICHIARATO (az.rev. S-127 #4: niente SL prestate); le sei storiche usano le
# SL committate (max s123/s125). rc autoritativo = SOLO ab-out/<TAG>.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="$H/.."
M="$REPO/wp127-harness/micro-orm"
M97="$REPO/wp123-harness/scaled"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s127b"
BB="${1:?BPATH}"; BEXP="${2:?BEXP8}"; TAG="${3:?TAG}"; R="${4:?R}"
OUT="$H/ab-out"; mkdir -p "$OUT"
VERD="$H/s128-$TAG-verdetto.out"
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
echo "== s128-ab $TAG: A=ccb63dca(pin s127b) B=$BEXP R=$R ordine ALTERNATO; giudice=objdatains; guardie solo-regressione; rc autoritativo = ab-out/$TAG.rc =="
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
# SL storiche (max s123/s125, committate) SOLO per le sei; micro-ORM senza
# banda storica: SL assente => domina il pavimento 4 ns («default 4 ns»
# DICHIARATO, az.rev. S-127 #4). Giudice: soglia max(4, rumoreB, SL prop).
SL = {"arith":0.94,"prop":0.80,"calls":0.73,"str":2.89,"arr":2.49,"re":4.46}
def med(v):
    s = sorted(v); n = len(s)
    return s[n//2] if n % 2 else (s[n//2-1]+s[n//2])/2
verdict_rc = 0
for cat, rs in rows.items():
    n = float(rs[0][1])
    na = [(float(t[3])-float(t[5]))/n*1e9 for t in rs]
    nb = [(float(t[4])-float(t[6]))/n*1e9 for t in rs]
    ma, mb = med(na), med(nb)
    delta = ma - mb                      # + = B (leva) più veloce
    noise = max(nb) - min(nb)
    if cat == "objdatains":
        thr = max(4.0, noise, SL["prop"])
        ok = delta >= thr
        print(f"GIUDICE {cat}: A={ma:.1f} B={mb:.1f} ns/iter D={delta:+.1f} soglia={thr:.1f} (rumoreB={noise:.1f}) -> {'PROMOSSA' if ok else 'SOTTO SOGLIA'}")
        if not ok: verdict_rc = 4
    else:
        thr = max(4.0, SL.get(cat, 0.0))
        reg = delta < -thr
        band = f"SL={SL[cat]}" if cat in SL else "default 4 ns (nessuna banda storica)"
        print(f"guardia {cat}: A={ma:.1f} B={mb:.1f} D={delta:+.1f} soglia_reg={-thr:.1f} [{band}] -> {'REGRESSIONE' if reg else 'ok'}")
        if reg: verdict_rc = 5
print("ESITO: " + {0:"PROMOSSA (giudice sopra soglia, guardie ok)",4:"NON PROMOSSA (sotto soglia)",5:"GUARDIA MORDE"}[verdict_rc])
sys.exit(verdict_rc)
PY
prc=$?
echo "$prc" > "$RC"
exit "$prc"
} >> "$VERD" 2>&1
