#!/bin/bash
# s165-ririsolvi-guardie.sh — RI-RISOLUZIONE delle guardie backtrace/objmap
# morse a mc1dr5 (REGOLE §3, az.rev. S-154: guardia su giudice QUANTIZZATO —
# objmap tick 3,3 ns a N=3M, backtrace tick 4,2 a N=2,4M; i delta morsi sono
# multipli esatti del tick e gli STESSI binari erano puliti a mc1d — si
# ri-risolve a N con tick ≤ soglia/4 sui bracci di RECORD). Driver derivati
# DICHIARATI: m-backtrace96.php (N=9,6M) · m-objmap12.php (N=12M).
# CRITERIO PRE-registrato: R=5 interleaved, ns/iter=(med raw−floor)/N;
# backtrace soglia_reg = −max(4, rumore drop-1); objmap = −max(4, 3.3 banda
# fondata, rumore). REGRESSIONE persistente ⇒ prezzo REALE, niente promo;
# entrambe pulite ⇒ guardie mc1dr5 ARBITRATE (quantizzazione), verdetto R=5
# si integra a rc=0 EMENDATO in questo verbale. rc autoritativo = ab-out/ririsolvi.rc
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s163"
B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s165-mc1d-D"
OUT="$H/ab-out"; VERD="$H/s165-ririsolvi-verdetto.out"; RC="$OUT/ririsolvi.rc"
[ -e "$VERD" ] && { echo "verdetto ESISTE — TAG nuovo" >&2; exit 7; }
LOCK=/private/tmp/phpr-measure.lock
grep -qi "s165\|s-165" "$LOCK" 2>/dev/null || { echo "lock assente" | tee -a "$VERD"; echo 9 > "$RC"; exit 9; }
"$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-ririsolvi.rc" > /dev/null 2>&1 || { echo "quiescenza FAIL" | tee -a "$VERD"; echo 8 > "$RC"; exit 8; }
AM="$(shasum -a 256 "$A" | cut -c1-8)"; BM="$(shasum -a 256 "$B" | cut -c1-8)"
[ "$AM" = fea4a2d0 ] && [ "$BM" = 1fd8757d ] || { echo "hash bracci errati $AM/$BM" | tee -a "$VERD"; echo 1 > "$RC"; exit 1; }
ucpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die' -- "$1" "$2" > /dev/null; } 2>&1 | awk '/^user/{print $2}'; }
floor3() { local a b c; a=$(ucpu "$1" "$H/empty.php"); b=$(ucpu "$1" "$H/empty.php"); c=$(ucpu "$1" "$H/empty.php"); printf '%s\n%s\n%s\n' "$a" "$b" "$c" | sort -n | awk 'NR==2'; }
TSV="$OUT/ririsolvi-runs.tsv"; : > "$TSV"
{
echo "== s165 ri-risoluzione guardie quantizzate (bracci di record A=$AM B=$BM; criterio in testa allo script) =="
for CFG in "backtrace96 9600000 m-backtrace96.php 4.0" "objmap12 12000000 m-objmap12.php 4.0"; do
  set -- $CFG; C=$1; N=$2; P=$3; BASE=$4
  "$A" "$H/$P" > "$OUT/ri-$C-A.out" 2>&1; "$B" "$H/$P" > "$OUT/ri-$C-B.out" 2>&1
  diff -q "$OUT/ri-$C-A.out" "$OUT/ri-$C-B.out" > /dev/null || { echo "$C output DIVERGE"; echo 2 > "$RC"; exit 2; }
  FA=$(floor3 "$A"); FB=$(floor3 "$B")
  echo "cat=$C N=$N floor_A=$FA floor_B=$FB"
  for i in 1 2 3 4 5; do
    if [ $((i % 2)) -eq 1 ]; then TA=$(ucpu "$A" "$H/$P"); TB=$(ucpu "$B" "$H/$P"); ord=AB; else TB=$(ucpu "$B" "$H/$P"); TA=$(ucpu "$A" "$H/$P"); ord=BA; fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$C" "$N" "$TA" "$TB" "$FA" "$FB" "$BASE" >> "$TSV"
    echo "  coppia$i [$ord]: rawA=$TA rawB=$TB"
  done
done
python3 - "$TSV" <<'PY'
import sys
rows = {}
for l in open(sys.argv[1]):
    t = l.rstrip("\n").split("\t")
    rows.setdefault(t[0], []).append(t)
def med(v): s=sorted(v); n=len(s); return s[n//2] if n%2 else (s[n//2-1]+s[n//2])/2
def trange(v):
    m=med(v); w=sorted(v, key=lambda x:(abs(x-m),x))[:-1]; return max(w)-min(w)
rc=0
for cat, rs in rows.items():
    n=float(rs[0][1])
    na=[(float(t[2])-float(t[4]))/n*1e9 for t in rs]; nb=[(float(t[3])-float(t[5]))/n*1e9 for t in rs]
    ma,mb=med(na),med(nb); d=ma-mb; ra,rb=trange(na),trange(nb); noise=max(ra,rb)
    band = 3.3 if cat=="objmap12" else 0.0
    thr=max(4.0, band, noise)
    reg = d < -thr
    print(f"guardia {cat}: A={ma:.2f} B={mb:.2f} D={d:+.2f} soglia_reg={-thr:.2f} (tick<=1ns; rumore A'={ra:.2f} B'={rb:.2f}) -> {'REGRESSIONE PERSISTENTE' if reg else 'PULITA (morso mc1dr5 = quantizzazione arbitrata)'}")
    if reg: rc=5
print("ESITO: " + ("GUARDIA REALE — niente promo" if rc else "GUARDIE ARBITRATE — R=5 mc1dr5 si integra a rc=0 EMENDATO"))
sys.exit(rc)
PY
prc=$?
echo "$prc" > "$RC"; exit "$prc"
} >> "$VERD" 2>&1
