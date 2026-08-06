#!/bin/bash
# s105-admission-disasm.sh <binario> <label> — admission-disasm della leva
# H-D args (KS-GR-106-2): taglia di run_loop + bl-count totale + bl verso
# i simboli alloc/free. Confronto PRIMA/DOPO: se l'inliner flippa
# (variazioni oltre i siti toccati) nessuna componente di costo si cita
# (KS-BA-106-1). Output ascii su stdout; il .dis raw resta in scratch.
# niente pipefail: gli awk con `exit` a valle di nm generano SIGPIPE voluti
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
BIN="$1"; LABEL="$2"
SCRATCH="${SCRATCH:-/private/tmp/claude-501/-Volumes-Extreme-Pro-Claude-php-8-5-7/6a6f6214-135d-45a7-9bbb-0e89620c9ede/scratchpad}"
mkdir -p "$SCRATCH"
DIS="$SCRATCH/runloop-$LABEL.dis"

SYM=$(nm "$BIN" | awk '/8run_loop17h[0-9a-f]+E$/{print; exit}')
ADDR=$(echo "$SYM" | awk '{print $1}')
NAME=$(echo "$SYM" | awk '{print $3}')
# taglia = simbolo successivo nello stesso segmento
NEXT=$(nm -n "$BIN" | awk -v a="$ADDR" 'f{print $1; exit} $1==a{f=1}')
SIZE=$(( 0x$NEXT - 0x$ADDR ))

echo "label=$LABEL bin=$(shasum -a 256 "$BIN" | cut -c1-16)"
echo "run_loop_addr=0x$ADDR run_loop_size_B=$SIZE"

# disasm della sola run_loop (llvm-objdump del toolchain di sistema)
objdump -d --disassemble-symbols="$NAME" "$BIN" > "$DIS" 2>/dev/null || \
  objdump -d "$BIN" | awk -v s="$NAME" 'index($0,s".:"){f=1} f&&/^$/{exit} f' > "$DIS"

TOT_BL=$(grep -cE '\bbl\b' "$DIS" || true)
echo "bl_totali=$TOT_BL"
echo "bl_alloc_free (target con malloc/free/alloc nel nome, mangled):"
grep -E '\bbl\b' "$DIS" | grep -oE '<[^>]+>' - | sort | uniq -c | sort -rn | grep -iE 'malloc|free|alloc|grow|reserve' - | head -12 || echo "  (nessuno)"
echo "top-12 target bl:"
grep -E '\bbl\b' "$DIS" | grep -oE '<[^>]+>' - | sort | uniq -c | sort -rn | head -12
echo "raw=$DIS"
