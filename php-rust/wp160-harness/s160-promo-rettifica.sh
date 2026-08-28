#!/bin/bash
# s160-promo-rettifica.sh — REGISTRAZIONE POSTUMA (revisione S-160 #3) dei
# comandi ESEGUITI in-sessione per ri-derivare le evidenze viziate dal run
# promo misdiretto (H stantio). Artifact prodotti: promo-out/{fxaf,fxam}-*.out,
# promo-out/quiesce-rett.rc, promo-out/conferma-rett-runs.tsv. Esiti nel
# blocco RETTIFICA di s160-promo-verdetto.out. NON rieseguire su pin diverso.
set -u
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp160-harness"
ORACLE=/opt/homebrew/opt/php/bin/php
BIN="$HOME/Claude/php-rust-output/release/phpr"   # pin s160 ceeb6e76e4ef5ace
AOLD="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s159"
cd "$H" || exit 4
# gate fx-af / fx-am v2: bilaterale + MARCATORE PRETESO (mai solo A==B)
"$ORACLE" fx-af.php > promo-out/fxaf-oracle.out 2>&1
"$BIN"    fx-af.php > promo-out/fxaf-pin.out    2>&1
cmp promo-out/fxaf-oracle.out promo-out/fxaf-pin.out && grep -aq "^FXAF-END$" promo-out/fxaf-oracle.out || exit 1
"$ORACLE" fx-am.php > promo-out/fxam-oracle.out 2>&1
"$BIN"    fx-am.php > promo-out/fxam-pin.out    2>&1
cmp promo-out/fxam-oracle.out promo-out/fxam-pin.out && grep -aq "^FXAM-END$" promo-out/fxam-oracle.out || exit 1
# conferma post-pin m-arrfilter: quiescenza + R=5 ABAB pin vs stash s159
../wp129-harness/s129-quiescenza.sh promo-out/quiesce-rett.rc > promo-out/quiesce-rett.log 2>&1 || exit 1
[ "$(shasum -a 256 "$AOLD" | cut -c1-16)" = "f2d17f18c00a4049" ] || exit 1
# (loop ABAB con /usr/bin/time -p e floors med3, matematica identica alla
#  promo; vedi conferma-rett-runs.tsv — esito: D=+15.0 rumore=1.0 segni=5/5)
