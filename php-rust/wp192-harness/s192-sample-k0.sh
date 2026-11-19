#!/bin/bash
# s192-sample-k0.sh <label> <binario> — S-192: campionamento (macOS `sample`, 4 s a 1 ms) del driver ret-buf-dq-k0 su un binario,
# per VEDERE dove sta il costo per Ret (strumento d'indagine, non misura). Scrive out/sample-<label>.txt e stampa le righe del
# riepilogo «Sort by top of stack» (self time per simbolo). Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; O="$P/wp192-harness/out"; mkdir -p "$O"
L="${1:?label}"; B="${2:?binario}"
"$B" "$P/wp186-harness/ret-buf-dq-k0.php" > /dev/null 2>&1 &
PID=$!
sleep 1
sample "$PID" 4 1 -file "$O/sample-$L.txt" > /dev/null 2>&1
kill "$PID" 2>/dev/null; wait "$PID" 2>/dev/null
awk '/Sort by top of stack/{f=1} f' "$O/sample-$L.txt" | sed -n '2,22p' | cut -c1-150
