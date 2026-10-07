#!/bin/bash
# s184-azrev4c-server.sh — az.rev. S-183 #4, prova 3: la ricetta di pin-server.sh eseguita dal PERCORSO della canonica (il repo,
# tree a HEAD = sorgente del pin server s182 @ 649a4b79, nessun checkout) in una target SEPARATA sulla bundle; hash confrontato
# col pin server s182 6783ad8503c5f2d6 appena nato dalla stessa ricetta sulla canonica. Esiti in azrev4-out/{build-c.log,verdetto-c.out,done-c}.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"; OUT="$REPO/php-rust/wp184-harness/azrev4-out"
TGT="$HOME/Claude/phpr-target/s184-azrev4c-tgt"; PIN_HASH=6783ad8503c5f2d6
cd "$REPO/php-rust" || exit 9; rm -f "$OUT/done-c"
H0=$(git rev-parse --short HEAD); T0=$(date +%s)
( SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" cargo build --release -p php-server --features axum-server ) > "$OUT/build-c.log" 2>&1; BRC=$?
H=$(shasum -a 256 "$TGT/release/php-server" 2>/dev/null | cut -c1-16)
{
  echo "== s184 az.rev.4 prova 3: ricetta pin-server.sh dal percorso canonico (tree HEAD $H0, sorgente server == 649a4b79), target separata $TGT, build rc=$BRC, $(( $(date +%s)-T0 )) s"
  if [ $BRC -ne 0 ]; then echo "ESITO rc=9 (build fallita)"; else
    echo "hash ricetta (target separata) = $H · pin server s182 (canonica) = $PIN_HASH"
    if [ "$H" = "$PIN_HASH" ]; then echo "RIPRODUCIBILE AL BYTE a parità di percorso sorgente ⇒ incidente #5 S-183 RITIRATO: la ricetta riproduce il pin; la prova 1 (archivio altrove) divergeva per il percorso, la S-183 per la build dell'intero workspace"; else echo "NON riproducibile ($H ≠ $PIN_HASH) pur a parità di percorso sorgente ⇒ la target entra nel binario o la ricetta non è deterministica: incidente #5 CONFERMATO (da leggere con la target)"; fi
  fi
} > "$OUT/verdetto-c.out"; touch "$OUT/done-c"
