#!/bin/bash
# s110-l1i-t1.sh — T1 gate tooling della leva (d) (criterio s110-l1i-criterio.out):
# UNA registrazione di prova CPU Counters su phpr/arith_small deve esporre via
# export conteggi per-processo con {cycles, INST_ALL, L1I_CACHE_MISS_DEMAND}.
# Esito nel file t1-verdetto.out: PASS-STOCK | PASS-CUSTOM | FAIL-TOOLING.
# NON parte finche' la coppia WP e' in volo (vincolo del criterio).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp110-harness"
OUT="$H/l1i-out"; mkdir -p "$OUT"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
# EMENDAMENTO S-110 (dichiarato): il figlio tracciato da Instruments si blocca
# in open() sui path del volume esterno (prompt TCC senza risposta in headless).
# I giudici sono copie BYTE-IDENTICHE di wp97-harness/micro su disco interno,
# uguali per ENTRAMBI i motori: la ricetta di misura non cambia.
MICRO="$HOME/Claude/l1i-micro"
TPL="${1:-CPU Counters}"   # nome template di serie O path a un .tracetemplate custom
TAG="${2:-stock}"
rm -f "$OUT/t1-$TAG.trace" 2>/dev/null; rm -rf "$OUT/t1-$TAG.trace"
xctrace record --template "$TPL" --output "$OUT/t1-$TAG.trace" --no-prompt \
  --target-stdout /dev/null --launch -- "$PHPR" "$MICRO/arith_small.php" \
  > "$OUT/t1-$TAG-record.log" 2>&1
rec_rc=$?
echo "record_rc=$rec_rc template=$TPL" > "$OUT/t1-$TAG-verdetto.out"
if [ "$rec_rc" -ne 0 ]; then echo "esito=FAIL-RECORD" >> "$OUT/t1-$TAG-verdetto.out"; exit 1; fi
xctrace export --input "$OUT/t1-$TAG.trace" --toc > "$OUT/t1-$TAG-toc.xml" 2>>"$OUT/t1-$TAG-record.log"
toc_rc=$?
echo "toc_rc=$toc_rc" >> "$OUT/t1-$TAG-verdetto.out"
grep -o 'schema="[^"]*"' "$OUT/t1-$TAG-toc.xml" | sort -u >> "$OUT/t1-$TAG-verdetto.out"
