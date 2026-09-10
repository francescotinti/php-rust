#!/bin/bash
# s173-catena-ab.sh — catena post-build della fetta 3 (criterio s173-criterio.md p.9): attende
# build B (P3) e C (P3+P4), parità + dump loop (micro/prop.php: le 8 op del loop sono invariate
# per costruzione, P3 è un peephole RUNTIME), fx-sl3/fx-sl2/fx-sl1 bilaterali sui candidati e
# fx-sl3-div invariante (candidato==pin), stash --braccio (atto unico pin-phpr.sh, COL COMMIT
# SORGENTE: az.rev. S-172 #3), symlink NEUTRI /private/tmp/s173-bins/{A,B,C}, disasm bl run_loop
# A/B/C (s172-disasm.sh, generico, riusato), poi exec del lanciatore (quiete s129 +
# s173-ab-leva.sh R=5 ruotato). COPIA DICHIARATA di s172-catena-ab.sh (manifest
# s173-catena-ab-copia.diff). Esiti a file (ab-out/catena-f3.log); rc autoritativo dell'A/B =
# ab-out/f3.rc; verdetto s173-f3-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp172-harness"; OUT="$H/ab-out"; LOG="$OUT/catena-f3.log"
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_EXP=5f2dff7d17ebed79
BINS=/private/tmp/s173-bins
COMMIT_B=2006d11d; COMMIT_C=64c55cc2
O=/opt/homebrew/opt/php/bin/php; MP="$REPO/wp97-harness/micro/prop.php"
echo "$(date '+%F %T') catena f3 avviata" >> "$LOG"
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
# fx-sl3 + fx-sl2 + fx-sl1 bilaterali sui candidati (presidio PRIMA della misura)
for fx in "$H/fx-sl3.php" "$H/fx-sl2.php" "$H/fx-sl1.php"; do
  n=$(basename "$fx" .php)
  "$O" -d log_errors=0 -d display_errors=1 "$fx" > "$OUT/$n-oracle.out" 2>&1
  for b in B C; do
    perl -e 'alarm 60; exec @ARGV or die' -- "$OUT/phpr-$b" "$fx" > "$OUT/$n-$b.out" 2>&1
    cmp -s "$OUT/$n-$b.out" "$OUT/$n-oracle.out" || { diff "$OUT/$n-oracle.out" "$OUT/$n-$b.out" > "$OUT/$n-$b.diff"; echo "$n: $b ≠ oracle (ab-out/$n-$b.diff) — STOP" >> "$LOG"; exit 2; }
  done
  echo "$n: B==oracle, C==oracle byte-id" >> "$LOG"
done
# fx-sl3-div INVARIANTE: candidato == pin (divergenza §3.32 pre-esistente, non toccata)
"$PIN" "$H/fx-sl3-div.php" > "$OUT/fx-sl3-div-A.out" 2>&1
for b in B C; do
  perl -e 'alarm 60; exec @ARGV or die' -- "$OUT/phpr-$b" "$H/fx-sl3-div.php" > "$OUT/fx-sl3-div-$b.out" 2>&1
  cmp -s "$OUT/fx-sl3-div-$b.out" "$OUT/fx-sl3-div-A.out" || { echo "fx-sl3-div: $b ≠ pin — STOP" >> "$LOG"; exit 2; }
done
echo "fx-sl3-div: B==pin, C==pin byte-id (§3.32 invariata)" >> "$LOG"
# stash dei bracci (atto unico pin-phpr --braccio: smoke 2 modi + stash + registro COL COMMIT SORGENTE)
for b in B C; do
  case $b in B) SC=$COMMIT_B;; C) SC=$COMMIT_C;; esac
  "$REPO/scripts/pin-phpr.sh" --braccio "s173-f3-$b" "$OUT/phpr-$b" "$SC" > "$OUT/braccio-f3-$b.log" 2>&1 || { echo "--braccio $b FALLITO (ab-out/braccio-f3-$b.log)" >> "$LOG"; exit 5; }
  echo "braccio $b: $(tail -1 "$OUT/braccio-f3-$b.log")" >> "$LOG"
done
# symlink neutri + disasm
rm -rf "$BINS"; mkdir -p "$BINS"; ln -s "$PIN" "$BINS/A"; ln -s "$OUT/phpr-B" "$BINS/B"; ln -s "$OUT/phpr-C" "$BINS/C"
bash "$H/s172-disasm.sh" "$BINS/A" "$BINS/B" "$BINS/C" > "$OUT/disasm-sl3.out" 2>&1
cat "$OUT/disasm-sl3.out" >> "$LOG"
echo "$(date '+%F %T') exec lanciatore" >> "$LOG"
exec /bin/bash "$H/s172-lancio-ab.sh" s173-ab-leva.sh "$BINS/A" "${HA:0:8}" "$BINS/B" "${HB:0:8}" "$BINS/C" "${HC:0:8}" f3 5
