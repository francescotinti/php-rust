#!/bin/bash
# s139-chain.sh — CATENA S-139: attende la fine della coppia WP
# (pair-out/pair139-t1.done) e lancia la rimisura dbal+ORM (s139-rimisura.sh)
# in SEQUENZA (run pesanti mai in parallelo, REGOLE §10). Il lock misura resta
# della SESSIONE: la catena non lo tocca. Poll 60 s, tetto 6 h; l'esito della
# coppia viene solo LOGGATO (la rimisura è indipendente dal suo rc).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp139-harness"
MARK="$H/pair-out/pair139-t1.done"
LOG="$H/chain-progress.txt"
p(){ echo "$(date +%H:%M:%S) $1" >> "$LOG"; }
: > "$LOG"
rm -f "$H/rimisura-out/rimisura.done"
i=0
while [ ! -e "$MARK" ]; do
  i=$((i+1))
  [ "$i" -gt 360 ] && { p "TIMEOUT 6h: coppia mai conclusa, rimisura NON lanciata"; exit 1; }
  /bin/sleep 60
done
p "coppia conclusa: $(cat "$MARK")"
# quiete post-coppia prima della rimisura (stessa scala del gate: ~1 min)
/bin/sleep 60
mkdir -p /private/tmp/s139-mappa-sp
p "rimisura START (MAPPA_SP=/private/tmp/s139-mappa-sp)"
MAPPA_SP=/private/tmp/s139-mappa-sp "$H/s139-rimisura.sh"
rc=$?
p "rimisura rc=$rc (fonte esito: rimisura-out/rimisura.done)"
exit "$rc"
