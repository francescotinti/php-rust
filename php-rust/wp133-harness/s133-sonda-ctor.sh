#!/bin/bash
# s133-sonda-ctor.sh — sonda CONTEGGI per-sito del cammino ctor (criterio
# s133-criterio-ctor.md p.1; predizioni REGISTRATE nel criterio). Binario
# EMENDATO dal target separato phpr-ctor-probe-target (patch
# count-probes-ctor.patch col tree, ripristinata a fine sonda). Conteggi
# deterministici e insensibili al rumore: niente quiescenza (dichiarato).
# rc autoritativo = SOLO sonda-out/sonda.rc. Parita' stdout vs oracle.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/../wp127-harness/micro-orm"
PROBE="/Volumes/Extreme Pro/Claude/phpr-ctor-probe-target/release/phpr"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
OUT="$H/sonda-out"; mkdir -p "$OUT"
VERD="$H/s133-sonda-ctor-verdetto.out"
RC="$OUT/sonda.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = file nuovo" >&2; exit 7; }
[ -x "$PROBE" ] || { echo "manca il binario sonda" >&2; echo 1 > "$RC"; exit 1; }

{
echo "== s133 sonda ctor: CONTEGGI per-sito (predizioni: objalloc A=2 B=2 TOT=4 ENTRATE=4 /iter; objdatains TOT=9) =="
echo "probe_bin=$(shasum -a 256 "$PROBE" | cut -c1-16) (build emendata, MAI pinnabile)"
FAIL=0
for C in objalloc objdatains; do
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$C.php")
  EXP=$("$ORACLE" "$M/$C.php")
  for r in 1 2; do
    GOT=$(PHPR_CTOR_PROBES="$OUT/$C-r$r.counts" "$PROBE" "$M/$C.php")
    [ "$GOT" = "$EXP" ] || { echo "$C r$r: PARITA' STDOUT ROTTA"; FAIL=1; }
  done
  if ! cmp -s "$OUT/$C-r1.counts" "$OUT/$C-r2.counts"; then
    # il pid nel dump differisce: confronta i soli contatori
    C1=$(sed 's/^ctorprobes pid=[0-9]* //' "$OUT/$C-r1.counts")
    C2=$(sed 's/^ctorprobes pid=[0-9]* //' "$OUT/$C-r2.counts")
    [ "$C1" = "$C2" ] || { echo "$C: CONTEGGI NON DETERMINISTICI tra R"; FAIL=1; }
  fi
  eval "$(sed 's/^ctorprobes pid=[0-9]* //; s/c\([0-9]\)=/K\1=/g' "$OUT/$C-r1.counts")"
  python3 - "$C" "$N" "$K0" "$K1" "$K2" "$K3" <<'PY'
import sys
c, n, k0, k1, k2, k3 = sys.argv[1], *map(int, sys.argv[2:])
print(f"{c}: N={n} sitoA(magic)={k0} ({k0/n:.4f}/iter) sitoB(propset)={k1} ({k1/n:.4f}/iter) TOT_resolve={k2} ({k2/n:.4f}/iter) entrate_propset={k3} ({k3/n:.4f}/iter)")
PY
done
if [ "$FAIL" = 0 ]; then echo "SONDA ACQUISITA (parita' ok, conteggi deterministici)"; echo 0 > "$RC"; else echo 1 > "$RC"; fi
} > "$VERD" 2>&1
exit "$(cat "$RC")"
