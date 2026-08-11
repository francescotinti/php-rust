#!/bin/bash
# s129-tempo-pin.sh — sonde temporali PIN bilaterali (criterio s129-criterio-tempo.md
# p.3, committato PRIMA di questo codice). Metodo s127-submicro: user CPU,
# pavimento med-R per-binario, R=5 alternato oracle/phpr, N dal sorgente,
# parità stdout. Derivate del modello per motore: Δins, Δ2nd, Δover, Δapp, Δloc.
# rc autoritativo = SOLO tempo-out/tempo-pin.rc scritto QUI.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp129-harness"
M="$REPO/wp127-harness/micro-orm"
P="$REPO/wp128-harness/probe"
OUT="$H/tempo-out"; mkdir -p "$OUT"
VERD="$H/s129-tempo-pin-verdetto.out"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
R=5
rm -f "$OUT/tempo-pin.rc"
[ "$(shasum -a 256 "$PHPR" | cut -c1-16)" = "ccb63dcaf565cffc" ] || { echo 9 > "$OUT/tempo-pin.rc"; echo "pin!=s127b" > "$VERD"; exit 9; }

src_of(){ case "$1" in p2-append|p3-local|p4-int|p5-two|p6-overwrite) echo "$P/$1.php";; *) echo "$M/$1.php";; esac; }
user_cpu() { { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die "exec: $!"' -- "$1" "$2" > "$3"; } 2>&1 | awk '/^user/{print $2}'; }
median() { printf '%s\n' "$@" | sort -n | awk '{v[NR]=$1} END{print (NR%2)?v[(NR+1)/2]:(v[NR/2]+v[NR/2+1])/2}'; }
minmax() { printf '%s\n' "$@" | sort -n | awk 'NR==1{lo=$1} {hi=$1} END{printf "%s %s", lo, hi}'; }

CATS="objalloc objdatains p2-append p4-int p5-two p6-overwrite p3-local"
{
echo "== s129 sonde temporali PIN (criterio p.3; R=$R alternato; user CPU netto-pavimento per-binario) =="
echo "grade=VERDICT  # rc autoritativo = tempo-out/tempo-pin.rc"
echo "oracle=$("$ORACLE" -r 'echo PHP_VERSION;') phpr=$(shasum -a 256 "$PHPR" | cut -c1-16) rustc_pin=n/a data=$(date '+%F %T')"

fo=(); fp=()
for i in $(seq 1 "$R"); do
  fo+=("$(user_cpu "$ORACLE" "$M/empty.php" /dev/null)")
  fp+=("$(user_cpu "$PHPR" "$M/empty.php" /dev/null)")
done
FO=$(median "${fo[@]}"); FP=$(median "${fp[@]}")
echo "pavimento_oracle_s=$FO pavimento_phpr_s=$FP"

declare -a NSO=() NSP=() CN=()
for c in $CATS; do
  s="$(src_of "$c")"
  to=(); tp=()
  for i in $(seq 1 "$R"); do
    to+=("$(user_cpu "$ORACLE" "$s" "$OUT/$c-oracle-$i.stdout")")
    tp+=("$(user_cpu "$PHPR"   "$s" "$OUT/$c-phpr-$i.stdout")")
  done
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$s")
  PAR=$(cat "$OUT/$c-"*.stdout | sort -u | wc -l | tr -d ' ')
  VUOTI=0; for v in "${to[@]}" "${tp[@]}"; do [ -n "$v" ] || VUOTI=1; done
  if [ "$VUOTI" = 1 ]; then echo "${c}: CATEGORIA NULLA (misura vuota)"; continue; fi
  MO=$(median "${to[@]}"); MP=$(median "${tp[@]}")
  read -r LO HO <<<"$(minmax "${to[@]}")"
  read -r LP HP <<<"$(minmax "${tp[@]}")"
  OUTLINE=$(python3 - "$c" "$MO" "$MP" "$FO" "$FP" "$LO" "$HO" "$LP" "$HP" "${N:-0}" "$PAR" <<'PY'
import sys
c = sys.argv[1]; mo, mp, fo, fp, lo, ho, lp, hp = map(float, sys.argv[2:10])
n = int(sys.argv[10]); par = int(sys.argv[11])
no, np_ = mo - fo, mp - fp
if par != 1 or no <= 0 or n == 0:
    print(f"{c}: NULLA (par={par} no={no:.2f} N={n})")
else:
    print(f"{c}: ns/iter oracle={no/n*1e9:.1f} phpr={np_/n*1e9:.1f} rapporto={np_/no:.1f} "
          f"spread_o={ (ho-lo)/n*1e9:.1f} spread_p={ (hp-lp)/n*1e9:.1f} [net {no:.2f}/{np_:.2f} s]")
PY
)
  echo "$OUTLINE"
  NSO+=("$(echo "$OUTLINE" | awk 'match($0,/oracle=[0-9.]+/){print substr($0,RSTART+7,RLENGTH-7)}')")
  NSP+=("$(echo "$OUTLINE" | awk 'match($0,/phpr=[0-9.]+/){print substr($0,RSTART+5,RLENGTH-5)}')")
  CN+=("$c")
done

python3 - "${#CN[@]}" "${CN[@]}" "${NSO[@]}" "${NSP[@]}" <<'PY'
import sys
k = int(sys.argv[1])
cats = sys.argv[2:2+k]
so = list(map(float, sys.argv[2+k:2+2*k]))
sp = list(map(float, sys.argv[2+2*k:2+3*k]))
o = dict(zip(cats, so)); p = dict(zip(cats, sp))
def d(a, b, tag):
    if a in o and b in o:
        print(f"{tag}: oracle={o[a]-o[b]:+.1f} phpr={p[a]-p[b]:+.1f} ns/iter (rapporto {'' if o[a]-o[b]<=0 else f'{(p[a]-p[b])/(o[a]-o[b]):.1f}'})")
print("-- derivate del modello (per motore, ns/iter) --")
d("objdatains", "objalloc", "Dins  (insert 'k')")
d("p5-two", "objdatains", "D2nd  (2o insert)")
d("p6-overwrite", "objdatains", "Dover (overwrite)")
d("p2-append", "objalloc", "Dapp  (append)")
d("p4-int", "objalloc", "Dint  (insert 0)")
d("p3-local", "objalloc", "Dloc  (locale)")
print("risoluzione dichiarata: max(4 ns/iter, spread del run) — sotto: non risolto")
PY
} > "$VERD" 2>&1
echo 0 > "$OUT/tempo-pin.rc"
echo "done rc=0 $(date +%T)" > "$OUT/tempo-pin.done"
