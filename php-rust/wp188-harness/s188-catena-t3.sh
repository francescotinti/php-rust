#!/bin/bash
# s188-catena-t3.sh — S-188 Tappa 2: nella stessa finestra (lock TOKEN s188 scritto dal lanciatore)
# (1) coppia CPU/req t3 con il braccio A (s188-wp-server-pair.sh t3), poi (2) throughput t1
# (s188-throughput.sh t1). Esiti SOLO dai .done dei due harness; catena-out/catena-t3.done riassume.
# PIN_ATTESO = hash del server pin s188 (esportato dall'avvio). Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp188-harness"
OUT="$H/pair-out"; mkdir -p "$OUT"
: "${PIN_ATTESO:?PIN_ATTESO obbligatorio (hash server pin s188)}"
export PIN_ATTESO
/bin/bash "$H/s188-wp-server-pair.sh" t3
P=$(cat "$OUT/pair188-t3.done" 2>/dev/null || echo "rc=99 manca")
/bin/bash "$H/s188-throughput.sh" t1
Q=$(cat "$OUT/tp188-t1.done" 2>/dev/null || echo "rc=99 manca")
echo "pair_t3=[$P] throughput_t1=[$Q] $(date '+%F %T')" > "$OUT/catena-t3.done"
