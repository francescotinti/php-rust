#!/bin/bash
# s164-arith-dq.sh — FASE 2 indagine arith (criterio s164-criterio-arith.md
# p.3-4, pre-registrato): rimisura DE-QUANTIZZATA arith-only su 4 bracci
# (oracle brew + stash FERMI s161/s162/s163 byte-verificati), driver
# arith-dq.php N=250000000 (tick rapporto ~0,026 <= 0,1/4), R=5 interleaved
# a ROTAZIONE O->161->162->163, pavimenti per-binario R=5, report a 3
# decimali. Derivato DICHIARATO di wp97-harness/micro/run-micro.sh
# (user_cpu/median identici; empty.php letto read-only dal micro canonico).
# Attese DAL CRITERIO p.4: (a) max-min tra le mediane nette phpr <= 0,40
# (banda ±0,20); (b) oracle netto in [2,15; 2,20]. rc SOLO da
# arith-out/arith-dq.done: 0 = indagine CHIUSA (denominatore/quantizzazione)
# · 5 = (a) violato: creep reale, istruttoria · 6 = (b) violato: finestra
# sporca, si ripete UNA volta · 8/9 = precondizioni.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp164-harness"
MIC="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp97-harness/micro"
ST="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
ORACLE="/opt/homebrew/opt/php/bin/php"
DRV="$H/arith-dq.php"; OUT="$H/arith-out"; mkdir -p "$OUT"
VERD="$H/s164-arith-dq-verdetto.out"; : > "$VERD"
RCF="$OUT/arith-dq.done"; rm -f "$RCF"
fin(){ echo "rc=$1 $(date +%T)" > "$RCF"; exit "$1"; }

[ -e /private/tmp/phpr-measure.lock ] || { echo "rc=9 measure-lock ASSENTE" >> "$VERD"; fin 9; }
declare -A BIN EXP
BIN[s161]="$ST/phpr-s161"; EXP[s161]=ec0a636ad0c42005
BIN[s162]="$ST/phpr-s162"; EXP[s162]=20c63af44bfd077a
BIN[s163]="$ST/phpr-s163"; EXP[s163]=fea4a2d040a0d8d0
for k in s161 s162 s163; do
  hh=$(shasum -a 256 "${BIN[$k]}" | cut -c1-16)
  [ "$hh" = "${EXP[$k]}" ] || { echo "rc=9 stash $k hash=$hh atteso=${EXP[$k]}" >> "$VERD"; fin 9; }
done
N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$DRV")
[ "$N" = "250000000" ] || { echo "rc=8 N driver=$N atteso 250000000" >> "$VERD"; fin 8; }

user_cpu() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
median() { printf '%s\n' "$@" | sort -n | awk '{v[NR]=$1} END{print (NR%2)?v[(NR+1)/2]:(v[NR/2]+v[NR/2+1])/2}'; }

{ echo "== S-164 p.2 FASE 2 — arith de-quantizzata (criterio s164-criterio-arith.md p.3-4; N=$N; R=5 rotazione O->161->162->163) =="
  echo "grade=VERDICT  # rc autoritativo = arith-out/arith-dq.done"
  echo "bracci: oracle=$("$ORACLE" -r 'echo PHP_VERSION;') s161=${EXP[s161]} s162=${EXP[s162]} s163=${EXP[s163]} (byte-verificati)"
  echo "sentinella igiene INIZIO: $(uptime | tr -s ' ')"
  echo "  mediaanalysisd_cpu=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')"
} >> "$VERD"

# pavimenti per-binario R=5 (empty.php dal micro canonico, read-only)
declare -A FLR
for k in O s161 s162 s163; do
  b="$ORACLE"; [ "$k" != O ] && b="${BIN[$k]}"
  fs=(); for r in 1 2 3 4 5; do fs+=("$(user_cpu "$b" "$MIC/empty.php")"); done
  FLR[$k]=$(median "${fs[@]}")
  echo "pavimento_${k}_s=${FLR[$k]}" >> "$VERD"
done
# misure interleaved a rotazione (raw.log azzerato: niente residui di run precedenti)
rm -f "$OUT/raw.log"
for r in 1 2 3 4 5; do
  for k in O s161 s162 s163; do
    b="$ORACLE"; [ "$k" != O ] && b="${BIN[$k]}"
    v=$(user_cpu "$b" "$DRV")
    echo "raw r$r $k=$v" >> "$OUT/raw.log"
  done
done
python3 - "$VERD" <<PYE
import statistics
flr = {"O": ${FLR[O]}, "s161": ${FLR[s161]}, "s162": ${FLR[s162]}, "s163": ${FLR[s163]}}
raw = {}
for l in open("$OUT/raw.log"):
    t = l.split()
    if len(t) == 3 and t[0] == "raw":
        k, v = t[2].split("=")
        raw.setdefault(k, []).append(float(v))
out = open("$VERD", "a")
net = {}
for k in ("O", "s161", "s162", "s163"):
    med = statistics.median(raw[k]); net[k] = med - flr[k]
    sp = max(raw[k]) - min(raw[k])
    out.write(f"{k}_mediana_s={med:.3f} netto_s={net[k]:.3f} spread_s={sp:.3f}\n")
for k in ("s161", "s162", "s163"):
    out.write(f"rapporto_{k}={net[k]/net['O']:.3f}\n")
mm = max(net[k] for k in ("s161","s162","s163")) - min(net[k] for k in ("s161","s162","s163"))
out.write(f"phpr_maxmin_s={mm:.3f} (attesa a: <=0.40)\n")
out.write(f"oracle_netto_s={net['O']:.3f} (attesa b: [2.15; 2.20])\n")
rc = 0
if not (2.15 <= net["O"] <= 2.20):
    out.write("ATTESA (b) VIOLATA: finestra sporca — si dichiara e si ripete UNA volta\n"); rc = 6
elif mm > 0.40:
    out.write("ATTESA (a) VIOLATA: creep REALE tra stash — istruttoria per-pin, NESSUNA taratura\n"); rc = 5
else:
    out.write("ATTESE (a)+(b) CONFERMATE: tick 5,3->5,5 = quantizzazione + centesimo oracle s161, NESSUNA regressione phpr — indagine CHIUSA\n")
open("$OUT/pyrc", "w").write(str(rc))
PYE
{ echo "sentinella igiene FINE: $(uptime | tr -s ' ')"
  echo "  mediaanalysisd_cpu=$(top -l 2 -stats pid,cpu,command 2>/dev/null | awk '/mediaanalysisd/ {v=$2} END {print (v==""?0:v)}')"
} >> "$VERD"
RC=$(cat "$OUT/pyrc" 2>/dev/null || echo 8)
echo "ESITO: rc=$RC" >> "$VERD"
fin "$RC"
