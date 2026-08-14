#!/bin/bash
# s136-sonda-fedelta.sh — az.rev. S-135 #1: la sonda di fedeltà AP1 registrata
# AGLI ATTI (script + verdetto + rc). Fixture v2 (uno statement per riga +
# sezioni s15-s22 dai probe del revisore) sui TRE binari: oracle, pin s135,
# stash s134. Giudice meccanico:
#   rc=0  pin==stash BYTE-ID e diff-oracle SOLO nelle famiglie a catalogo §3.21
#   rc=1  pin!=stash (claim di equivalenza della leva AP1 FALSIFICATO)
#   rc=2  diff-oracle con righe NON catalogate (divergenza nuova da istruire)
#   rc=9  pin/stash non conformi al registro (hash)
# rc autoritativo = sonda-out/sonda-fedelta.rc (mai da pipe).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp136-harness"
OUT="$H/sonda-out"; mkdir -p "$OUT"
VERD="$H/s136-sonda-fedelta-verdetto.out"
RCF="$OUT/sonda-fedelta.rc"
FIX="$H/fixtures-ap1-v2.php"
ORACLE=/opt/homebrew/opt/php/bin/php
PIN="$HOME/Claude/php-rust-output/release/phpr"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s134"
PIN_ATTESO="6518a1e14a266d52"
STASH_ATTESO="61896da13654fd00"

fin(){ echo "$1" > "$RCF"; exit "$1"; }

P="$(shasum -a 256 "$PIN" | cut -c1-16)"
S="$(shasum -a 256 "$STASH" | cut -c1-16)"
{
echo "== s136 sonda fedeltà AP1 (fixture v2, az.rev. S-135 #1) $(date '+%F %T') =="
echo "pin=$P (atteso $PIN_ATTESO) stash=$S (atteso $STASH_ATTESO)"
} > "$VERD"
if [ "$P" != "$PIN_ATTESO" ] || [ "$S" != "$STASH_ATTESO" ]; then
  echo "ESITO: rc=9 hash non conforme" >> "$VERD"; fin 9
fi

MIMALLOC_PURGE_DELAY=0 "$ORACLE" "$FIX" > "$OUT/fix-oracle.txt" 2>&1
echo "oracle rc=$?" >> "$VERD"
MIMALLOC_PURGE_DELAY=0 "$PIN"    "$FIX" > "$OUT/fix-pin.txt" 2>&1
echo "pin rc=$?" >> "$VERD"
MIMALLOC_PURGE_DELAY=0 "$STASH"  "$FIX" > "$OUT/fix-stash.txt" 2>&1
echo "stash rc=$?" >> "$VERD"

if ! diff "$OUT/fix-pin.txt" "$OUT/fix-stash.txt" > "$OUT/diff-pin-stash.txt"; then
  {
  echo "pin==stash: FALLITA (claim equivalenza AP1 falsificato) — diff:"
  sed 's/^/  /' "$OUT/diff-pin-stash.txt"
  echo "ESITO: rc=1"
  } >> "$VERD"; fin 1
fi
echo "pin==stash: BYTE-ID ok" >> "$VERD"

diff "$OUT/fix-oracle.txt" "$OUT/fix-pin.txt" > "$OUT/diff-oracle-pin.txt"
# famiglie DICHIARATE (catalogo §3.21 a/b/c): ogni riga di contenuto del diff
# deve appartenervi; righe di controllo diff (NcN/---) escluse.
UNDECL=$(grep '^[<>]' "$OUT/diff-oracle-pin.txt" | grep -Ev \
  'Illegal offset type|Cannot access offset of type|null as an array offset|null.*array offset|Automatic conversion of false to array|Implicit conversion from float' \
  || true)
{
echo "diff oracle→pin: $(grep -c '^[<>]' "$OUT/diff-oracle-pin.txt") righe marcate (file: sonda-out/diff-oracle-pin.txt)"
echo "--- diff integrale (a verbale) ---"
sed 's/^/  /' "$OUT/diff-oracle-pin.txt"
} >> "$VERD"
if [ -n "$UNDECL" ]; then
  {
  echo "RIGHE NON CATALOGATE (fuori §3.21):"
  echo "$UNDECL" | sed 's/^/  /'
  echo "ESITO: rc=2"
  } >> "$VERD"; fin 2
fi
echo "ESITO: rc=0 (pin==stash byte-id; divergenze oracle tutte nelle famiglie §3.21)" >> "$VERD"
fin 0
