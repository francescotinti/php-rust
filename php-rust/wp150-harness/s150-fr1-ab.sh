#!/bin/bash
# s150-fr1-ab.sh — istruttoria FR1: A/B MONOBINARIO kill-switch (criterio
# s150-criterio-fr1.md p.4-6; fatti statici in s150-fr1-fatti.out).
# M1 = build con interruttore PHPR_FR1_OFF nel SOLO matcher (commit bd40fc1);
# RICETTA ESATTA DICHIARATA (lezione s150-identita-candidato.md):
#   SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 \
#   CARGO_TARGET_DIR=/private/tmp/phpr-m1-target \
#   cargo build --release -p php-cli --bin phpr
# Bracci: ON = M1 (peephole attivo) · OFF = M1 + PHPR_FR1_OFF=1 (fuso NON
# emesso; run_loop identico per costruzione). R=5 ABAB per giudice; DENTE
# p.5 su m-dimread (OFF DEVE perdere ~+16,7, pena VOID); ancoraggio smoke
# M1-ON vs pin s150; disasm run_loop M1 vs pin (attesi UGUALI). Grado
# ISTRUTTORIA: nessuna cifra in PERF_MAP. rc = fr1-out/rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp150-harness"
M1="${M1_BIN:?binario M1 richiesto}"
PIN="$HOME/Claude/php-rust-output/release/phpr"
ORACLE=/opt/homebrew/opt/php/bin/php
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
EMPTY="$REPO/wp97-harness/micro/empty.php"
J1="$REPO/wp146-harness/m-dimrmw10.php"; N1=30000000
J2="$REPO/wp145-harness/m-dimread.php";  N2=3000000
OUT="$H/fr1-out"; mkdir -p "$OUT"
VERD="$H/s150-fr1-verdetto.out"
RC="$OUT/rc"
LOCK=/private/tmp/phpr-measure.lock
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = TAG nuovo" >&2; exit 7; }
[ -e "$LOCK" ] || { echo "measure-lock ASSENTE" >&2; exit 6; }

user_of(){ awk '/^user/{print $2}' "$1"; }
run_once(){ # $1=FR1OFF(0|1) $2=script $3=tag → user; parita' stdout vs oracle
  local off="$1" scr="$2" tag="$3"
  local exp; exp="$("$ORACLE" "$scr")"
  local got
  if [ "$off" = 1 ]; then
    got=$( { PHPR_FR1_OFF=1 /usr/bin/time -p "$M1" "$scr" > "$OUT/$tag.out"; } 2> "$OUT/$tag.time"; cat "$OUT/$tag.out")
  else
    got=$( { /usr/bin/time -p "$M1" "$scr" > "$OUT/$tag.out"; } 2> "$OUT/$tag.time"; cat "$OUT/$tag.out")
  fi
  [ "$got" = "$exp" ] || { echo "PARITA' ROTTA $tag" >> "$OUT/parita.log"; }
  user_of "$OUT/$tag.time"
}

