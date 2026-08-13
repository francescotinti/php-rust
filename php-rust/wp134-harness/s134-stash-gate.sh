#!/bin/bash
# s134-stash-gate.sh — gate `stash` (az.rev. S-133 #2): ri-hash dei QUATTRO
# binari pinnati (corrente+precedente, phpr E server) contro PIN_REGISTRY.md.
# Compone con la guardia NO-CLOBBER dei pin-*.sh: quella impedisce la
# sovrascrittura nell'atto del pin, questo verifica A POSTERIORI che gli stash
# su disco siano ancora i byte registrati (la classe dell'incidente S-133 —
# stash storico sovrascritto — muore qui anche se entrata per altra via).
# rc = numero di difformità; parse fail-closed (2+2 righe o rc!=0).
set -u
REG="${STASH_GATE_REG:-/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/PIN_REGISTRY.md}"
STASH="${STASH_GATE_DIR:-/Volumes/Extreme Pro/Claude/phpr-old-target/release}"

# Estrae dalle due tabelle le prime 2 righe dato (hash col.2 + nome stash in
# backtick). Output: "hash nome-stash", 4 righe attese (server prima, poi phpr).
PAIRS=$(awk '
  /^## php-server/ { sez="server"; n=0; next }
  /^## phpr/       { sez="phpr";   n=0; next }
  sez != "" && /^\| [0-9a-f]{16} \|/ && n < 2 {
    split($0, c, "|"); gsub(/ /, "", c[2])
    if (match($0, /stash `(php-server|phpr)-[a-zA-Z0-9-]+`/)) {
      s = substr($0, RSTART, RLENGTH)
      gsub(/stash `|`/, "", s)
      print c[2], s; n++
    }
  }
' "$REG")

N=$(printf '%s\n' "$PAIRS" | grep -c .)
if [ "$N" -ne 4 ]; then
  echo "STASH-GATE PARSE FALLITO: attese 4 coppie hash+stash (2 server + 2 phpr), trovate $N"
  printf '%s\n' "$PAIRS"
  echo "STASH-GATE rc=9"
  exit 9
fi

RC=0
while read -r EXP NAME; do
  F="$STASH/$NAME"
  if [ ! -f "$F" ]; then
    echo "STASH-GATE MANCANTE: $NAME (atteso $EXP) non esiste in $STASH"
    RC=$((RC + 1)); continue
  fi
  GOT=$(shasum -a 256 "$F" | cut -c1-16)
  if [ "$GOT" = "$EXP" ]; then
    echo "STASH-GATE ok: $NAME = $EXP"
  else
    echo "STASH-GATE DIFFORME: $NAME su disco $GOT != registro $EXP"
    RC=$((RC + 1))
  fi
done <<< "$PAIRS"

echo "STASH-GATE rc=$RC (0 = 4/4 conformi al registro)"
exit "$RC"
