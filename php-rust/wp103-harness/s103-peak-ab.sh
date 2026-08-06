#!/bin/bash
# s103-peak-ab.sh — S-103 punto 1: RERUN dell'A/B full-peak S-99 <-> S-100
# dopo il VOID dell'audit finestra (peak-ab-audit-verdetto.out, protocollo
# PRE-REGISTRATO lì PRIMA del lancio):
#   - ABAB interleaved, stessi pin, modo off/off, R=7 (zona marginale
#     consumata dal passaggio a R>=7);
#   - UN giro full di WARMUP non misurato PRIMA della coppia 1 (la coppia 1
#     del run S-102 era ~85 MiB sopra le altre su ENTRAMBI i bracci);
#   - giudizio contro la BANDA FASE-1 (34,64 MiB), NON lo spread intra-run;
#     tetto spread 1,5x fase-1 (51,96 MiB) pena VOID (A-GR-104-2/A-LE-104-5);
#   - coppie adiacenti pubblicate; segni opposti in quota >=3/5 => sospetta.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
R="${1:-7}"
WPDEV="/Users/francescotinti/Claude/wpdev"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp103-harness"
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
  echo "grade=AB-PEAK-R7  # rerun post-VOID: warmup non misurato + banda fase-1"
  echo "pin_A=$H_A (s99-sigillo)  pin_B=$H_B (s100-fix)"
  echo "banda_fase1_MiB=34.64 tetto_spread_MiB=51.96 (pre-registrati)"
  echo "epoch=$(date +%s)"
} > "$OUT/ab.identity"

reset_env() {
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" || exit 3
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
}
"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { echo "rc=4 guard-backup" > "$OUT/ab.done"; exit 4; }
cd "$WPDEV" || { echo "rc=2 $(date +%T)" > "$OUT/ab.done"; exit 2; }

# WARMUP pre-registrato: un giro full (braccio A) NON misurato, scartato
# per costruzione — scalda FS-cache/JIT del sistema prima della coppia 1.
step "warmup (non misurato): full off su pin A"
reset_env
MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER=0 "$PIN_A" vendor/bin/phpunit \
  > "$OUT/warmup.txt" 2>&1
step "warmup rc=$? (scartato per costruzione)"

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
# Banda e tetto PRE-REGISTRATI (peak-ab-audit-verdetto.out, WP-104):
my $BAND  = 34.64 * 1048576;   # fase-1 (s102-peak-noise)
my $CEIL  = 51.96 * 1048576;   # 1,5x fase-1, pena VOID
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
my $smax = $sa > $sb ? $sa : $sb;
my $d = $mb - $ma;
printf "mediana_A_MiB=%.2f spread_A_MiB=%.2f\n", $ma/1048576, $sa/1048576;
printf "mediana_B_MiB=%.2f spread_B_MiB=%.2f\n", $mb/1048576, $sb/1048576;
printf "banda_fase1_MiB=34.64 tetto_MiB=51.96 delta_mediane_MiB=%+.2f\n", $d/1048576;
my $opp = grep { ($_ <=> 0) != ($d <=> 0) && $_ != 0 } @pair;
printf "coppie_segno_opposto=%d/%d%s\n", $opp, scalar @pair,
  ($opp*5 >= $r*3 ? " FINESTRA SOSPETTA (quota >=3/5): si ripete" : "");
if ($smax > $CEIL) {
  printf "verdetto=VOID (spread %.2f MiB > tetto 51.96): la finestra non giudica\n", $smax/1048576;
} elsif (abs($d) <= $BAND) {
  print "verdetto=RUMORE (|delta| <= banda fase-1 34.64): voce CHIUSA senza bisect\n";
} else {
  printf "verdetto=CRESCITA REALE (|delta| > banda fase-1); bisect %s (spread %.2f MiB vs soglia 48)\n",
    ($smax/1048576 >= 48 ? "VIETATO (KS-LE-103-3)" : "ammesso"), $smax/1048576;
}
PERL
step "AB-PEAK-R7 DONE"
echo "rc=0 $(date +%T)" > "$OUT/ab.done"
