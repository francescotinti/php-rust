#!/bin/bash
# s152-census-quiet.sh — rerun QUIET testa hostcall (criterio s152-criterio-quiet.md).
# Base DICHIARATA: wp151-harness/s151-census-orm.sh (adattamenti: out/verdetto in
# wp152-harness; quiescenza s129 in retry PRIMA delle repliche; giudizio = solo
# Δ hostcall_n + sentinelle, le identità piene restano al verdetto s151).
# rc: 0=eseguito con giudizio · 7=setup · 8=probe muto · 9=lock altrui.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp152-harness"; OUT="$H/quiet-out"; mkdir -p "$OUT"
V="$H/s152-census-quiet-verdetto.out"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
PROBE="$REPO/wp151-harness/census-prep/phpr-census-s151"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
LOCK=/private/tmp/phpr-measure.lock
[ -e "$V" ] && { echo "verdetto ESISTE" >&2; exit 7; }
fin(){ echo "$1" > "$OUT/quiet.rc"; touch "$OUT/quiet.done"; if [ -f "$LOCK" ] && grep -q "s152-quiet" "$LOCK" 2>/dev/null; then rm -f "$LOCK"; echo "lock: rimosso (era mio)" >> "$V"; fi; exit "$1"; }

echo "== s152 rerun QUIET testa hostcall (criterio s152-criterio-quiet.md; probe ab02faec0abfab67) ==" > "$V"
HP=$(shasum -a 256 "$PROBE" | cut -c1-16)
[ "$HP" = "ab02faec0abfab67" ] || { echo "rc=7 probe $HP != atteso" >> "$V"; fin 7; }
if [ -e "$LOCK" ]; then echo "rc=9 lock di misura ALTRUI: abort" >> "$V"; echo 9 > "$OUT/quiet.rc"; touch "$OUT/quiet.done"; exit 9; fi
echo "s152-quiet pid=$$ avvio $(date '+%F %T')" > "$LOCK"
echo "lock: CREATO (finestra quiet; rimozione a fine run)" >> "$V"

QOK=1
for t in $(seq 1 45); do
  if "$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce-t$t.log" 2>&1; then QOK=0; echo "quiescenza: PASS al tentativo $t" >> "$V"; break; fi
  sleep 60
done
[ "$QOK" = 0 ] || { echo "quiescenza: MAI PASS — STOP" >> "$V"; fin 8; }

PAD=$(printf 'x%.0s' $(seq 1 70))
SP="/private/tmp/s152-quiet-sp-$PAD"
WORK="$SP/orm-work"
[ "${#WORK}" -ge 100 ] || { echo "rc=7 workdir ${#WORK} <100" >> "$V"; fin 7; }
echo "workdir=${#WORK} char (>=100, come s151)" >> "$V"
rm -rf "$SP"; mkdir -p "$SP"
tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo "rc=7 untar" >> "$V"; fin 7; }
cd "$WORK" || { echo "rc=7 cd" >> "$V"; fin 7; }

busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

for R in 1 2; do
  RAW="$OUT/census-mem-r$R.txt"; rm -f "$RAW"
  PRE=$(busy)
  PHPR_MEM_CENSUS="$RAW" "$PROBE" vendor/bin/phpunit --no-coverage > "$OUT/run-r$R.txt" 2>&1 &
  PID=$!
  ( sleep 1800; kill -9 "$PID" 2>/dev/null ) & WD=$!
  wait "$PID"; RC=$?
  kill "$WD" 2>/dev/null; wait "$WD" 2>/dev/null
  POST=$(busy)
  echo "replica r$R: rc=$RC busy_pre=$PRE busy_post=$POST (attese CLEAN, criterio §2)" >> "$V"
  [ -s "$RAW" ] || { echo "rc=8 raw r$R VUOTO" >> "$V"; fin 8; }
  tr -d '\0' < "$OUT/run-r$R.txt" | sed -n 's/^[0-9][0-9]*) \(.*\)$/\1/p' | sort -u > "$OUT/r$R.failnames"
  tr -d '\0' < "$RAW" > "$OUT/clean-r$R.txt"
done
diff -q "$REPO/wp125-harness/orm-baseline-failnames.txt" "$OUT/r1.failnames" > /dev/null 2>&1 && P1=OK || P1=DIFF
diff -q "$OUT/r1.failnames" "$OUT/r2.failnames" > /dev/null 2>&1 && P12=OK || P12=DIFF
echo "sentinella fail-set: r1-vs-baseline16=$P1 · r1-vs-r2=$P12 (non-gate, si dichiara)" >> "$V"

H1=$(awk '/^s149sum /{for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="hostcall_n") s+=kv[2]}} END{print s+0}' "$OUT/clean-r1.txt")
H2=$(awk '/^s149sum /{for(i=2;i<=NF;i++){split($i,kv,"="); if(kv[1]=="hostcall_n") s+=kv[2]}} END{print s+0}' "$OUT/clean-r2.txt")
D=$((H1>H2 ? H1-H2 : H2-H1))
echo "HOSTN_QUIET_R1=$H1" >> "$V"
echo "HOSTN_QUIET_R2=$H2" >> "$V"
echo "DELTA=$D" >> "$V"
if [ "$D" -eq 0 ]; then
  echo "GIUDIZIO (criterio §3): Δ=0 ⇒ non-determinismo s151 ATTRIBUITO alla contesa (meccanismo indiretto, dichiarato); testa hostcall CITABILE alla cifra quiet $H1" >> "$V"
else
  echo "GIUDIZIO (criterio §3): Δ=$D ≠ 0 ⇒ testa hostcall NON citabile; meccanismo resta da nominare (apertura per NOME); canali C1–C5 non toccati" >> "$V"
fi
fin 0
