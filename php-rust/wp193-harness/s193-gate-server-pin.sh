#!/bin/bash
# s193-gate-server-pin.sh <tag> — S-193 (NEXT 2a): i due gate FUNZIONALI del server ripetuti sul server PINNATO s191
# (canonica, hash atteso c413398a44f9b0a4), dopo la promozione s185. Stessi due script di wp192-harness/s192-gate-server.sh,
# INVARIATI e con le stesse variabili d'ambiente; qui NON si costruisce nulla: il binario è quello della canonica.
# Attende la quiete della CI (ultimo evento DONE, nessun cargo). Nessun tempo. Esito: s193-gate-server-pin-<tag>.out e
# out/gate-server-pin-<tag>.rc (0 = entrambi verdi). Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp193-harness"; O="$H/out"; mkdir -p "$O"
TAG="${1:?tag}"; SRV="$HOME/Claude/php-rust-output/release/php-server"; ATTESO="c413398a44f9b0a4"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"; FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
V="$H/s193-gate-server-pin-$TAG.out"; : > "$V"; rm -f "$O/gate-server-pin-$TAG.rc"; RC=0
note(){ echo "$1" >> "$V"; }
fin(){ echo "$1" > "$O/gate-server-pin-$TAG.rc"; exit "$1"; }
until tail -1 "$FEED" | grep -q '^DONE' && ! pgrep -x cargo >/dev/null; do sleep 30; done
HS=$(shasum -a 256 "$SRV" | cut -c1-16)
[ "$HS" = "$ATTESO" ] || { note "server canonico $HS != s191 $ATTESO — STOP"; fin 9; }
note "== s193 gate server sul PIN s191 tag=$TAG php-server=$HS Data=$(df -g /System/Volumes/Data | awk 'NR==2{print $4}')G $(date '+%F %T') =="
OLD="$STASH/php-server-s188"
if [ -s "$OLD" ]; then
  OLD_BIN="$OLD" NEW_BIN="$SRV" PIN_ATTESO="$(shasum -a 256 "$OLD" | cut -c1-16)" bash "$P/wp189-harness/gate-loopback/gate-loopback.sh" > "$O/gate-server-pin-$TAG-loopback.log" 2>&1; r=$?
else
  ARMS="new3 new1 oracle3 oracle1" NEW_BIN="$SRV" bash "$P/wp189-harness/gate-loopback/gate-loopback.sh" > "$O/gate-server-pin-$TAG-loopback.log" 2>&1; r=$?
fi
note "gate-loopback: rc=$r · $(tail -1 "$P/wp189-harness/gate-loopback/out/gate_loopback.verdict" 2>/dev/null | cut -c1-200)"; [ "$r" = 0 ] || RC=1
PHPSERVER="$SRV" OUT="$O/battery61-axum-$TAG" bash "$P/wp188-harness/battery61-axum.sh" > "$O/gate-server-pin-$TAG-battery61.log" 2>&1; r=$?
note "battery61-axum: rc=$r · $(tail -3 "$O/gate-server-pin-$TAG-battery61.log" | tr '\n' ' ' | cut -c1-260)"; [ "$r" = 0 ] || RC=1
note "esito rc=$RC $(date '+%F %T')"; fin "$RC"
