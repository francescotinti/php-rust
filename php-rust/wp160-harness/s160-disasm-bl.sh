#!/bin/bash
# s160-disasm-bl.sh <binA> <binB> <out> — criterio af1 p.8: conteggio bl nel
# simbolo run_loop dei due bracci (stesso formato di ab-out/disasm-am1.out).
set -u
export PATH=/usr/bin:/bin:/usr/sbin
blcount(){ # stampa "sym=<nome> bl=<n>" del primo simbolo run_loop
  otool -tv "$1" | awk '
    /^__ZN.*:$/ { if (done) { insym=0; next }
                  if (index($0, "run_loop") > 0 && !sym) { insym=1; sym=$0; sub(/:$/,"",sym) }
                  else if (sym) { insym=0; done=1 }
                  else insym=0
                  next }
    insym && $2 == "bl" { c++ }
    END { printf "sym=%s bl=%d\n", sym, c }'
}
{ echo "A: $(blcount "$1")"
  echo "B: $(blcount "$2")"
} > "$3"
cat "$3"
