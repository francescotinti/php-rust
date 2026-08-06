#!/bin/bash
# s102-peak-ab.sh — S-102 punto 3 fase 2 (criterio in s102-peak-criterio.out,
# scritto PRIMA): A/B full-peak pin S-99 <-> pin S-100, modo FISSATO off/off
# (A-LE-103-2), ABAB INTERLEAVED stessa sera (KS-GR-103-3), R=5 per braccio.
# Statistica: mediana per braccio + spread max-min (mai la media) + delta per
# coppia adiacente (A-GR-103-1: >=3/5 coppie a segno opposto => finestra
# sospetta). Verdetto dalla REGOLA DI CHIUSURA pre-registrata.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
R="${1:-5}"
WPDEV="/Users/francescotinti/Claude/wpdev"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp102-harness"
OUT="$H/peak-ab-out"; mkdir -p "$OUT"
PIN_A="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s99-sigillo"
PIN_B="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s100-fix"
ATT_A=52330330873f0132
ATT_B=f29883eb432806ce
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
rm -f "$OUT/ab.done"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }

H_A="$(shasum -a 256 "$PIN_A" | cut -c1-16)"
H_B="$(shasum -a 256 "$PIN_B" | cut -c1-16)"
if [ "$H_A" != "$ATT_A" ] || [ "$H_B" != "$ATT_B" ]; then
  step "REFUSE: pin A=$H_A (att $ATT_A) B=$H_B (att $ATT_B)"
  echo "rc=65 $(date +%T)" > "$OUT/ab.done"; exit 65
fi
{
  echo "grade=AB-PEAK  # ABAB interleaved, R=$R per braccio, modo off/off FISSATO"
  echo "pin_A=$H_A (s99-sigillo)  pin_B=$H_B (s100-fix)"
  echo "epoch=$(date +%s)"
} > "$OUT/ab.identity"

reset_env() {
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || exit 3
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
}
"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=4 guard-backup" > "$OUT/ab.done"; exit 4; }
cd "$WPDEV" || { echo "rc=2 $(date +%T)" > "$OUT/ab.done"; exit 2; }

for i in $(seq 1 "$R"); do
  for arm in A B; do
    case "$arm" in A) BIN="$PIN_A" ;; B) BIN="$PIN_B" ;; esac
    step "coppia $i/$R braccio $arm: full off"
    reset_env
    MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER=0 /usr/bin/time -l "$BIN" vendor/bin/phpunit \
      > "$OUT/full-$arm$i.txt" 2> "$OUT/full-$arm$i.time"
    step "coppia $i/$R braccio $arm rc=$? peak=$(sed -n 's/^ *\([0-9]*\) *peak memory footprint.*/\1/p' "$OUT/full-$arm$i.time")"
  done
done

find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$OUT/progress.txt" 2>&1

perl - "$OUT" "$R" <<'PERL' > "$OUT/ab-verdetto.out"
use strict; use warnings;
my ($out, $r) = @ARGV;
sub peak { my ($f) = @_; open my $h, "<", "$out/full-$f.time" or die "$f: $!";
  my $pf; while (my $l = <$h>) { $pf = $1 if $l =~ /^\s*(\d+)\s+peak memory footprint/; }
  die "$f: peak assente" unless defined $pf; return $pf; }
my (@a, @b, @pair);
for my $i (1..$r) {
  my ($pa, $pb) = (peak("A$i"), peak("B$i"));
  push @a, $pa; push @b, $pb; push @pair, $pb - $pa;
  printf "coppia%d peak_A_MiB=%.2f peak_B_MiB=%.2f delta_BA_MiB=%+.2f\n",
    $i, $pa/1048576, $pb/1048576, ($pb-$pa)/1048576;
}
sub med { my @s = sort { $a <=> $b } @_; @s % 2 ? $s[$#s/2] : ($s[@s/2-1]+$s[@s/2])/2 }
sub spread { my @s = sort { $a <=> $b } @_; $s[-1] - $s[0] }
my ($ma, $mb) = (med(@a), med(@b));
my ($sa, $sb) = (spread(@a), spread(@b));
my $band = $sa > $sb ? $sa : $sb;
my $d = $mb - $ma;
printf "mediana_A_MiB=%.2f spread_A_MiB=%.2f\n", $ma/1048576, $sa/1048576;
printf "mediana_B_MiB=%.2f spread_B_MiB=%.2f\n", $mb/1048576, $sb/1048576;
printf "banda_MiB=%.2f delta_mediane_MiB=%+.2f\n", $band/1048576, $d/1048576;
my $opp = grep { ($_ <=> 0) != ($d <=> 0) && $_ != 0 } @pair;
printf "coppie_segno_opposto=%d/%d%s\n", $opp, scalar @pair,
  ($opp >= 3 ? " FINESTRA SOSPETTA (A-GR-103-1): si ripete" : "");
if (abs($d) <= $band) {
  print "verdetto=RUMORE (|delta| <= banda): voce CHIUSA senza bisect (regola pre-registrata)\n";
} else {
  printf "verdetto=CRESCITA REALE (|delta| > banda); bisect %s (spread %.2f MiB vs soglia 48)\n",
    ($band/1048576 >= 48 ? "VIETATO (KS-LE-103-3)" : "ammesso"), $band/1048576;
}
PERL
step "AB-PEAK DONE"
echo "rc=0 $(date +%T)" > "$OUT/ab.done"
