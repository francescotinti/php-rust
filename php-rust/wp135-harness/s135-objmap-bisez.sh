#!/bin/bash
# s135-objmap-bisez.sh — istruttoria objmap: bisezione bilaterale a 4 varianti
# (criterio s135-criterio-bisez.md, committato PRIMA del run; copia DICHIARATA
# di s126-orm-micro.sh). Metodo run-micro.sh: user CPU, pavimento per-binario,
# R=5 alternato, N dal sorgente, parita' stdout.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
M="$H/micro-bisez"
OUT="$H/bisez-out"; mkdir -p "$OUT"
VERD="$H/s135-objmap-bisez-verdetto.out"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
R=5
CATS="${CATS:-m0_obj2048 m1_int2048 m2_obj0 m3_int0}"
VERD="${VERD_OVERRIDE:-$VERD}"
rm -f "$OUT/micro.done"
[ "$(shasum -a 256 "$PHPR" | cut -c1-16)" = "61896da13654fd00" ] || { echo "rc=9 pin!=s134" > "$OUT/micro.done"; exit 9; }

user_cpu() { # <binario> <script> <stdout-file> -> secondi user CPU (alarm 900: sopravvive all'exec)
  { /usr/bin/time -p perl -e 'alarm 900; exec @ARGV or die "exec: $!"' -- "$1" "$2" > "$3"; } 2>&1 | awk '/^user/{print $2}'
}
median() { printf '%s\n' "$@" | sort -n | awk '{v[NR]=$1} END{print (NR%2)?v[(NR+1)/2]:(v[NR/2]+v[NR/2+1])/2}'; }
minmax() { printf '%s\n' "$@" | sort -n | awk 'NR==1{lo=$1} {hi=$1} END{printf "%s %s", lo, hi}'; }

{
echo "== s135 istruttoria objmap: bisezione bilaterale (criterio s135-criterio-bisez.md) =="
echo "grade=VERDICT  # derivazione meccanica; R=$R; user CPU netto-pavimento per-binario"
echo "oracle=$("$ORACLE" -r 'echo PHP_VERSION;') phpr=$(shasum -a 256 "$PHPR" | cut -c1-16)"

fo=(); fp=()
for i in $(seq 1 "$R"); do
  fo+=("$(user_cpu "$ORACLE" "$M/empty.php" /dev/null)")
  fp+=("$(user_cpu "$PHPR" "$M/empty.php" /dev/null)")
done
FO=$(median "${fo[@]}"); FP=$(median "${fp[@]}")
echo "pavimento_oracle_s=$FO pavimento_phpr_s=$FP"

for c in $CATS; do
  to=(); tp=()
  for i in $(seq 1 "$R"); do
    to+=("$(user_cpu "$ORACLE" "$M/$c.php" "$OUT/$c-oracle-$i.stdout")")
    tp+=("$(user_cpu "$PHPR"   "$M/$c.php" "$OUT/$c-phpr-$i.stdout")")
  done
  N=$(awk 'match($0, /\$i<[0-9]+/) {print substr($0, RSTART+3, RLENGTH-3); exit}' "$M/$c.php")
  # parita': tutti gli stdout (2 motori x R) identici tra loro
  PAR=$(cat "$OUT/$c-"*.stdout | sort -u | wc -l | tr -d ' ')
  VUOTI=0
  for v in "${to[@]}" "${tp[@]}"; do [ -n "$v" ] || VUOTI=1; done
  if [ "$VUOTI" = 1 ]; then echo "${c}: CATEGORIA NULLA (alarm/errore: misura vuota)"; continue; fi
  MO=$(median "${to[@]}"); MP=$(median "${tp[@]}")
  read -r LO HO <<<"$(minmax "${to[@]}")"
  read -r LP HP <<<"$(minmax "${tp[@]}")"
  python3 - "$c" "$MO" "$MP" "$FO" "$FP" "$LO" "$HO" "$LP" "$HP" "${N:-0}" "$PAR" <<'PY'
import sys
c = sys.argv[1]; mo, mp, fo, fp, lo, ho, lp, hp = map(float, sys.argv[2:10])
n = int(sys.argv[10]); par = int(sys.argv[11])
no, np_ = mo - fo, mp - fp
print(f"{c}_N_iter={n}")
print(f"{c}_oracle_netto_s={no:.2f} {c}_phpr_netto_s={np_:.2f}")
if par != 1:
    print(f"{c}: PARITA' ROTTA (stdout distinti={par}) — cifra NULLA")
elif no <= 0:
    print(f"{c}: oracle netto <=0 — cifra NULLA (N troppo piccolo)")
else:
    print(f"rapporto_{c}={np_/no:.1f}  [companion {np_:.2f}/{no:.2f}]")
    print(f"{c}_ns_iter: oracle={no/n*1e9:.1f} phpr={np_/n*1e9:.1f} delta={np_/n*1e9-no/n*1e9:+.1f}")
print(f"{c}_spread_oracle_s={ho-lo:.2f} {c}_spread_phpr_s={hp-lp:.2f}")
PY
done
} > "$VERD" 2>&1
echo "rc=0 $(date +%T)" > "$OUT/micro.done"
