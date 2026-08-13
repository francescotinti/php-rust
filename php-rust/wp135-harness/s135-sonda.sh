#!/bin/bash
# s135-sonda.sh — sonda CONTEGGI s134 vs s133 (criterio s135-criterio-sonda.md;
# predizioni REGISTRATE nel criterio, commit PRIMA del run). Due binari EMENDATI
# dai worktree (patch s135-probes-{s133,s134}.patch agli atti), target separati
# — MAI pinnabili. Conteggi deterministici e insensibili al rumore: niente
# quiescenza (dichiarato). rc autoritativo = SOLO sonda-out/sonda.rc.
# Parita' stdout vs oracle su ogni run.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp127-harness/micro-orm"
PA="/Volumes/Extreme Pro/Claude/phpr-s135-target-s133/release/phpr"
PB="/Volumes/Extreme Pro/Claude/phpr-s135-target-s134/release/phpr"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
OUT="$H/sonda-out"; mkdir -p "$OUT"
VERD="$H/s135-sonda-verdetto.out"
RC="$OUT/sonda.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = file nuovo" >&2; exit 7; }
[ -x "$PA" ] || { echo "manca binario gamba A (s133)" >&2; echo 1 > "$RC"; exit 1; }
[ -x "$PB" ] || { echo "manca binario gamba B (s134)" >&2; echo 1 > "$RC"; exit 1; }

{
echo "== s135 sonda conteggi s134 vs s133 (12 seg; predizioni nel criterio s135-criterio-sonda.md) =="
echo "gambaA(s133)=$(shasum -a 256 "$PA" | cut -c1-16) gambaB(s134)=$(shasum -a 256 "$PB" | cut -c1-16) (build emendate, MAI pinnabili)"
FAIL=0
for C in objalloc objdatains; do
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$C.php")
  EXP=$("$ORACLE" "$M/$C.php")
  for G in A B; do
    BIN="$PA"; [ "$G" = B ] && BIN="$PB"
    for r in 1 2; do
      GOT=$(PHPR_S135_PROBES="$OUT/$C-$G-r$r.counts" "$BIN" "$M/$C.php")
      [ "$GOT" = "$EXP" ] || { echo "$C $G r$r: PARITA' STDOUT ROTTA"; FAIL=1; }
    done
    C1=$(sed 's/^s135probes pid=[0-9]* //' "$OUT/$C-$G-r1.counts")
    C2=$(sed 's/^s135probes pid=[0-9]* //' "$OUT/$C-$G-r2.counts")
    [ "$C1" = "$C2" ] || { echo "$C $G: CONTEGGI NON DETERMINISTICI tra R"; FAIL=1; }
    eval "$(printf '%s\n' "$C1" | sed 's/c\([0-9][0-9]*\)=/K\1=/g')"
    python3 - "$C" "$G" "$N" "$K0" "$K1" "$K2" "$K3" "$K4" "$K5" "$K6" "$K7" "$K8" "$K9" "$K10" "$K11" <<'PY'
import sys
c, g = sys.argv[1], sys.argv[2]
n, *k = map(int, sys.argv[3:])
lbl = ["entrate", "resolve", "NPhit", "NPfill", "magic", "hook", "enum", "asym", "ro", "depr", "coerce", "plainhit"]
print(f"{c} {g}: N={n} " + " ".join(f"{l}={v} ({v/n:.4f}/i)" for l, v in zip(lbl, k)))
PY
  done
done
if [ "$FAIL" = 0 ]; then echo "SONDA ACQUISITA (parita' ok, conteggi deterministici)"; echo 0 > "$RC"; else echo 1 > "$RC"; fi
} > "$VERD" 2>&1
exit "$(cat "$RC")"
