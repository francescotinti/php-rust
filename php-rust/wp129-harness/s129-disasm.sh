#!/bin/bash
# s129-disasm.sh — disasm mirato seg.3 (criterio p.6, protocollo S-104):
# ins/bl-count dei simboli del PRELUDIO per-statement e della famiglia resolve
# sul pin s127b. Analisi statica, nessuna cifra cronometrica.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
PIN="$HOME/Claude/php-rust-output/release/phpr"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp129-harness"
OUT="$H/s129-disasm-verdetto.out"
PAT="${1:-byref_hook_root|field_lazy_root|indirect_hook_target|lazy_prop_access|resolve_prop_access|prop_hook|hook_guarded|field_set_mode|field_write_prop_step|field_write_walk|pop_field_keys}"
[ "$(shasum -a 256 "$PIN" | cut -c1-16)" = "ccb63dcaf565cffc" ] || { echo "pin!=s127b" > "$OUT"; echo 9 > "$H/disasm.rc"; exit 9; }
{
echo "== s129 disasm seg.3: preludio per-statement + famiglia resolve (pin s127b; protocollo S-104) =="
echo "grade=DISASM  # analisi statica del pin, nessuna cifra cronometrica; rc autoritativo = wp129-harness/disasm.rc"
echo "pattern=$PAT"
T="$H/tempo-out"; mkdir -p "$T"
nm "$PIN" | grep -E "$PAT" | sort > "$T/s129-syms.tmp"
nm -n "$PIN" | awk '$1 ~ /^[0-9a-f]{8,}$/ {print $1}' > "$T/s129-all.tmp"
while read -r addr _t name; do
  next=$(awk -v a="$addr" '($1 "") > (a "") {print $1; exit}' "$T/s129-all.tmp")
  [ -n "$next" ] || continue
  n_ins=$(objdump -d --start-address=0x$addr --stop-address=0x$next "$PIN" 2>/dev/null | grep -cE "^\s*[0-9a-f]+:")
  n_bl=$(objdump -d --start-address=0x$addr --stop-address=0x$next "$PIN" 2>/dev/null | grep -cE "\bbl\s")
  short=$(echo "$name" | sed 's/17h[0-9a-f]*E$//')
  echo "sym=$short ins=$n_ins bl=$n_bl"
done < "$T/s129-syms.tmp"
rm -f "$T/s129-syms.tmp" "$T/s129-all.tmp"
} > "$OUT" 2>&1
echo 0 > "$H/disasm.rc"
cat "$OUT"
