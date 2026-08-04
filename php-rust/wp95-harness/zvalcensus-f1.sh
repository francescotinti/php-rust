#!/bin/bash
# zvalcensus-f1.sh — S-95.0 A-ZV2 fase F1: il conteggio would_take sul media
# group, STESSE condizioni di zvalcensus-before.out (stessa suite, stesso DB
# reset, stessa guardia uploads, MIMALLOC_PURGE_DELAY=0), binario di
# STRUMENTAZIONE (--features zval-census) da target-dir separato.
# Il binario di parità ~/Claude/php-rust-output/release/phpr NON è toccato.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
WPDEV="/Users/francescotinti/Claude/wpdev"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp95-harness"
OUT="$H/f1-out"; mkdir -p "$OUT"
PHPR="/Volumes/Extreme Pro/Claude/phpr-census-target/release/phpr"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
rm -f "$OUT/f1.done" "$OUT/zvalcensus-f1.raw"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }

{
  echo "phpr_census=$(shasum -a 256 "$PHPR" | cut -c1-16)"
  echo "phpr_parity=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)"
  echo "head=$(git -C "$H/.." rev-parse --short=12 HEAD)"
  echo "epoch=$(date +%s)"
} > "$OUT/f1.identity"

"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=4 guard-backup" > "$OUT/f1.done"; exit 4; }
mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || { echo "rc=3 mysql" > "$OUT/f1.done"; exit 3; }
find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
cd "$WPDEV" || { echo "rc=2 cd" > "$OUT/f1.done"; exit 2; }

step "media phpr-census F1"
MIMALLOC_PURGE_DELAY=0 PHPR_ZVAL_CENSUS="$OUT/zvalcensus-f1.raw" \
  "$PHPR" vendor/bin/phpunit --group media \
  > "$OUT/media-f1.txt" 2> "$OUT/media-f1.err"
rc=$?
step "media phpr-census rc=$rc"

find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$OUT/progress.txt" 2>&1
step "F1 DONE rc=$rc"
echo "rc=$rc $(date +%T)" > "$OUT/f1.done"

# Somma delle righe per-processo: output MACCHINA (il gate cifre vuole raw).
perl - "$OUT/zvalcensus-f1.raw" <<'PERL' > "$OUT/zvalcensus-f1.sum"
use strict; use warnings;
my %s; my $n = 0;
while (<>) {
  next unless /^zvalcensus /;
  $n++;
  while (/(\w+)=(\d+)/g) { $s{$1} += $2 }
}
print "zvalcensus-f1 processes=$n";
for my $k (qw(slot_reads slot_reads_rc slot_reads_avoided would_take would_take_rc sites_total sites_movable)) {
  printf " %s=%d", $k, $s{$k} // 0;
}
print "\n";
PERL
