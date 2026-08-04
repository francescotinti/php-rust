#!/bin/bash
# run-micro.sh — LA SPINA DORSALE: il rapporto phpr/oracle per CATEGORIA di
# lavoro dell'interprete, misurato sullo STESSO sorgente PHP.
#
# Perche' esiste: fino a S-96.0 il progetto ha misurato UN solo numero, il
# rapporto sulla suite WordPress (1,873x), e ha scelto le leve guardando il
# profilo di phpr DA SOLO. Un profilo a un lato solo dice dove phpr spende
# tempo, non dove ne spende PIU' dell'oracle: seleziona le cose che entrambi i
# motori fanno. E un aggregato diluito nasconde il segnale — sulla suite
# WordPress la maggior parte del tempo e' I/O, database e builtin, dove phpr
# regge; la lentezza vera del nucleo interprete non si vede.
#
# Qui ogni categoria isola UN meccanismo e lo misura in un ciclo stretto, dove
# il segnale e' di ordine 10x invece che dell'1%.
#
# IL PAVIMENTO NON E' OPZIONALE: l'avvio a vuoto dell'oracle e di phpr sono
# diversi (l'oracle e' piu' lento a partire), e senza sottrarli i rapporti
# escono sbagliati di un fattore 3 — errore commesso e corretto alla prima
# misura di questo harness.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
R="${R:-3}"
CATS="arith prop calls str arr re"

user_cpu() { # <binario> <script> -> secondi di user CPU
  # `time` scrive su stderr: se si zittisce lo stderr del comando qui dentro si
  # zittisce anche la misura. Solo lo STDOUT dello script va in /dev/null.
  { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'
}

median() { printf '%s\n' "$@" | sort -n | awk '{v[NR]=$1} END{print (NR%2)?v[(NR+1)/2]:(v[NR/2]+v[NR/2+1])/2}'; }
minmax() { printf '%s\n' "$@" | sort -n | awk 'NR==1{lo=$1} {hi=$1} END{printf "%s %s", lo, hi}'; }

echo "micro-baseline: il rapporto phpr/oracle PER CATEGORIA (la spina dorsale, S-97.0)"
echo "grade=VERDICT  # R=$R ripetizioni per lato, mediana E spread pubblicati; l'effetto misurato"
echo "  e' di ordine 10x, cioe' tre ordini di grandezza sopra la risoluzione del cronometro"
echo "formato=ascii-nudo"
echo "oracle=$("$ORACLE" -r 'echo PHP_VERSION;') jit=$("$ORACLE" -r 'echo ini_get("opcache.jit") ?: "n/d";')"
echo "phpr=$(shasum -a 256 "$PHPR" | cut -c1-16)"
echo "ripetizioni=$R"
echo "metodo=user CPU di /usr/bin/time -p, AL NETTO del pavimento di avvio misurato"
echo "  sullo stesso binario con uno script vuoto (i due pavimenti sono DIVERSI)"

# --- pavimento ---
fo=(); fp=()
for i in $(seq 1 "$R"); do
  fo+=("$(user_cpu "$ORACLE" "$H/empty.php")")
  fp+=("$(user_cpu "$PHPR" "$H/empty.php")")
done
FO=$(median "${fo[@]}"); FP=$(median "${fp[@]}")
echo "pavimento_oracle_s=$FO"
echo "pavimento_phpr_s=$FP"
echo

for c in $CATS; do
  to=(); tp=()
  for i in $(seq 1 "$R"); do
    to+=("$(user_cpu "$ORACLE" "$H/$c.php")")
    tp+=("$(user_cpu "$PHPR" "$H/$c.php")")
  done
  MO=$(median "${to[@]}"); MP=$(median "${tp[@]}")
  read -r LO HO <<<"$(minmax "${to[@]}")"
  read -r LP HP <<<"$(minmax "${tp[@]}")"
  python3 - "$c" "$MO" "$MP" "$FO" "$FP" "$LO" "$HO" "$LP" "$HP" <<'PY'
import sys
c, mo, mp, fo, fp, lo, ho, lp, hp = sys.argv[1], *map(float, sys.argv[2:])
no, np_ = mo - fo, mp - fp
print(f"{c}_oracle_netto_s={no:.2f}")
print(f"{c}_phpr_netto_s={np_:.2f}")
print(f"rapporto_{c}={np_/no:.1f}  [derivata: companion {np_:.2f}/{no:.2f}]")
print(f"{c}_spread_oracle_s={ho-lo:.2f}  {c}_spread_phpr_s={hp-lp:.2f}")
PY
done
