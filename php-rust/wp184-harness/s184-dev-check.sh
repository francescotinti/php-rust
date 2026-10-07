#!/bin/bash
# s184-dev-check.sh — collaudo di sviluppo di L-RT2 (NON misura, NON pin): build dev-release di php-cli sulla target
# di sviluppo (bundle, .cargo/config.toml) + parità delle fixture bilaterali (fx-rt2, fx-rt1, fx-cr1, fx-sw1, fx-sl1-3) e
# del marcatore fx-sw2-gc contro il pin. Esiti in dev-out/{build.log,verdetto.out,done}.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
R="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$R/wp184-harness"; OUT="$H/dev-out"; mkdir -p "$OUT"
ORACLE=/opt/homebrew/opt/php/bin/php; PIN="$HOME/Claude/php-rust-output/release/phpr"
rm -f "$OUT/done"; : > "$OUT/verdetto.out"
/sbin/mount | grep -q " $HOME/Claude/phpr-target " || { echo "bundle smontata" >> "$OUT/verdetto.out"; echo "rc=8" > "$OUT/done"; exit 8; }
T0=$(date +%s)
( cd "$R" && cargo build --profile dev-release -p php-cli ) > "$OUT/build.log" 2>&1; BRC=$?
echo "build dev-release rc=$BRC in $(( $(date +%s) - T0 )) s" >> "$OUT/verdetto.out"
[ $BRC -eq 0 ] || { echo "rc=4" > "$OUT/done"; exit 4; }
DEV="$HOME/Claude/phpr-target/dev-output/dev-release/phpr"
RC=0
chk(){ # nome file marcatore opzioni...
  local n="$1" f="$2" m="$3"; shift 3
  "$ORACLE" "$@" "$f" > "$OUT/$n-oracle.out" 2>&1
  perl -e 'alarm 300; exec @ARGV or die' -- "$DEV" "$f" > "$OUT/$n-dev.out" 2>&1
  grep -q "$m" "$OUT/$n-dev.out" || { echo "$n: marcatore $m ASSENTE" >> "$OUT/verdetto.out"; RC=2; return; }
  if diff -q "$OUT/$n-oracle.out" "$OUT/$n-dev.out" >/dev/null; then echo "$n: dev == oracle BYTE-ID" >> "$OUT/verdetto.out"; else diff "$OUT/$n-oracle.out" "$OUT/$n-dev.out" > "$OUT/$n.diff"; echo "$n: DIVERGE ($(wc -l < "$OUT/$n.diff" | tr -d ' ') righe, dev-out/$n.diff)" >> "$OUT/verdetto.out"; RC=2; fi
}
chk fxrt2 "$H/fx-rt2.php" "FX-RT2 DONE" -d log_errors=0 -d display_errors=1
chk fxrt1 "$R/wp182-harness/fx-rt1.php" "FX-RT1 DONE" -d log_errors=0 -d display_errors=1
chk fxcr1 "$R/wp181-harness/fx-cr1.php" "FX-CR1 DONE" -d log_errors=0 -d display_errors=1
chk fxsw1 "$R/wp174-harness/fixtures/fx-sw1.php" "FX-SW1 DONE"
chk fxsl1 "$R/wp172-harness/fx-sl1.php" "FX-SL1 DONE" -d log_errors=0 -d display_errors=1
chk fxsl2 "$R/wp172-harness/fx-sl2.php" "FX-SL2 DONE" -d log_errors=0 -d display_errors=1
chk fxsl3 "$R/wp172-harness/fx-sl3.php" "FX-SL3 DONE" -d log_errors=0 -d display_errors=1
perl -e 'alarm 120; exec @ARGV or die' -- "$PIN" "$R/wp174-harness/fixtures/fx-sw2-gc.php" > "$OUT/sw2-pin.out" 2>&1
perl -e 'alarm 120; exec @ARGV or die' -- "$DEV" "$R/wp174-harness/fixtures/fx-sw2-gc.php" > "$OUT/sw2-dev.out" 2>&1
if diff -q "$OUT/sw2-pin.out" "$OUT/sw2-dev.out" >/dev/null; then echo "fx-sw2-gc: dev == pin BYTE-ID" >> "$OUT/verdetto.out"; else echo "fx-sw2-gc: dev ≠ pin" >> "$OUT/verdetto.out"; RC=2; fi
# giudice: parità del risultato di calls-dq (N ridotto) dev vs oracle
sed 's/60000000/600000/' "$R/wp182-harness/calls-dq.php" > "$OUT/calls-dq-small.php"
A=$("$ORACLE" "$OUT/calls-dq-small.php"); B=$("$DEV" "$OUT/calls-dq-small.php")
[ "$A" = "$B" ] && echo "calls-dq (N=600000): dev == oracle ($A)" >> "$OUT/verdetto.out" || { echo "calls-dq: dev $B ≠ oracle $A" >> "$OUT/verdetto.out"; RC=2; }
echo "ESITO rc=$RC" >> "$OUT/verdetto.out"; echo "rc=$RC" > "$OUT/done"
