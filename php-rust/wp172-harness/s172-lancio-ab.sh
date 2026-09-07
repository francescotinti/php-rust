#!/bin/bash
# s172-lancio-ab.sh <script> <args…> — lanciatore: attende la QUIETE (gate separato
# s129-quiescenza.sh, retry fino a 30×60 s come la catena di promozione s163 p.«quiescenza»;
# lezione S-170: il flare di mediaanalysisd/rust-analyzer NON si aspetta a mano) e POI
# esegue lo script di misura con i suoi argomenti. Plumbing nuovo S-171 (dichiarato), niente
# giudizio: l'rc resta quello dello script lanciato (ab-out/<TAG>.rc). Log in ab-out/lancio-<TAG>.log.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
SCRIPT="${1:?script}"; shift
OUT="$H/ab-out"; mkdir -p "$OUT"
TAG="${@: -2:1}"; LOG="$OUT/lancio-$TAG.log"
echo "$(date '+%F %T') attesa quiete per $SCRIPT ($*)" >> "$LOG"
QOK=1
for t in $(seq 1 30); do
  if "$H/../wp129-harness/s129-quiescenza.sh" "$OUT/quiesce-lancio-$TAG.rc" > "$OUT/quiesce-lancio-$TAG.log" 2>&1; then
    QOK=0; echo "$(date '+%F %T') quiete PASS al tentativo $t: $(tail -1 "$OUT/quiesce-lancio-$TAG.log")" >> "$LOG"; break
  fi
  echo "$(date '+%F %T') tentativo $t: $(tail -1 "$OUT/quiesce-lancio-$TAG.log")" >> "$LOG"
  sleep 60
done
[ "$QOK" = 0 ] || { echo "$(date '+%F %T') quiete MAI PASS in 30 tentativi — script NON lanciato" >> "$LOG"; exit 8; }
exec /bin/bash "$H/$SCRIPT" "$@"
