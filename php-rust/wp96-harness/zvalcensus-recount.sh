#!/bin/bash
# zvalcensus-recount.sh — S-96.0, passo 1 dell'ordine del Concilio WP-97:
# RICONTEGGIO di F1 e F2 dopo il fix di soundness A-TH-97-1 (la def non e' piu'
# sottratta sul contributo dell'arco eccezionale), i match ESAUSTIVI di
# effect()/renounce() (A-TH-97-2 / A-SK-97-2), le varianti che mancavano
# (NewAnonDeferred + DeclareDeferred, A-SK-97-1; debug_zval_refcount /
# debug_zval_dump, A-DS-97-5 / A-MS-97-5) e il contatore nuovo
# would_take_safe_ref (A-MS-97-1).
#
# Condizioni IDENTICHE a zvalcensus-f1.sh/f2.sh: stesso media group, stesso
# reset DB, stessa guardia uploads, MIMALLOC_PURGE_DELAY=0, binario di
# STRUMENTAZIONE da target-dir separato. Il binario di parita'
# ~/Claude/php-rust-output/release/phpr NON e' toccato — e la sua sha e'
# trascritta nell'identity PRIMA e DOPO il run (A-PP-97-1: la provenienza si
# trascrive dalla macchina, non si dichiara).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
WPDEV="/Users/francescotinti/Claude/wpdev"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp96-harness"
OUT="$H/recount-out"; mkdir -p "$OUT"
PHPR="/Volumes/Extreme Pro/Claude/phpr-census-target/release/phpr"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
rm -f "$OUT/recount.done" "$OUT/zvalcensus-recount.raw"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }

{
  echo "phpr_census=$(shasum -a 256 "$PHPR" | cut -c1-16)"
  echo "phpr_parity_before=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)"
  echo "head=$(git -C "$H/.." rev-parse --short=12 HEAD)"
  echo "tree_dirty=$(git -C "$H/.." status --porcelain | wc -l | tr -d ' ')"
  echo "epoch_start=$(date +%s)"
} > "$OUT/recount.identity"

"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=4 guard-backup" > "$OUT/recount.done"; exit 4; }
mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || { echo "rc=3 mysql" > "$OUT/recount.done"; exit 3; }
find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
cd "$WPDEV" || { echo "rc=2 cd" > "$OUT/recount.done"; exit 2; }

step "media phpr-census RECOUNT"
MIMALLOC_PURGE_DELAY=0 PHPR_ZVAL_CENSUS="$OUT/zvalcensus-recount.raw" \
  "$PHPR" vendor/bin/phpunit --group media \
  > "$OUT/media-recount.txt" 2> "$OUT/media-recount.err"
rc=$?
step "media phpr-census rc=$rc"

find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$OUT/progress.txt" 2>&1
{
  echo "phpr_parity_after=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)"
  echo "epoch_end=$(date +%s)"
} >> "$OUT/recount.identity"
step "RECOUNT DONE rc=$rc"
echo "rc=$rc $(date +%T)" > "$OUT/recount.done"

# Somma delle righe per-processo: output MACCHINA (il gate cifre vuole raw).
perl - "$OUT/zvalcensus-recount.raw" <<'PERL' > "$OUT/zvalcensus-recount.sum"
use strict; use warnings;
my %s; my $n = 0;
while (<>) {
  next unless /^zvalcensus /;
  $n++;
  while (/(\w+)=(\d+)/g) { $s{$1} += $2 }
}
print "zvalcensus-recount processes=$n";
for my $k (qw(slot_reads slot_reads_rc slot_reads_avoided would_take would_take_rc would_take_safe would_take_safe_rc would_take_safe_str would_take_safe_ref sites_total sites_movable sites_safe)) {
  printf " %s=%d", $k, $s{$k} // 0;
}
print "\n";
PERL
