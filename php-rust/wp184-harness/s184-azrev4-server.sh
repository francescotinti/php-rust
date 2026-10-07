#!/bin/bash
# s184-azrev4-server.sh — az.rev. S-183 #4: la RICETTA di pin-server.sh
# (`cargo build --release -p php-server --features axum-server`, SOURCE_DATE_EPOCH=0,
# CARGO_INCREMENTAL=0) eseguita dall'archivio del commit del pin (45e41174) in una
# target SEPARATA sulla sparsebundle; hash confrontato con b2802f08c5887e77.
# Esiti SOLO in azrev4-out/{build.log,verdetto.out,rc,done}. Nessuna build sulla canonica.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"
OUT="$REPO/php-rust/wp184-harness/azrev4-out"
SRC="/Volumes/Extreme Pro/Claude/s184-azrev4"
TGT="$HOME/Claude/phpr-target/s184-azrev4-tgt"
PIN_COMMIT=45e41174; PIN_HASH=b2802f08c5887e77
cd "$REPO" || exit 9
rm -f "$OUT/done" "$OUT/rc"
/sbin/mount | grep -q " $HOME/Claude/phpr-target " || { echo "bundle smontata" > "$OUT/verdetto.out"; echo 8 > "$OUT/rc"; touch "$OUT/done"; exit 8; }
rm -rf "$SRC"; mkdir -p "$SRC"
git archive "$PIN_COMMIT" php-rust/crates php-rust/Cargo.toml php-rust/Cargo.lock php-rust/rust-toolchain.toml php-rust/.cargo | tar -x -C "$SRC" || { echo "archive fallito" > "$OUT/verdetto.out"; echo 9 > "$OUT/rc"; touch "$OUT/done"; exit 9; }
T0=$(date +%s)
( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" \
    cargo build --release -p php-server --features axum-server ) > "$OUT/build.log" 2>&1
BRC=$?
T1=$(date +%s)
{
  echo "== s184 az.rev.4: ricetta pin-server.sh da git archive $PIN_COMMIT, target $TGT, build rc=$BRC, $((T1-T0)) s"
  if [ $BRC -ne 0 ]; then echo "ESITO rc=9 (build fallita)"; echo 9 > "$OUT/rc"; else
    H=$(shasum -a 256 "$TGT/release/php-server" | cut -c1-16)
    echo "hash ricetta = $H · pin s181 = $PIN_HASH · stash = $(shasum -a 256 '/Volumes/Extreme Pro/Claude/phpr-old-target/release/php-server-s181' | cut -c1-16)"
    if [ "$H" = "$PIN_HASH" ]; then echo "RIPRODUCIBILE AL BYTE ⇒ incidente #5 S-183 = RITIRATO (era la build dell'intero workspace, non la ricetta)"; echo 0 > "$OUT/rc";
    else echo "NON riproducibile ($H ≠ $PIN_HASH) ⇒ incidente #5 CONFERMATO sulla ricetta"; echo 1 > "$OUT/rc"; fi
  fi
} > "$OUT/verdetto.out"
touch "$OUT/done"