{
echo "== s150 istruttoria FR1 — A/B monobinario kill-switch (criterio s150-criterio-fr1.md; grado ISTRUTTORIA) =="
echo "M1=$(shasum -a 256 "$M1" | cut -c1-16) (commit bd40fc1, ricetta nel header)  pin=$(shasum -a 256 "$PIN" | cut -c1-16) (s150)"
date "+start=%F %T"
: > "$OUT/parita.log"
QOK=1
for t in $(seq 1 30); do
  if "$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce-t$t.log" 2>&1; then QOK=0; echo "quiescenza: PASS al tentativo $t"; break; fi
  sleep 60
done
[ "$QOK" = 0 ] || { echo "quiescenza MAI PASS — STOP"; echo 8 > "$RC"; exit 8; }

# sanity dell'interruttore (esito ESATTO, forgia mai silenziosa): il dump OFF
# NON deve contenere il fuso, il dump ON sì.
PHPR_DUMP_OPS=1 "$M1" "$J2" > /dev/null 2> "$OUT/dump-on.txt"
PHPR_FR1_OFF=1 PHPR_DUMP_OPS=1 "$M1" "$J2" > /dev/null 2> "$OUT/dump-off.txt"
CON=$(grep -c "PropDimGetConst" "$OUT/dump-on.txt"); COFF=$(grep -c "PropDimGetConst" "$OUT/dump-off.txt") || true
echo "dump m-dimread: fuso ON=$CON OFF=$COFF (attesi: >=1 / 0)"
if [ "$CON" -lt 1 ] || [ "$COFF" != 0 ]; then
  echo "INTERRUTTORE NON PROVATO — VOID"; echo 9 > "$RC"; date "+end=%F %T"; exit 9
fi

# pavimento med3 (stesso binario per i due bracci: UN pavimento, misurato)
for r in 1 2 3; do run_once 0 "$EMPTY" "fl$r" > "$OUT/fl$r.u"; done
FL=$(sort -n "$OUT"/fl[123].u | awk 'NR==2')
echo "pavimento M1 med3=$FL"

# ancoraggio: M1-ON vs pin s150, smoke R=2 ABAB per giudice (scarto dichiarato)
for j in J1 J2; do
  scr=$([ $j = J1 ] && echo "$J1" || echo "$J2")
  a1=$(run_once 0 "$scr" "anc-$j-m1a"); { /usr/bin/time -p "$PIN" "$scr" > /dev/null; } 2> "$OUT/anc-$j-p1.time"
  a2=$(run_once 0 "$scr" "anc-$j-m1b"); { /usr/bin/time -p "$PIN" "$scr" > /dev/null; } 2> "$OUT/anc-$j-p2.time"
  p1=$(user_of "$OUT/anc-$j-p1.time"); p2=$(user_of "$OUT/anc-$j-p2.time")
  echo "ancoraggio $j: M1-ON=$a1/$a2 pin=$p1/$p2 (scarto kill-switch dichiarato, non-gate)"
done

# A/B R=5 ABAB per giudice: ON poi OFF a coppie
for j in J1 J2; do
  scr=$([ $j = J1 ] && echo "$J1" || echo "$J2")
  uon=(); uoff=()
  for r in 1 2 3 4 5; do
    uon+=("$(run_once 0 "$scr" "ab-$j-on$r")")
    uoff+=("$(run_once 1 "$scr" "ab-$j-off$r")")
  done
  echo "$j grezzi ON: ${uon[*]}  OFF: ${uoff[*]}"
  N=$([ $j = J1 ] && echo "$N1" || echo "$N2")
  SOGLIA_MIN=$([ $j = J1 ] && echo "1.0" || echo "4.0")
  python3 - "$FL" "$N" "$SOGLIA_MIN" "$j" "${uon[@]}" "${uoff[@]}" <<'PY'
import sys
fl, n, smin = float(sys.argv[1]), float(sys.argv[2]), float(sys.argv[3])
j = sys.argv[4]
on = list(map(float, sys.argv[5:10])); off = list(map(float, sys.argv[10:15]))
segni = sum(1 for x, y in zip(off, on) if x > y)   # OFF piu' lento (in ordine di run)
o = sorted(on); f = sorted(off)
non = (o[2]-fl)/n*1e9; noff = (f[2]-fl)/n*1e9
drop1 = max(o[3]-o[0], f[3]-f[0])/n*1e9
soglia = max(smin, drop1)
d = noff - non   # >0 = OFF piu' lento (il fuso AIUTA); <0 = OFF piu' veloce (il fuso COSTA)
print(f"{j}: ON={non:.1f} ns/iter  OFF={noff:.1f} ns/iter  D(OFF-ON)={d:+.1f}  segni_OFF_lento={segni}/5  soglia={soglia:.1f}")
PY
done

# disasm run_loop M1 vs pin (metodo S-109; attesi UGUALI — registrati)
for side in M1 PIN; do
  bin=$([ "$side" = M1 ] && echo "$M1" || echo "$PIN")
  sym=$(nm -n "$bin" 2>/dev/null | grep "8run_loop17h" | awk '{print $3}')
  SZ=$(nm -n "$bin" 2>/dev/null | grep -A1 "8run_loop17h" | head -2 | python3 -c '
import sys
lines = sys.stdin.read().split("\n")
print(int(lines[1].split()[0],16)-int(lines[0].split()[0],16))')
  BL=$(otool -tv "$bin" 2>/dev/null | awk -v s="$sym:" '
    $0 == s {f=1; bl=0; next}
    f && /^__/ {print bl; exit}
    f && $2 == "bl" {bl++}
  ')
  echo "disasm $side: run_loop size=$SZ bl=$BL"
done
echo "parita' stdout: $(wc -l < "$OUT/parita.log" | tr -d ' ') violazioni ($(sort -u "$OUT/parita.log" 2>/dev/null | tr '\n' ' '))"
date "+end=%F %T"
echo 0 > "$RC"
} > "$VERD" 2>&1
exit "$(cat "$RC")"
