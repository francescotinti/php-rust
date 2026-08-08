#!/bin/bash
# pin-phpr.sh <tag> — REGOLE.md §2: lo stash del pin phpr nasce SOLO da qui.
# RICETTA BUILD (pipeline A′, S-117): [profile.release] lto="fat"+codegen-units=1
# (Cargo.toml di root) + `SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build
# --release` — determinismo provato build ×2 hash IDENTICO (s117-criterio §2/§2-bis).
# Smoke di parità VERO (giudice arith vs oracle, entrambi i modi) + stash +
# verbale a stdout. La batteria/corpus/micro restano i gate del protocollo
# PIN (REGOLE §6): questo script garantisce che NESSUN binario non-eseguito
# venga mai stashato.
set -euo pipefail
BIN="$HOME/Claude/php-rust-output/release/phpr"
ORACLE=/opt/homebrew/opt/php/bin/php
MICRO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp97-harness/micro/arith_small.php"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
TAG="${1:?uso: pin-phpr.sh <tag, es. s107>}"

H=$(shasum -a 256 "$BIN" | cut -c1-16)
EXP=$("$ORACLE" "$MICRO")
ON=$(env -u PHPR_REG_LOWER "$BIN" "$MICRO")
OFF=$(PHPR_REG_LOWER=0 "$BIN" "$MICRO")
if [ "$ON" != "$EXP" ] || [ "$OFF" != "$EXP" ]; then
  echo "SMOKE FALLITO: oracle='$EXP' on='$ON' off='$OFF' => NIENTE stash."; exit 1
fi
cp "$BIN" "$STASH/phpr-$TAG"
HS=$(shasum -a 256 "$STASH/phpr-$TAG" | cut -c1-16)
[ "$H" = "$HS" ] || { echo "hash dello stash diverso ($H vs $HS)"; exit 1; }
echo "PIN phpr $TAG = $H (smoke parità 2 modi OK, stash phpr-$TAG). I gate del protocollo (batteria/corpus/fixture/micro) restano dovuti."
