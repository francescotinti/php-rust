#!/bin/bash
# s149-sonda-pair.sh — sonda-PREZZO pair churn 16–48 B (criterio
# s149-criterio-pair.md). Base DICHIARATA: wp145-harness/s145-sonda-prezzi.sh
# (adattamenti: chiavi s149.price.* aggiunte allo smoke/check, verdetto e out
# in wp149-harness). Manifest s149-sonda-copia.diff (copia-gate). Probe
# monobinario `sonda-price` (nessuna census attiva): 1 smoke + 2 repliche del
# driver, chiavi a esito ESATTO, quiescenza col gate SEPARATO in retry (la CI
# locale ha il mutex sul lock). rc autoritativo = sonda-out/prezzi.rc.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp149-harness"
BIN="${PRICE_PHPR:?binario sonda-price richiesto}"
QUIESCE="$REPO/wp129-harness/s129-quiescenza.sh"
OUT="$H/sonda-out"; mkdir -p "$OUT"
VERD="$H/s149-sonda-pair-verdetto${TAG:-}.out"   # TAG=-tN per repliche
RC="$OUT/prezzi.rc"
LOCK="/private/tmp/phpr-measure.lock"
[ -e "$VERD" ] && { echo "verdetto ESISTE — tentativo nuovo = file nuovo" >&2; exit 7; }
KEYS145="cal mv_scalar mv_str mv_arr mv_obj note_scalar note_cont_repeat pair_zcell pair_arr0"
KEYS149="pair16 pair32 pair48 splitoff3"

check_keys() { # $1=file → 0 se tutte le chiavi presenti con valore >0
  for k in $KEYS145; do
    v=$(sed -n "s/^s145\.price\.${k}_ns=\([0-9.]*\)$/\1/p" "$1" | head -1)
    [ -n "${v:-}" ] || return 1
    awk -v x="$v" 'BEGIN{ exit !(x>0) }' || return 1
  done
  for k in $KEYS149; do
    v=$(sed -n "s/^s149\.price\.${k}_ns=\([0-9.]*\)$/\1/p" "$1" | head -1)
    [ -n "${v:-}" ] || return 1
    awk -v x="$v" 'BEGIN{ exit !(x>0) }' || return 1
  done
  grep -q '^s145\.price\.n_mv=' "$1" && grep -q '^s145\.price\.n_pair=' "$1" \
    && grep -q '^s149\.price\.zval_size=' "$1" && grep -q '^s149\.price\.n_pair=' "$1"
}
busy(){ { pgrep -x rustc || pgrep -x cargo || pgrep -f rust-analyzer; } > /dev/null 2>&1 && echo BUSY || echo CLEAN; }

{
echo "== s149 sonda-PREZZO pair churn 16-48 B (criterio s149-criterio-pair.md; base s145-sonda-prezzi.sh) =="
echo "probe_bin=$(shasum -a 256 "$BIN" | cut -c1-16) (build sonda-price, MAI parita')"
date "+start=%F %T"

# lock di finestra (mutex CI, ci/README.md): creato QUI, rimosso in coda
# ESPLICITAMENTE (mai da trap: il lock è di questa finestra, non del processo).
echo "s149-pair pid=$$ $(date +%FT%T)" > "$LOCK"
echo "lock: creato $LOCK"

# quiescenza in RETRY (gate separato, lezione S-128): la CI in volo la fa
# fallire finché il job non chiude; 45 tentativi x 60 s ~ 45 min max.
if [ -z "${SKIP_RA_KILL:-}" ]; then
  pkill -f rust-analyzer 2>/dev/null && echo "rust-analyzer: KILL inviato"
else
  echo "rust-analyzer: lasciato quieto (il gate CPU della quiescenza arbitra; il kill lo fa RIPARTIRE a indicizzare — osservato s145 t2: 21 tentativi)"
fi
QOK=1
for t in $(seq 1 45); do
  if "$QUIESCE" "$OUT/quiesce.rc" > "$OUT/quiesce-t$t.log" 2>&1; then QOK=0; echo "quiescenza: PASS al tentativo $t"; break; fi
  sleep 60
done
if [ "$QOK" != 0 ]; then echo "quiescenza: MAI PASS (45 tentativi) — STOP"; echo 8 > "$RC"; rm -f "$LOCK"; exit 8; fi

# smoke (criterio p.3/p.6: probe muto/parziale => STOP rc=8, niente run)
rm -f "$OUT/smoke-price.txt"
GOT=$(PHPR_SONDA_OUT="$OUT/smoke-price.txt" "$BIN" "$REPO/wp145-harness/driver-sonda-b.php" 2>&1)
if [ "$GOT" != "SONDA-OK" ] || ! check_keys "$OUT/smoke-price.txt"; then
  echo "smoke: FALLITO (stdout='$GOT'; chiavi s145=$(grep -c '^s145\.price\.' "$OUT/smoke-price.txt" 2>/dev/null || echo 0)/11 s149=$(grep -c '^s149\.price\.' "$OUT/smoke-price.txt" 2>/dev/null || echo 0)/6) — STOP rc=8"
  echo 8 > "$RC"; rm -f "$LOCK"; exit 8
fi
echo "smoke: PASS (stdout SONDA-OK, 11 chiavi s145 + 6 chiavi s149, prezzi >0)"
echo "zval_size=$(sed -n 's/^s149\.price\.zval_size=\([0-9]*\)$/\1/p' "$OUT/smoke-price.txt") (audit taglia, atteso 16)"

FAIL=0
for r in 1 2; do
  echo "r$r pre=$(busy)"
  rm -f "$OUT/price-r$r.txt"
  GOT=$(PHPR_SONDA_OUT="$OUT/price-r$r.txt" "$BIN" "$REPO/wp145-harness/driver-sonda-b.php" 2>&1)
  echo "r$r post=$(busy)"
  if [ "$GOT" != "SONDA-OK" ] || ! check_keys "$OUT/price-r$r.txt"; then
    echo "r$r: FALLITO (stdout='$GOT')"; FAIL=1
  else
    echo "r$r: OK $(tr '\n' ' ' < "$OUT/price-r$r.txt")"
  fi
done
rm -f "$LOCK"; echo "lock: rimosso"
date "+end=%F %T"
echo "$FAIL" > "$RC"
[ "$FAIL" = 0 ] && echo "PREZZI ACQUISITI (banda per chiave = [min,max] delle 2 repliche; replica >2% sulla chiave di decisione => t3 dovuta — criterio p.2)"
} > "$VERD" 2>&1
exit "$(cat "$RC")"
