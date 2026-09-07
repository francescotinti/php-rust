#!/bin/bash
# s172-disasm.sh <bin>… — conteggio `bl` dentro run_loop per ogni binario (guardia p.5:
# lezione WP-104 «ogni leva su run_loop pretende il disasm prima/dopo»). Stesso metodo
# di S-171 (ab-out/disasm-sl1.out): simbolo run_loop da nm, disassembly del solo simbolo
# con llvm-objdump, conteggio delle istruzioni `bl`. Esito su stdout, una riga per binario.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
for BIN in "$@"; do
  SYM=$(nm "$BIN" | awk '/run_loop/ && /Vm/ {print $3; exit}')
  [ -n "$SYM" ] || { echo "$BIN: simbolo run_loop ASSENTE"; continue; }
  N=$(objdump --disassemble-symbols="$SYM" "$BIN" 2>/dev/null | grep -cw "bl" | tr -d ' ')
  echo "$(basename "$BIN") $(shasum -a 256 "$BIN" | cut -c1-8): sym=$SYM bl=$N"
done
