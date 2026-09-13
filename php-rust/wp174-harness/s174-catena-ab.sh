#!/bin/bash
# s174-catena-ab.sh — catena post-build della leva «Sweep-in-op» (criterio s174-criterio.md p.7):
# attende build Z (gemello del pin), B (sweep-in-op) e C (B+back-edge fuso); parità + dump {main}
# sui DUE giudici (arith-dq, prop-dq: le op del loop sono invariate per costruzione, entrambe le
# fusioni sono peephole RUNTIME); fx-sw1 (NUOVA) + fx-sl3/fx-sl2/fx-sl1 bilaterali sui candidati e
# fx-sl3-div invariante (candidato==pin); stash --braccio (atto unico pin-phpr.sh COL COMMIT
# SORGENTE) per B e C (Z = 19c0540e già stashato come phpr-s173-f3-C: dichiarato, non ristashato);
# symlink NEUTRI /private/tmp/s174-bins/{A,Z,B,C}; disasm bl run_loop dei 4 (s172-disasm.sh riusato);
# poi exec del lanciatore (quiete s129 + s174-ab-leva.sh R=5 ruotato a 4 bracci).
# COPIA DICHIARATA di ../wp172-harness/s173-catena-ab.sh (manifest s174-catena-ab-copia.diff).
# Esiti a file (ab-out/catena-sw.log); rc autoritativo dell'A/B = ab-out/sw.rc; verdetto s174-sw-verdetto.out.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp174-harness"; H2="$REPO/wp172-harness"; OUT="$H/ab-out"; LOG="$OUT/catena-sw.log"
PIN="$HOME/Claude/php-rust-output/release/phpr"; PIN_EXP=da4921a52eba0187; Z_EXP=19c0540e03d590a8
BINS=/private/tmp/s174-bins
COMMIT_B=9c9e6a61; COMMIT_C=883cb598
O=/opt/homebrew/opt/php/bin/php; GD="$REPO/wp164-harness/arith-dq.php"; PD="$H2/prop-dq.php"
echo "$(date '+%F %T') catena sw avviata" >> "$LOG"
until [ -e "$OUT/build-C.done" ]; do sleep 10; done
for b in Z B C; do
  [ "$(cat "$OUT/build-$b.rc")" = 0 ] || { echo "build $b rc=$(cat "$OUT/build-$b.rc") — STOP" >> "$LOG"; exit 4; }
done
HA=$(shasum -a 256 "$PIN" | cut -c1-16); [ "$HA" = "$PIN_EXP" ] || { echo "pin $HA != $PIN_EXP — STOP" >> "$LOG"; exit 9; }
HZ=$(shasum -a 256 "$OUT/phpr-Z" | cut -c1-16); HB=$(shasum -a 256 "$OUT/phpr-B" | cut -c1-16); HC=$(shasum -a 256 "$OUT/phpr-C" | cut -c1-16)
echo "A=$HA Z=$HZ B=$HB C=$HC" >> "$LOG"
[ "$HZ" = "$Z_EXP" ] || { echo "Z $HZ != atteso $Z_EXP (gemello del pin, emenda dichiarata) — STOP" >> "$LOG"; exit 6; }
[ "$HB" != "$HA" ] && [ "$HC" != "$HA" ] && [ "$HB" != "$HC" ] && [ "$HB" != "$HZ" ] && [ "$HC" != "$HZ" ] || { echo "hash coincidenti — STOP" >> "$LOG"; exit 6; }
# parità + dump {main} sui due giudici: output == oracle e dump identico A/Z/B/C
for D in "$GD" "$PD"; do
  n=$(basename "$D" .php)
  "$O" "$D" > "$OUT/parita-$n-oracle.out" 2>&1
  for b in A Z B C; do
    case $b in A) BIN="$PIN";; *) BIN="$OUT/phpr-$b";; esac
    PHPR_DUMP_OPS=1 "$BIN" "$D" > "$OUT/parita-$n-$b.out" 2> "$OUT/dump-$n-$b.txt"
    cmp -s "$OUT/parita-$n-$b.out" "$OUT/parita-$n-oracle.out" || { echo "parità $n $b ≠ oracle — STOP" >> "$LOG"; exit 2; }
    awk '/^-- \{main\}/{f=1} f&&/^-- fn /{exit} f' "$OUT/dump-$n-$b.txt" > "$OUT/dump-$n-$b-main.txt"
  done
  cmp -s "$OUT/dump-$n-A-main.txt" "$OUT/dump-$n-Z-main.txt" && cmp -s "$OUT/dump-$n-A-main.txt" "$OUT/dump-$n-B-main.txt" && cmp -s "$OUT/dump-$n-A-main.txt" "$OUT/dump-$n-C-main.txt" \
    && echo "parità $n: A==Z==B==C==oracle; dump {main} A==Z==B==C ($(grep -c . "$OUT/dump-$n-A-main.txt") righe)" >> "$LOG" \
    || { echo "dump {main} $n DIVERGE tra bracci — STOP" >> "$LOG"; exit 2; }
