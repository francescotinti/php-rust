#!/bin/bash
# s184-azrev4b-server.sh — az.rev. S-183 #4, seconda prova: la ricetta di pin-server.sh eseguita dal PERCORSO della canonica
# (il repo stesso, checkout DETACHED del commit del pin server 45e41174) in target SEPARATA sulla bundle — la prova 1 (archivio
# in /Volumes/Extreme Pro/Claude/s184-azrev4) non è conclusiva: il percorso del sorgente entra nel binario (Z ≠ pin anche per
# phpr in S-183). Tree pulito richiesto; si torna a main in ogni caso. Esiti in azrev4-out/{build-b.log,verdetto-b.out,rc-b,done-b}.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment"; OUT="$REPO/php-rust/wp184-harness/azrev4-out"
TGT="$HOME/Claude/phpr-target/s184-azrev4b-tgt"; PIN_COMMIT=45e41174; PIN_HASH=b2802f08c5887e77
cd "$REPO" || exit 9; rm -f "$OUT/done-b" "$OUT/rc-b"
fin(){ echo "$1" > "$OUT/rc-b"; touch "$OUT/done-b"; exit "$1"; }
[ -z "$(git status --porcelain | grep -v '^??')" ] || { echo "tree NON pulito: non faccio checkout" > "$OUT/verdetto-b.out"; fin 9; }
BR=$(git rev-parse --abbrev-ref HEAD); [ "$BR" = main ] || { echo "non su main ($BR)" > "$OUT/verdetto-b.out"; fin 9; }
/sbin/mount | grep -q " $HOME/Claude/phpr-target " || { echo "bundle smontata" > "$OUT/verdetto-b.out"; fin 8; }
git checkout -q "$PIN_COMMIT" || { echo "checkout $PIN_COMMIT fallito" > "$OUT/verdetto-b.out"; fin 9; }
T0=$(date +%s)
( cd "$REPO/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" cargo build --release -p php-server --features axum-server ) > "$OUT/build-b.log" 2>&1; BRC=$?
H=$(shasum -a 256 "$TGT/release/php-server" 2>/dev/null | cut -c1-16)
git checkout -q main; BACK=$(git rev-parse --abbrev-ref HEAD)
{
  echo "== s184 az.rev.4 prova 2: ricetta pin-server.sh dal percorso della canonica (checkout detached $PIN_COMMIT), target $TGT, build rc=$BRC, $(( $(date +%s)-T0 )) s, tornato su: $BACK"
  if [ $BRC -ne 0 ]; then echo "ESITO rc=9 (build fallita)"; R=9; else
    echo "hash ricetta = $H · pin s181 = $PIN_HASH"
    if [ "$H" = "$PIN_HASH" ]; then echo "RIPRODUCIBILE AL BYTE dal percorso canonico ⇒ incidente #5 S-183 RITIRATO (era la build dell'intero workspace, non la ricetta); la prova 1 dall'archivio ≠ pin = effetto del percorso"; R=0;
    else echo "NON riproducibile ($H ≠ $PIN_HASH) anche dal percorso canonico ⇒ incidente #5 CONFERMATO sulla ricetta"; R=1; fi
  fi
} > "$OUT/verdetto-b.out"
[ "$BACK" = main ] || echo "ATTENZIONE: tree NON su main" >> "$OUT/verdetto-b.out"
fin "${R:-9}"
