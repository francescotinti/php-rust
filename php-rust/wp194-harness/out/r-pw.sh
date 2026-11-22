#!/bin/bash
# r-pw.sh — S-194 ledger follow-up: instructions per password_hash+password_verify pair, A arm against the oracle (differential over two counts).
W="$(cd "$(dirname "$0")/.." && pwd)"
for eng in "$W/out/phpr-orm-istr-A" /opt/homebrew/opt/php/bin/php; do
  for cost in 10 12; do
    a=$( { /usr/bin/time -l "$eng" "$W/out/r-pw.php" 2 $cost >/dev/null; } 2>&1 | awk '/instructions retired/ {print $1}')
    b=$( { /usr/bin/time -l "$eng" "$W/out/r-pw.php" 6 $cost >/dev/null; } 2>&1 | awk '/instructions retired/ {print $1}')
    echo "$(basename "$eng") cost=$cost per hash+verify: $(( (b - a) / 4 ))"
  done
done
grep -rn "cost" "$HOME/Claude/wpdev/tests/phpunit/includes/bootstrap.php" "$HOME/Claude/wpdev/tests/phpunit/includes/functions.php" 2>/dev/null | cut -c1-160 | sed -n '1,6p'