done
# fx-sw1 (NUOVA) su Z/B/C + fx-sl3/fx-sl2/fx-sl1 bilaterali su B/C (presidio PRIMA della misura)
for fx in "$H/fixtures/fx-sw1.php" "$H2/fx-sl3.php" "$H2/fx-sl2.php" "$H2/fx-sl1.php"; do
  n=$(basename "$fx" .php)
  "$O" -d log_errors=0 -d display_errors=1 "$fx" > "$OUT/$n-oracle.out" 2>&1
  for b in Z B C; do
    [ "$n" != fx-sw1 ] && [ "$b" = Z ] && continue
    perl -e 'alarm 60; exec @ARGV or die' -- "$OUT/phpr-$b" "$fx" > "$OUT/$n-$b.out" 2>&1
    cmp -s "$OUT/$n-$b.out" "$OUT/$n-oracle.out" || { diff "$OUT/$n-oracle.out" "$OUT/$n-$b.out" > "$OUT/$n-$b.diff"; echo "$n: $b ≠ oracle (ab-out/$n-$b.diff) — STOP" >> "$LOG"; exit 2; }
  done
  echo "$n: candidati == oracle byte-id ($(grep -c . "$OUT/$n-oracle.out") righe)" >> "$LOG"
done
# fx-sl3-div INVARIANTE: candidato == pin (divergenza §3.32 pre-esistente, non toccata)
"$PIN" "$H2/fx-sl3-div.php" > "$OUT/fx-sl3-div-A.out" 2>&1
for b in B C; do
  perl -e 'alarm 60; exec @ARGV or die' -- "$OUT/phpr-$b" "$H2/fx-sl3-div.php" > "$OUT/fx-sl3-div-$b.out" 2>&1
  cmp -s "$OUT/fx-sl3-div-$b.out" "$OUT/fx-sl3-div-A.out" || { echo "fx-sl3-div: $b ≠ pin — STOP" >> "$LOG"; exit 2; }
done
echo "fx-sl3-div: B==pin, C==pin byte-id (§3.32 invariata)" >> "$LOG"
# stash dei bracci B e C (atto unico pin-phpr --braccio COL COMMIT SORGENTE); Z non ristashato (== phpr-s173-f3-C)
for b in B C; do
  case $b in B) SC=$COMMIT_B;; C) SC=$COMMIT_C;; esac
  "$REPO/scripts/pin-phpr.sh" --braccio "s174-sw-$b" "$OUT/phpr-$b" "$SC" > "$OUT/braccio-sw-$b.log" 2>&1 || { echo "--braccio $b FALLITO (ab-out/braccio-sw-$b.log)" >> "$LOG"; exit 5; }
  echo "braccio $b: $(tail -1 "$OUT/braccio-sw-$b.log")" >> "$LOG"
done
# symlink neutri + disasm dei 4 bracci
rm -rf "$BINS"; mkdir -p "$BINS"; ln -s "$PIN" "$BINS/A"; ln -s "$OUT/phpr-Z" "$BINS/Z"; ln -s "$OUT/phpr-B" "$BINS/B"; ln -s "$OUT/phpr-C" "$BINS/C"
bash "$H2/s172-disasm.sh" "$BINS/A" "$BINS/Z" "$BINS/B" "$BINS/C" > "$OUT/disasm-sw.out" 2>&1
cat "$OUT/disasm-sw.out" >> "$LOG"
echo "$(date '+%F %T') exec lanciatore" >> "$LOG"
exec /bin/bash "$H/s174-lancio-ab.sh" s174-ab-leva.sh "$BINS/A" "${HA:0:8}" "$BINS/Z" "${HZ:0:8}" "$BINS/B" "${HB:0:8}" "$BINS/C" "${HC:0:8}" sw 5
