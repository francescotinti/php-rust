#!/bin/bash
# pair94.sh — S-94.0 OGGETTO #1: la COPPIA FULL stessa-sera, oracle vs phpr.
#
# Perche esiste: il contatore full/media e fermo a WP-85 (otto sessioni) e il
# Concilio WP-95 ha eletto questa coppia a PRIMA misura di S-94.0 — nessuna
# leva footprint prima di un «prima» fresco (legge WP-48: la predizione si
# misura contro una baseline della stessa sera, non contro una citazione).
#
# Metodo (invariato da GAP_TREND §Metodo, mai reinventato qui):
#   1. media group: 1 run oracle + 1 run phpr, /usr/bin/time -l, DB reset +
#      uploads azzerati PRIMA DI OGNI run, MIMALLOC_PURGE_DELAY=0
#      -> rapporti user CPU e peak footprint
#   2. full suite: stesse condizioni, master-CPU (user+sys) e peak footprint
# Ordine: oracle PRIMA in entrambe le gambe (il riferimento non si scalda
# sulla cache del misurato).
# Uploads: SEMPRE dalla guardia Gregg R7, mai un wipe a mano (WP-62).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
WPDEV="/Users/francescotinti/Claude/wpdev"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp94-harness"
OUT="$H/pair-out"; mkdir -p "$OUT"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
rm -f "$OUT/pair94.done"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
reset_env() {
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || exit 3
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
}
# identita dei misurati, IN BANDA (A-PP-77: un pin dichiarato non e un pin)
{
  echo "phpr=$(shasum -a 256 "$PHPR" | cut -c1-16)"
  echo "oracle=$("$ORACLE" -v | head -1)"
  echo "rustc=$(cd "$H/.." && rustc -Vv | tr '\n' ';')"
  echo "head=$(git -C "$H/.." rev-parse --short=12 HEAD)"
  echo "epoch=$(date +%s)"
} > "$OUT/pair94.identity"
"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=4 guard-backup" > "$OUT/pair94.done"; exit 4; }
cd "$WPDEV" || exit 2

step "media oracle"
reset_env
MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$ORACLE" vendor/bin/phpunit --group media \
  > "$OUT/media-oracle.txt" 2> "$OUT/media-oracle.time"
step "media oracle rc=$?"
step "media phpr"
reset_env
MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$PHPR" vendor/bin/phpunit --group media \
  > "$OUT/media-phpr.txt" 2> "$OUT/media-phpr.time"
step "media phpr rc=$?"

step "full oracle"
reset_env
MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$ORACLE" vendor/bin/phpunit \
  > "$OUT/full-oracle.txt" 2> "$OUT/full-oracle.time"
step "full oracle rc=$?"
step "full phpr"
reset_env
MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$PHPR" vendor/bin/phpunit \
  > "$OUT/full-phpr.txt" 2> "$OUT/full-phpr.time"
step "full phpr rc=$?"

find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$OUT/progress.txt" 2>&1
step "pair94 DONE"
echo "rc=0 $(date +%T)" > "$OUT/pair94.done"
