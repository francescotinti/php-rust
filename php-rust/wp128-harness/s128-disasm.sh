#!/bin/bash
# s128-disasm.sh — disasm «prima» seg.2 (criterio p.3, protocollo S-104):
# bl-count dei simboli del sentiero insert-su-proprieta' sul pin s127b.
# Analisi statica, nessuna cifra cronometrica. Output macchina.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
PIN="$HOME/Claude/php-rust-output/release/phpr"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp128-harness"
OUT="$H/s128-disasm-verdetto.out"
[ "$(shasum -a 256 "$PIN" | cut -c1-16)" = "ccb63dcaf565cffc" ] || { echo "pin!=s127b" > "$OUT"; echo 9 > "$H/disasm.rc"; exit 9; }
{
echo "== s128 disasm seg.2: sentiero insert-su-proprieta' (pin s127b ccb63dca; protocollo S-104) =="
echo "grade=DISASM  # analisi statica del pin, nessuna cifra cronometrica; rc autoritativo = wp128-harness/disasm.rc"
T="$H/admission-out"; mkdir -p "$T"
nm "$PIN" | grep -E "field_write|to_hashed|KeyIndex|PhpArray6insert" | sort > "$T/s128-syms.tmp"
# range simbolico: da addr(sym) all'addr successivo nella mappa nm globale
nm -n "$PIN" | awk '$1 ~ /^[0-9a-f]{8,}$/ {print $1}' > "$T/s128-all.tmp"
while read -r addr _t name; do
  # confronto FORZATO a stringa: con hex tutto-cifre awk confronterebbe in
  # numerico e "…e18" diventa notazione scientifica (morso S-128)
  next=$(awk -v a="$addr" '($1 "") > (a "") {print $1; exit}' "$T/s128-all.tmp")
  [ -n "$next" ] || continue
  n_ins=$(objdump -d --start-address=0x$addr --stop-address=0x$next "$PIN" 2>/dev/null | grep -cE "^\s*[0-9a-f]+:")
  n_bl=$(objdump -d --start-address=0x$addr --stop-address=0x$next "$PIN" 2>/dev/null | grep -cE "\bbl\s")
  short=$(echo "$name" | sed 's/17h[0-9a-f]*E$//' )
  echo "sym=$short ins=$n_ins bl=$n_bl"
done < "$T/s128-syms.tmp"
rm -f "$T/s128-syms.tmp" "$T/s128-all.tmp"
} > "$OUT" 2>&1
echo 0 > "$H/disasm.rc"
cat "$OUT"
