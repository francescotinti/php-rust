#!/bin/bash
# s194-sonda-adiff.sh — S-194: day probe (direction only, CI may be running): user CPU of fixtures/adiff.php, oracle vs pin, 3 runs each.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin
W="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp194-harness"; F="$W/fixtures/adiff.php"; V="$W/s194-sonda-adiff.out"
O=/opt/homebrew/opt/php/bin/php; B="$HOME/Claude/php-rust-output/release/phpr"
echo "== sonda adiff $(date '+%F %T') load=$(sysctl -n vm.loadavg) cargo=$(pgrep -x cargo | wc -l | tr -d ' ') (direzione) ==" > "$V"
for i in 1 2 3; do
  for b in "$O" "$B"; do
    /usr/bin/time -p "$b" "$F" > "$W/out/adiff.stdout" 2> "$W/out/adiff.time"
    echo "$(basename "$b") out=$(tr -d '\n' < "$W/out/adiff.stdout") user=$(awk '/^user/{print $2}' "$W/out/adiff.time")" >> "$V"
  done
done
