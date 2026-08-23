#!/bin/bash
# pin-phpr.sh <tag> — REGOLE.md §2: lo stash del pin phpr nasce SOLO da qui.
# RICETTA BUILD (pipeline A′, S-117): [profile.release] lto="fat"+codegen-units=1
# (Cargo.toml di root) + `SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build
# --release` — determinismo provato build ×2 hash IDENTICO (s117-criterio §2/§2-bis).
# Smoke di parità VERO (giudice arith vs oracle, entrambi i modi) + stash +
# verbale a stdout. La batteria/corpus/micro restano i gate del protocollo
# PIN (REGOLE §6): questo script garantisce che NESSUN binario non-eseguito
# venga mai stashato.
# GATE CORPUS CANONICO (az. rev. S-117): `scripts/corpus-gate.sh <runner> <out>`
# giudica NOMI (congelato s109) E CONTENUTO (golden digest) E off↔on — ogni
# pin/promozione lo usa al posto del confronto solo-nomi.
set -euo pipefail
BIN="$HOME/Claude/php-rust-output/release/phpr"
ORACLE=/opt/homebrew/opt/php/bin/php
MICRO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp97-harness/micro/arith_small.php"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
# MODALITÀ BRACCIO/CANDIDATO (az.rev. S-154 #4): `pin-phpr.sh --braccio <tag>
# <binario>` — stesso atto unico (smoke parità 2 modi + no-clobber + stash +
# hash-verify + registro) su un binario di BRACCIO (gemello/candidato da
# target dedicato). La riga a registro è marcata BRACCIO (non pin): i gate
# del protocollo PIN non sono implicati.
MODE=pin
if [ "${1:-}" = "--braccio" ]; then
  MODE=braccio; shift
  TAG="${1:?uso: pin-phpr.sh --braccio <tag> <binario>}"
  BIN="${2:?uso: pin-phpr.sh --braccio <tag> <binario>}"
  [ -f "$BIN" ] || { echo "STOP: binario braccio '$BIN' inesistente."; exit 1; }
else
  TAG="${1:?uso: pin-phpr.sh <tag, es. s107> | --braccio <tag> <binario>}"
fi

H=$(shasum -a 256 "$BIN" | cut -c1-16)
EXP=$("$ORACLE" "$MICRO")
ON=$(env -u PHPR_REG_LOWER "$BIN" "$MICRO")
OFF=$(PHPR_REG_LOWER=0 "$BIN" "$MICRO")
if [ "$ON" != "$EXP" ] || [ "$OFF" != "$EXP" ]; then
  echo "SMOKE FALLITO: oracle='$EXP' on='$ON' off='$OFF' => NIENTE stash."; exit 1
fi
# Guardia NO-CLOBBER (incidente S-133: promo con tag vecchio → stash storico
# sovrascritto): uno stash ESISTENTE con hash DIVERSO non si sovrascrive MAI
# in silenzio — un tag sbagliato muore QUI, non sul binario storico.
# Sostituzione consapevole = PIN_FORCE=1 dichiarato a verbale.
if [ -e "$STASH/phpr-$TAG" ] && [ "${PIN_FORCE:-0}" != 1 ]; then
  HOLD=$(shasum -a 256 "$STASH/phpr-$TAG" | cut -c1-16)
  if [ "$HOLD" != "$H" ]; then
    echo "STOP NO-CLOBBER: stash phpr-$TAG ESISTE con hash DIVERSO ($HOLD != $H) — tag sbagliato o rotazione non dichiarata; sovrascrivere solo con PIN_FORCE=1."
    exit 1
  fi
fi
cp "$BIN" "$STASH/phpr-$TAG"
HS=$(shasum -a 256 "$STASH/phpr-$TAG" | cut -c1-16)
[ "$H" = "$HS" ] || { echo "hash dello stash diverso ($H vs $HS)"; exit 1; }
# Az.rev. S-134 (apparato S-135): la riga phpr entra a REGISTRO da qui,
# fail-closed come pin-server.sh — nella SEZIONE phpr, in testa alla sua
# tabella (newest-first); ancora assente => registro NON toccato, STOP.
# La riga registra il collaudo MINIMO; l'arricchimento (promozione, A/B)
# resta un edit dichiarato della stessa riga a promozione avvenuta.
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
cd "$REPO"
HEAD_SHA=$(git rev-parse --short HEAD)
if [ "$MODE" = braccio ]; then
  ROW="| $H | $TAG BRACCIO (sorgente @ $HEAD_SHA; riga da pin-phpr.sh --braccio) | smoke parità 2 modi OK $(date '+%F %T') — braccio di misura, NON pin | stash \`phpr-$TAG\` |"
else
  ROW="| $H | $TAG (sorgente @ $HEAD_SHA; riga da pin-phpr.sh) | smoke parità 2 modi OK $(date '+%F %T') — batteria/corpus/fixture/micro DOVUTI a parte | stash \`phpr-$TAG\` |"
fi
awk -v row="$ROW" '
  /^## phpr/ { sec=1 }
  { print }
  sec==1 && /^\|---\|---\|---\|---\|$/ { print row; sec=2; ins=1 }
  END { if (!ins) exit 3 }
' PIN_REGISTRY.md > PIN_REGISTRY.md.new || { echo "STOP: ancora tabella phpr non trovata in PIN_REGISTRY.md => registro NON scritto."; rm -f PIN_REGISTRY.md.new; exit 1; }
mv PIN_REGISTRY.md.new PIN_REGISTRY.md
MSG=$(mktemp); echo "${MODE} phpr $TAG: registro PIN_REGISTRY ($H)" > "$MSG"
git add PIN_REGISTRY.md && git commit -F "$MSG" >/dev/null && git push >/dev/null
rm -f "$MSG"
if [ -n "$(git status --porcelain PIN_REGISTRY.md)" ]; then
  echo "STOP: PIN_REGISTRY.md ancora sporco dopo il commit dell'atto."; exit 1
fi
if [ "$MODE" = braccio ]; then
  echo "BRACCIO phpr $TAG = $H (smoke parità 2 modi OK, stash phpr-$TAG, registrato+committato). Braccio di misura: NON è un pin."
else
  echo "PIN phpr $TAG = $H (smoke parità 2 modi OK, stash phpr-$TAG, registrato+committato). I gate del protocollo (batteria/corpus/fixture/micro) restano dovuti."
fi
