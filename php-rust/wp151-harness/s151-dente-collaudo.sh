#!/bin/bash
# s151-dente-collaudo.sh — armamento dente A4 (loc_dente.rs) con collaudo in
# NEGATIVO+POSITIVO nell'atto (Hoare R7, sintesi §RATIFICA-A4). Esiti ESATTI,
# mai «diverso da» (feedback-forge-silent-failure). rc: 0=ARMATO · 5=negativo
# non morde · 6=positivo rosso · 7=setup.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp151-harness"; OUT="$H/dente-out"; mkdir -p "$OUT"
V="$H/s151-dente-collaudo-verdetto.out"
SYN="$REPO/crates/php-types/src/dente_collaudo_neg.rs"
: > "$V"; rm -f "$OUT/collaudo.done"
cd "$REPO" || { echo "rc=7 cd" >> "$V"; echo 7 > "$OUT/collaudo.rc"; exit 7; }

# Braccio NEGATIVO: violazione sintetica (2001 righe, fuori allowlist) DEVE
# mordere col messaggio ESATTO del dente.
yes '// collaudo negativo dente A4 (file sintetico, mai committare)' | head -n 2001 > "$SYN"
NLINES=$(wc -l < "$SYN" | tr -d ' ')
cargo test --release -p php-runtime --test loc_dente > "$OUT/neg.log" 2>&1
NEGRC=$?
NEGHIT=$(tr -d '\0' < "$OUT/neg.log" | grep -ac "dente_collaudo_neg.rs: 2001 righe > 2000")
rm -f "$SYN"
if [ "$NEGRC" -eq 0 ] || [ "$NLINES" != "2001" ] || [ "$NEGHIT" -lt 1 ]; then
  echo "collaudo NEGATIVO FALLITO: rc=$NEGRC (atteso !=0) syn=$NLINES righe hit=$NEGHIT — DENTE NON ARMATO, incidente da contare" >> "$V"
  echo 5 > "$OUT/collaudo.rc"; touch "$OUT/collaudo.done"; exit 5
fi
echo "collaudo negativo: rc=$NEGRC, hit=1 (violazione sintetica 2001 righe morde col messaggio esatto)" >> "$V"

# Braccio POSITIVO: albero pulito, il dente DEVE passare (cap esatti allineati).
cargo test --release -p php-runtime --test loc_dente > "$OUT/pos.log" 2>&1
POSRC=$?
POSOK=$(tr -d '\0' < "$OUT/pos.log" | grep -ac "test result: ok. 1 passed")
if [ "$POSRC" -ne 0 ] || [ "$POSOK" -lt 1 ]; then
  echo "collaudo POSITIVO FALLITO: rc=$POSRC ok=$POSOK — cap non allineati o scansione errata; DENTE NON ARMATO (dettaglio: dente-out/pos.log)" >> "$V"
  echo 6 > "$OUT/collaudo.rc"; touch "$OUT/collaudo.done"; exit 6
fi
echo "collaudo positivo: rc=0, 1 passed (allowlist 21 a cap esatti, anti-slack 200)" >> "$V"
echo "DENTE ARMATO rc=0 (negativo morde + positivo verde)" >> "$V"
echo 0 > "$OUT/collaudo.rc"; touch "$OUT/collaudo.done"
