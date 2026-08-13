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
# RICETTA A′ (S-118): profilo lto=fat+cgu=1 dal Cargo.toml di root +
# SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 (determinismo provato S-117).
PHPR_PRE=$(shasum -a 256 "$PHPR" | cut -c1-16)
SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release -p php-server --features axum-server
H=$(shasum -a 256 "$BIN" | cut -c1-16)
PHPR_POST=$(shasum -a 256 "$PHPR" | cut -c1-16)
NOTE_PHPR="pin phpr INVARIATO ($PHPR_POST)"
if [ "$PHPR_PRE" != "$PHPR_POST" ]; then
  # churn del release/: si sana con build ricetta; il determinismo A′ esige
  # il ritorno AL BYTE all'hash pre-build, pena STOP (criterio R3 gate 1).
  SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release
  PHPR_POST=$(shasum -a 256 "$PHPR" | cut -c1-16)
  if [ "$PHPR_PRE" = "$PHPR_POST" ]; then
    NOTE_PHPR="churn phpr SANATO con build ricetta (torna $PHPR_POST al byte)"
  else
    echo "STOP: phpr churnato $PHPR_PRE->$PHPR_POST e la build ricetta NON riproduce H1 => niente pin."
    exit 1
  fi
fi

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

# Guardia NO-CLOBBER (incidente S-133, stessa forma di pin-phpr.sh): uno
# stash ESISTENTE con hash DIVERSO non si sovrascrive mai in silenzio;
# sostituzione consapevole = PIN_FORCE=1 dichiarato a verbale.
if [ -e "$STASH/php-server-$TAG" ] && [ "${PIN_FORCE:-0}" != 1 ]; then
  HOLD=$(shasum -a 256 "$STASH/php-server-$TAG" | cut -c1-16)
  if [ "$HOLD" != "$H" ]; then
    echo "STOP NO-CLOBBER: stash php-server-$TAG ESISTE con hash DIVERSO ($HOLD != $H) — tag sbagliato o rotazione non dichiarata; sovrascrivere solo con PIN_FORCE=1."
    exit 1
  fi
fi
cp "$BIN" "$STASH/php-server-$TAG"
HS=$(shasum -a 256 "$STASH/php-server-$TAG" | cut -c1-16)
[ "$H" = "$HS" ] || { echo "hash dello stash diverso ($H vs $HS)"; exit 1; }
HEAD_SHA=$(git rev-parse --short HEAD)
echo "| $H | $TAG | \`cargo build --release -p php-server --features axum-server\` @ $HEAD_SHA | smoke --axum OK $(date '+%F %T'); $NOTE_PHPR; GRADO PIENO a parte (s106-grado-server.sh) | stash \`php-server-$TAG\` |" >> PIN_REGISTRY.md
# Az.rev. S-130 #1: il registro si COMMITTA nello stesso atto e il tree deve
# restare pulito su PIN_REGISTRY.md, pena STOP (registro senza evidenza = niente pin).
MSG=$(mktemp); echo "pin server $TAG: registro PIN_REGISTRY ($H)" > "$MSG"
git add PIN_REGISTRY.md && git commit -F "$MSG" >/dev/null && git push >/dev/null
rm -f "$MSG"
if [ -n "$(git status --porcelain PIN_REGISTRY.md)" ]; then
  echo "STOP: PIN_REGISTRY.md ancora sporco dopo il commit dell'atto."; exit 1
fi
echo "PIN server $TAG = $H @ $HEAD_SHA (smoke OK, stashato, registrato+committato). $NOTE_PHPR"
echo "NB: questo e' il pin COLLAUDATO AL MINIMO; le cifre server esigono il grado pieno."
