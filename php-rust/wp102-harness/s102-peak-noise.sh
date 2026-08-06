#!/bin/bash
# s102-peak-noise.sh — S-102 punto 3 fase 1 (Concilio WP-103, KS-GR-102-2
# emendato da A-LE-103-2): BANDA RUMORE full-peak della gamba PHPR, mai
# misurata prima (solo quella oracle, ~10% intra-sera). R>=5 run full
# IDENTICI (stesso binario = pin S-100 f29883eb, stesso modo = off ESPLICITO),
# mediana+spread MECCANICI dai .time. La banda alimenta le bande UNILATERALI
# dell'A/B pin S-99<->S-100; spread >= 48 MiB => bisect VIETATO (KS-LE-103-2).
# USO: s102-peak-noise.sh [R]   (default 5; run SEQUENZIALI, macchina quieta)
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
R="${1:-5}"
WPDEV="/Users/francescotinti/Claude/wpdev"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp102-harness"
OUT="$H/peak-noise-out"; mkdir -p "$OUT"
PHPR_PIN="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s100-fix"
PIN_ATTESO=f29883eb432806ce
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
rm -f "$OUT/noise.done"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }

# pin FAIL-CLOSED
H_PIN="$(shasum -a 256 "$PHPR_PIN" | cut -c1-16)"
if [ "$H_PIN" != "$PIN_ATTESO" ]; then
  step "REFUSE: binario $H_PIN != pin atteso $PIN_ATTESO"
  echo "rc=65 $(date +%T)" > "$OUT/noise.done"; exit 65
fi
{
  echo "grade=NOISE-BAND  # R=$R run identici, pin S-100 modo off FISSATO"
  echo "phpr_pin=$H_PIN (stash phpr-s100-fix)"
  echo "modo=off (PHPR_REG_LOWER=0 esplicito, A-LE-103-2)"
  echo "epoch=$(date +%s)"
} > "$OUT/noise.identity"

reset_env() {
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || exit 3
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
}
"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=4 guard-backup" > "$OUT/noise.done"; exit 4; }
cd "$WPDEV" || { echo "rc=2 $(date +%T)" > "$OUT/noise.done"; exit 2; }

for i in $(seq 1 "$R"); do
  step "run $i/$R: full phpr pin S-100 off"
  reset_env
  MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER=0 /usr/bin/time -l "$PHPR_PIN" vendor/bin/phpunit \
    > "$OUT/full-$i.txt" 2> "$OUT/full-$i.time"
  step "run $i/$R rc=$? peak=$(sed -n 's/^ *\([0-9]*\) *peak memory footprint.*/\1/p' "$OUT/full-$i.time")"
done

find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$OUT/progress.txt" 2>&1

# mediana+spread MECCANICI (mai a mano)
perl - "$OUT" "$R" <<'PERL' > "$OUT/noise-band.out"
use strict; use warnings;
my ($out, $r) = @ARGV;
my @pk;
for my $i (1..$r) {
  open my $h, "<", "$out/full-$i.time" or die "full-$i.time: $!";
  my $pf;
  while (my $l = <$h>) { $pf = $1 if $l =~ /^\s*(\d+)\s+peak memory footprint/; }
  die "full-$i: peak assente" unless defined $pf;
  push @pk, $pf;
  printf "run%d_peak_B=%d run%d_peak_MiB=%.2f\n", $i, $pf, $i, $pf/1048576;
}
my @s = sort { $a <=> $b } @pk;
my $med = @s % 2 ? $s[$#s/2] : ($s[@s/2-1] + $s[@s/2]) / 2;
my $spread = $s[-1] - $s[0];
printf "R=%d\n", scalar @pk;
printf "mediana_B=%d mediana_MiB=%.2f\n", $med, $med/1048576;
printf "spread_B=%d spread_MiB=%.2f\n", $spread, $spread/1048576;
printf "min_MiB=%.2f max_MiB=%.2f\n", $s[0]/1048576, $s[-1]/1048576;
printf "verdetto_bisect=%s\n", ($spread/1048576 >= 48) ? "VIETATO (spread >= 48 MiB, KS-LE-103-2)" : "ammesso dal lato rumore";
PERL
step "noise-band DONE"
echo "rc=0 $(date +%T)" > "$OUT/noise.done"
