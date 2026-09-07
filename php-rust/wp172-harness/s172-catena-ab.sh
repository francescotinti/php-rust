#!/bin/bash
# s172-catena-ab.sh — catena post-build della fetta 2 (criterio p.9): attende build B e C,
# parità + dump loop (micro/prop.php: le 8 op del loop sono invariate per costruzione,
# nessun cambio a reg_lower), fx-sl2/fx-sl1 bilaterali sui candidati, stash --braccio
# (atto unico pin-phpr.sh), symlink NEUTRI /private/tmp/s172-bins/{A,B,C} (veto S-171:
# mai il nome del binario nell'argv delle attese), disasm bl run_loop A/B/C, poi exec del
# lanciatore (quiete s129 + s172-ab-leva.sh R=5). Esiti a file (ab-out/catena.log);
# rc autoritativo dell'A/B = ab-out/leva.rc; verdetto s172-leva-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp172-harness"; OUT="$H/ab-out"; LOG="$OUT/catena.log"
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_EXP=b360b2933eddfe18
BINS=/private/tmp/s172-bins
O=/opt/homebrew/opt/php/bin/php; MP="$REPO/wp97-harness/micro/prop.php"
echo "$(date '+%F %T') catena avviata" >> "$LOG"
until [ -e "$OUT/build-C.done" ]; do sleep 10; done
for b in B C; do
  [ "$(cat "$OUT/build-$b.rc")" = 0 ] || { echo "build $b rc=$(cat "$OUT/build-$b.rc") — STOP" >> "$LOG"; exit 4; }
done
HA=$(shasum -a 256 "$PIN" | cut -c1-16); [ "$HA" = "$PIN_EXP" ] || { echo "pin $HA != $PIN_EXP — STOP" >> "$LOG"; exit 9; }
HB=$(shasum -a 256 "$OUT/phpr-B" | cut -c1-16); HC=$(shasum -a 256 "$OUT/phpr-C" | cut -c1-16)
echo "A=$HA B=$HB C=$HC" >> "$LOG"
[ "$HB" != "$HA" ] && [ "$HC" != "$HA" ] && [ "$HB" != "$HC" ] || { echo "hash coincidenti — STOP" >> "$LOG"; exit 6; }
# parità + dump loop (micro/prop.php N=30M): output == oracle e dump {main} identico A/B/C
"$O" "$MP" > "$OUT/parita-prop-oracle.out" 2>&1
for b in A B C; do
  case $b in A) BIN="$PIN";; B) BIN="$OUT/phpr-B";; C) BIN="$OUT/phpr-C";; esac
  PHPR_DUMP_OPS=1 "$BIN" "$MP" > "$OUT/parita-prop-$b.out" 2> "$OUT/dump-prop-$b.txt"
  cmp -s "$OUT/parita-prop-$b.out" "$OUT/parita-prop-oracle.out" || { echo "parità prop $b ≠ oracle — STOP" >> "$LOG"; exit 2; }
  awk '/^-- \{main\}/{f=1} f&&/^-- fn /{exit} f' "$OUT/dump-prop-$b.txt" > "$OUT/dump-prop-$b-main.txt"
done
cmp -s "$OUT/dump-prop-A-main.txt" "$OUT/dump-prop-B-main.txt" && cmp -s "$OUT/dump-prop-A-main.txt" "$OUT/dump-prop-C-main.txt" \
  && echo "parità prop: A==B==C==oracle; dump {main} A==B==C ($(grep -c . "$OUT/dump-prop-A-main.txt") righe)" >> "$LOG" \
  || { echo "dump {main} DIVERGE tra bracci — STOP" >> "$LOG"; exit 2; }
# fx-sl2 + fx-sl1 bilaterali sui candidati (presidio PRIMA della misura)
for fx in "$H/fx-sl2.php" "$H/fx-sl1.php"; do
  n=$(basename "$fx" .php)
  "$O" -d log_errors=0 -d display_errors=1 "$fx" > "$OUT/$n-oracle.out" 2>&1
  for b in B C; do
    perl -e 'alarm 60; exec @ARGV or die' -- "$OUT/phpr-$b" "$fx" > "$OUT/$n-$b.out" 2>&1
    cmp -s "$OUT/$n-$b.out" "$OUT/$n-oracle.out" || { diff "$OUT/$n-oracle.out" "$OUT/$n-$b.out" > "$OUT/$n-$b.diff"; echo "$n: $b ≠ oracle (ab-out/$n-$b.diff) — STOP" >> "$LOG"; exit 2; }
  done
  echo "$n: B==oracle, C==oracle byte-id" >> "$LOG"
done
# stash dei bracci (atto unico pin-phpr --braccio: smoke 2 modi + stash + registro)
for b in B C; do
  "$REPO/scripts/pin-phpr.sh" --braccio "s172-sl2-$b" "$OUT/phpr-$b" > "$OUT/braccio-$b.log" 2>&1 || { echo "--braccio $b FALLITO (ab-out/braccio-$b.log)" >> "$LOG"; exit 5; }
  echo "braccio $b: $(tail -1 "$OUT/braccio-$b.log")" >> "$LOG"
done
# symlink neutri + disasm
rm -rf "$BINS"; mkdir -p "$BINS"; ln -s "$PIN" "$BINS/A"; ln -s "$OUT/phpr-B" "$BINS/B"; ln -s "$OUT/phpr-C" "$BINS/C"
bash "$H/s172-disasm.sh" "$BINS/A" "$BINS/B" "$BINS/C" > "$OUT/disasm-sl2.out" 2>&1
cat "$OUT/disasm-sl2.out" >> "$LOG"
echo "$(date '+%F %T') exec lanciatore" >> "$LOG"
exec /bin/bash "$H/s172-lancio-ab.sh" s172-ab-leva.sh "$BINS/A" "${HA:0:8}" "$BINS/B" "${HB:0:8}" "$BINS/C" "${HC:0:8}" leva 5
