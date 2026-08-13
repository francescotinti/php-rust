#!/bin/bash
# copia-gate.sh <template> <copia> <manifest> — dente di collaudo delle COPIE
# DICHIARATE (az.rev. S-133 #3). Il manifest è il `diff template copia` ATTESO:
# elenca per NOME (riga e contenuto) TUTTE le divergenze dichiarate, righe
# quotate e commenti INCLUSI. rc=0 solo se il diff reale coincide AL BYTE col
# manifest: una riga che DOVEVA divergere e non diverge (la classe del sed
# cieco S-133) esce come discrepanza, come ogni divergenza non dichiarata.
set -u
T="${1:?uso: copia-gate.sh <template> <copia> <manifest-diff-atteso>}"
C="${2:?uso: copia-gate.sh <template> <copia> <manifest-diff-atteso>}"
M="${3:?uso: copia-gate.sh <template> <copia> <manifest-diff-atteso>}"
for f in "$T" "$C" "$M"; do
  [ -f "$f" ] || { echo "COPIA-GATE FILE MANCANTE: $f"; echo "COPIA-GATE rc=9"; exit 9; }
done
D=$(mktemp)
diff "$T" "$C" > "$D"
if diff "$M" "$D" > /dev/null; then
  echo "COPIA-GATE ok: divergenze reali == manifest ($(grep -c '^[<>]' "$M") righe dichiarate)"
  rm -f "$D"
  echo "COPIA-GATE rc=0"
  exit 0
fi
echo "COPIA-GATE DIFFORME: il diff reale template->copia NON coincide col manifest."
echo "--- righe nel manifest ma NON nel diff reale (dichiarate e MAI avvenute: classe sed-cieco) / viceversa ---"
diff "$M" "$D" | sed 's/^/  /'
rm -f "$D"
echo "COPIA-GATE rc=1"
exit 1
