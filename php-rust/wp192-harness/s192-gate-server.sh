#!/bin/bash
# s192-gate-server.sh <tag> <commit> — S-192: i due gate FUNZIONALI del server sul sorgente con le leve di S-192, che
# l'accettazione di giorno non esercita: (1) `wp189-harness/gate-loopback/gate-loopback.sh` (coda condivisa del pool axum),
# (2) `wp188-harness/battery61-axum.sh` (WordPress su `php-server --axum --workers 4` contro `php -S`, sei probe byte-id).
# Il server si costruisce da ARCHIVIO del commit con la ricetta di pin-server (`cargo build --release -p php-server
# --features axum-server`) in una target a parte sulla bundle (`s192-armS`): NON è un pin, NON tocca la canonica.
# I due script girano INVARIATI (solo variabili d'ambiente: NEW_BIN/OLD_BIN/PIN_ATTESO, PHPSERVER/OUT). Nessun tempo.
# Esito: s192-gate-server-<tag>.out, out/gate-server-<tag>.rc (0 = entrambi verdi). Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; O="$H/out"; BT="$HOME/Claude/phpr-target"
TAG="${1:?tag}"; COMMIT="${2:?commit}"; SRC="$BT/s192-srcS"; TGT="$BT/s192-armS"
STASH="/Volumes/Extreme Pro/Claude/phpr-old-target/release"; FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
V="$H/s192-gate-server-$TAG.out"; : > "$V"; rm -f "$O/gate-server-$TAG.rc"; RC=0
note(){ echo "$1" >> "$V"; }
fin(){ echo "$1" > "$O/gate-server-$TAG.rc"; exit "$1"; }
until tail -1 "$FEED" | grep -q '^DONE' && ! pgrep -x cargo >/dev/null; do sleep 30; done
rm -rf "$SRC"; mkdir -p "$SRC" || fin 7
git -C "$P/.." archive "$COMMIT" php-rust/crates php-rust/Cargo.toml php-rust/Cargo.lock php-rust/rust-toolchain.toml php-rust/.cargo 2>/dev/null | tar -x -m -C "$SRC" || fin 7
( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" cargo build --release -p php-server --features axum-server ) > "$O/gate-server-$TAG-build.log" 2>&1 || { note "build del server FALLITA"; fin 7; }
SRV="$O/php-server-$TAG"; cp "$TGT/release/php-server" "$SRV" || fin 7
note "== s192 gate server tag=$TAG commit=$COMMIT php-server=$(shasum -a 256 "$SRV" | cut -c1-16) $(date '+%F %T') =="
OLD="$STASH/php-server-s188"
if [ -s "$OLD" ]; then
  OLD_BIN="$OLD" NEW_BIN="$SRV" PIN_ATTESO="$(shasum -a 256 "$OLD" | cut -c1-16)" bash "$P/wp189-harness/gate-loopback/gate-loopback.sh" > "$O/gate-server-$TAG-loopback.log" 2>&1; r=$?
else
  ARMS="new3 new1 oracle3 oracle1" NEW_BIN="$SRV" bash "$P/wp189-harness/gate-loopback/gate-loopback.sh" > "$O/gate-server-$TAG-loopback.log" 2>&1; r=$?
fi
note "gate-loopback: rc=$r · $(tail -1 "$P/wp189-harness/gate-loopback/out/gate_loopback.verdict" 2>/dev/null | cut -c1-200)"; [ "$r" = 0 ] || RC=1
PHPSERVER="$SRV" OUT="$O/battery61-axum-$TAG" bash "$P/wp188-harness/battery61-axum.sh" > "$O/gate-server-$TAG-battery61.log" 2>&1; r=$?
note "battery61-axum: rc=$r · $(tail -3 "$O/gate-server-$TAG-battery61.log" | tr '\n' ' ' | cut -c1-260)"; [ "$r" = 0 ] || RC=1
note "esito rc=$RC $(date '+%F %T')"; fin "$RC"
