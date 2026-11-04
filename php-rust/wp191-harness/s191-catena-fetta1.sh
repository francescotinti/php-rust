#!/bin/bash
# s191-catena-fetta1.sh — S-191 coppia server s190 (AXnew, canonica) vs s189 (AXold, stash), tentativo t3, nella finestra
# della notte (lock TOKEN s191). COPIA DICHIARATA di ../wp189-harness/s189-catena-fetta1.sh (manifest
# s191-catena-fetta1-copia.diff, generatore gen-notte-s191.py) coi SOLI adattamenti: harness wp191, PIN_OLD default s189,
# script s191-*, done ga191/tw191, exit col massimo dei due rc (prima usciva sempre 0). Testo S-189 sotto:
# s189-catena-fetta1.sh — S-189 FETTA 1 nella stessa finestra (lock TOKEN s189 scritto dal lanciatore):
# (1) guardia throughput AXold/AXnew (s189-guardia-ax.sh t1), poi (2) tripwire W=1 (s189-tripwire-w1.sh t1).
# Esiti SOLO dai .done dei due harness; pair-out/catena-fetta1.done riassume. PIN_ATTESO = hash del server pin
# s189 (esportato dall'avvio), PIN_OLD = s188. Pattern di s188-catena-t3.sh. Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp191-harness"
OUT="$H/pair-out"; mkdir -p "$OUT"
: "${PIN_ATTESO:?PIN_ATTESO obbligatorio (hash server pin s189)}"
export PIN_ATTESO PIN_OLD="${PIN_OLD:-80eea401b71c2be7}"
T="${T:-t1}"
/bin/bash "$H/s191-guardia-ax.sh" "$T"
G=$(cat "$OUT/ga191-$T.done" 2>/dev/null || echo "rc=99 manca")
/bin/bash "$H/s191-tripwire-w1.sh" "$T"
W=$(cat "$OUT/tw191-$T.done" 2>/dev/null || echo "rc=99 manca")
echo "guardia_$T=[$G] tripwire_$T=[$W] $(date '+%F %T')" > "$OUT/catena-fetta1.done"
rg=$(printf '%s' "$G" | sed -n 's/^rc=\([0-9]*\).*/\1/p'); rw=$(printf '%s' "$W" | sed -n 's/^rc=\([0-9]*\).*/\1/p')
RC=${rg:-99}; [ "${rw:-99}" -gt "$RC" ] && RC=${rw:-99}; exit "$RC"
