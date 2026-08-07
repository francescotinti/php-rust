#!/bin/bash
# pin-server.sh <tag> — REGOLE.md §2: il pin server nasce SOLO da qui.
# Build con ricetta + SMOKE VERO (--axum serve una pagina) + stash + registro,
# in un atto. Smoke fallito => niente stash (la classe de67cb64 muore qui).
set -euo pipefail
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
BIN="$HOME/Claude/php-rust-output/release/php-server"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
TAG="${1:?uso: pin-server.sh <tag, es. s107>}"

cd "$REPO"
PHPR_PRE=$(shasum -a 256 "$PHPR" | cut -c1-16)
cargo build --release -p php-server --features axum-server
H=$(shasum -a 256 "$BIN" | cut -c1-16)
PHPR_POST=$(shasum -a 256 "$PHPR" | cut -c1-16)
NOTE_PHPR="pin phpr INVARIATO ($PHPR_POST)"
[ "$PHPR_PRE" = "$PHPR_POST" ] || NOTE_PHPR="ATTENZIONE: build ha toccato phpr $PHPR_PRE->$PHPR_POST"

# ---- SMOKE: o serve davvero via --axum, o non esiste alcun pin ----
D=$(mktemp -d)
echo '<?php echo "SMOKE-OK";' > "$D/s.php"
"$BIN" --axum --workers 1 --port 8299 -t "$D" >/dev/null 2>"$D/srv.log" &
SRV=$!
sleep 2
BODY=$(curl -s -m 5 http://127.0.0.1:8299/s.php || true)
kill "$SRV" 2>/dev/null || true; pkill -f "port 8299" 2>/dev/null || true
if [ "$BODY" != "SMOKE-OK" ]; then
  echo "SMOKE FALLITO (body='$BODY'; srv.log:)"; head -3 "$D/srv.log" || true
  rm -rf "$D"; echo "=> NIENTE stash, NIENTE registro."; exit 1
fi
rm -rf "$D"

cp "$BIN" "$STASH/php-server-$TAG"
HS=$(shasum -a 256 "$STASH/php-server-$TAG" | cut -c1-16)
[ "$H" = "$HS" ] || { echo "hash dello stash diverso ($H vs $HS)"; exit 1; }
HEAD_SHA=$(git rev-parse --short HEAD)
echo "| $H | $TAG | \`cargo build --release -p php-server --features axum-server\` @ $HEAD_SHA | smoke --axum OK $(date '+%F %T'); $NOTE_PHPR; GRADO PIENO a parte (s106-grado-server.sh) | stash \`php-server-$TAG\` |" >> PIN_REGISTRY.md
echo "PIN server $TAG = $H @ $HEAD_SHA (smoke OK, stashato, registrato). $NOTE_PHPR"
echo "NB: questo e' il pin COLLAUDATO AL MINIMO; le cifre server esigono il grado pieno."
