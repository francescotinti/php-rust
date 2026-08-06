#!/bin/bash
# s106-grado-chain.sh — chain SEQUENZIALE off→on del grado PIENO php-server
# (S-106 punto 2). Da lanciare DETACHED col daemonizer (wp57).
#
# Recepisce A-PE-107-2 (chain v2): pre-flight PRIMA della prima gamba
# (ping MySQL con elenco database, STATE uploads-guard assente, spazio
# disco), restore tentato su rc gamba ≠0 (belt oltre al trap del
# launcher), marker .done con rc di ENTRAMBE le gambe.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin

H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp106-harness"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
STATE="/Volumes/Extreme Pro/Claude/uploads-backups/current"
MARK="$H/grado-chain"
mkdir -p "$MARK"
log() { echo "== $(date +%H:%M:%S) [S106-GRADO-CHAIN] $1"; }

# ---- pre-flight (A-PE-107-2) ----
DBS="$(mysql --socket=/private/tmp/mysql-wp8.sock -uroot -N -e 'SHOW DATABASES' 2>/dev/null)"
for db in wp wptests; do
  echo "$DBS" | grep -qx "$db" || { log "ABORT pre-flight: database $db assente (MySQL wp8 giù o datadir sbagliato)"; echo "rc_off=70 rc_on=70 $(date +%T)" > "$MARK/grado-chain.done"; exit 70; }
done
if [ -e "$STATE" ]; then
  log "ABORT pre-flight: uploads-guard STATE pendente ($(cat "$STATE"))"; echo "rc_off=71 rc_on=71 $(date +%T)" > "$MARK/grado-chain.done"; exit 71
fi
FREE_G="$(df -g '/Volumes/Extreme Pro' | tail -1 | awk '{print $4}')"
[ "$FREE_G" -ge 10 ] || { log "ABORT pre-flight: <10G liberi su Extreme Pro"; echo "rc_off=72 rc_on=72 $(date +%T)" > "$MARK/grado-chain.done"; exit 72; }
log "pre-flight OK (MySQL wp8 su, STATE assente, ${FREE_G}G liberi)"

# ---- gamba OFF ----
log "gamba OFF: via"
/bin/bash "$H/s106-grado-server.sh" off
RC_OFF=$?
log "gamba OFF: rc=$RC_OFF"
if [ "$RC_OFF" != 0 ]; then
  log "gamba OFF rossa: tento restore uploads (belt A-PE-107-2)"
  "$GUARD" restore || true
fi

# ---- gamba ON (si esegue comunque: pattern pair105, l'esito è per-gamba) ----
log "gamba ON: via"
/bin/bash "$H/s106-grado-server.sh" on
RC_ON=$?
log "gamba ON: rc=$RC_ON"
if [ "$RC_ON" != 0 ]; then
  log "gamba ON rossa: tento restore uploads (belt A-PE-107-2)"
  "$GUARD" restore || true
fi

echo "rc_off=$RC_OFF rc_on=$RC_ON $(date +%T)" > "$MARK/grado-chain.done"
log "CHAIN DONE rc_off=$RC_OFF rc_on=$RC_ON"
exit 0
