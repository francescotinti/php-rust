#!/bin/bash
# s105-pair-chain.sh — catena DETACHED della coppia WP bimodale S-105
# (salda il debito KS-PE-106-2: trigger = leva args promossa).
# Esegue pair105.sh off poi on IN SEQUENZA (mai in parallelo), marker
# .done finale con gli rc di entrambe le gambe. Da lanciare col
# daemonizer (double-fork+setsid) — mai come task figlio della sessione.
set -u
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
OUT="$H/pair-chain"
mkdir -p "$OUT"
rm -f "$OUT/pair-chain.done"
echo "$(date +%H:%M:%S) chain start (phpr pin atteso d4d0fa5217515dd9)" >> "$OUT/progress.txt"
pin=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)
if [ "$pin" != "d4d0fa5217515dd9" ]; then
  echo "rc=66 pin-mismatch $pin" > "$OUT/pair-chain.done"; exit 66
fi
"$H/pair105.sh" off >> "$OUT/progress.txt" 2>&1
rc_off=$?
echo "$(date +%H:%M:%S) off done rc=$rc_off" >> "$OUT/progress.txt"
"$H/pair105.sh" on >> "$OUT/progress.txt" 2>&1
rc_on=$?
echo "$(date +%H:%M:%S) on done rc=$rc_on" >> "$OUT/progress.txt"
echo "rc_off=$rc_off rc_on=$rc_on $(date +%T)" > "$OUT/pair-chain.done"
