#!/bin/bash
# s189-catena-fetta1.sh — S-189 FETTA 1 nella stessa finestra (lock TOKEN s189 scritto dal lanciatore):
# (1) guardia throughput AXold/AXnew (s189-guardia-ax.sh t1), poi (2) tripwire W=1 (s189-tripwire-w1.sh t1).
# Esiti SOLO dai .done dei due harness; pair-out/catena-fetta1.done riassume. PIN_ATTESO = hash del server pin
# s189 (esportato dall'avvio), PIN_OLD = s188. Pattern di s188-catena-t3.sh. Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp189-harness"
OUT="$H/pair-out"; mkdir -p "$OUT"
: "${PIN_ATTESO:?PIN_ATTESO obbligatorio (hash server pin s189)}"
export PIN_ATTESO PIN_OLD="${PIN_OLD:-97b58fd225ca7289}"
T="${T:-t1}"
/bin/bash "$H/s189-guardia-ax.sh" "$T"
G=$(cat "$OUT/ga189-$T.done" 2>/dev/null || echo "rc=99 manca")
/bin/bash "$H/s189-tripwire-w1.sh" "$T"
W=$(cat "$OUT/tw189-$T.done" 2>/dev/null || echo "rc=99 manca")
echo "guardia_$T=[$G] tripwire_$T=[$W] $(date '+%F %T')" > "$OUT/catena-fetta1.done"
