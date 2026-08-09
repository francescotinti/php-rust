#!/bin/bash
# s125-pair-intercal.sh — S-125 p.1: rimisura full/media WP sul pin s124,
# coppia bimodale N=2 per modo a gambe INTERCALATE off1→on1→off2→on2
# (criterio: s125-criterio-pair.md, committato PRIMA del run — rev. S-124 #1).
# Ricetta pair109 INVARIATA (script collaudato riusato); qui solo
# orchestrazione + archivio per gamba + rapporti CROSS-ORACLE dai .time.
# Pin misurato = release/phpr; l'identità per gamba la scrive pair109.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp125-harness"
H109="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp109-harness"
OUT="$H/pair-out"; mkdir -p "$OUT"
step(){ echo "$(date +%H:%M:%S) $1" >> "$OUT/progress.txt"; }
: > "$OUT/progress.txt"
rm -f "$OUT/pair-intercal.done"
PIN_ATTESO="c5ba2573a23adf69"
PIN_REALE="$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16)"
if [ "$PIN_REALE" != "$PIN_ATTESO" ]; then
  step "ABORT: release/phpr=$PIN_REALE != pin s124 $PIN_ATTESO"
  echo "rc=9 pin-mismatch" > "$OUT/pair-intercal.done"; exit 9
fi
RC=0
for leg in 1 2; do
  for mode in off on; do
    step "gamba $mode-$leg START"
    "$H109/pair109.sh" "$mode" >> "$OUT/progress.txt" 2>&1
    lrc=$?
    step "gamba $mode-$leg rc=$lrc"
    [ "$lrc" = 0 ] || RC=1
    rm -rf "$OUT/leg$leg-$mode"
    mv "$H109/pair-out-$mode" "$OUT/leg$leg-$mode"
    mv "$H109/pair109-ratios-$mode.out" "$OUT/leg$leg-$mode/ratios.out"
  done
done

# ---- rapporti per gamba + CROSS-ORACLE (output macchina dai .time)
perl - "$OUT" > "$OUT/cross-ratios.out" <<'PERL'
use strict; use warnings;
my $out = shift @ARGV;
sub t { my ($f) = @_; open my $h, "<", $f or die "$f: $!";
  my %v; while (my $l = <$h>) {
    $v{user} = $1 if $l =~ /([\d.]+)\s+user/; $v{sys} = $1 if $l =~ /([\d.]+)\s+sys/;
    $v{pf}   = $1 if $l =~ /^\s*(\d+)\s+peak memory footprint/; }
  $v{cpu} = $v{user} + $v{sys}; return \%v; }
my (%o, %p, %om, %pm);
for my $leg (1, 2) { for my $mode (qw(off on)) {
  my $d = "$out/leg$leg-$mode";
  $o{"$mode$leg"}  = t("$d/full-oracle.time");  $p{"$mode$leg"}  = t("$d/full-phpr.time");
  $om{"$mode$leg"} = t("$d/media-oracle.time"); $pm{"$mode$leg"} = t("$d/media-phpr.time");
}}
print "cross-ratios S-125 (full master cpu = user+sys; 4 gambe intercalate off1 on1 off2 on2; pin s124)\n";
print "grade=VERDICT  # derivazione meccanica dai .time\n";
for my $k (sort keys %o) {
  printf "full_oracle_%s=%.2f full_phpr_%s=%.2f media_oracle_%s=%.2f media_phpr_%s=%.2f peak_phpr_%s_MiB=%.1f\n",
    $k, $o{$k}{cpu}, $k, $p{$k}{cpu}, $k, $om{$k}{cpu}, $k, $pm{$k}{cpu}, $k, $p{$k}{pf}/1048576;
}
print "matrice full phpr(riga)/oracle(colonna):\n";
for my $pk (sort keys %p) {
  my @cells;
  for my $ok (sort keys %o) { push @cells, sprintf "%s/%s=%.3f", $pk, $ok, $p{$pk}{cpu}/$o{$ok}{cpu}; }
  print join("  ", @cells), "\n";
}
my @oc = map { $o{$_}{cpu} } sort keys %o;
my @pc = map { $p{$_}{cpu} } sort keys %p;
my ($omin, $omax) = (sort { $a <=> $b } @oc)[0, -1];
my ($pmin, $pmax) = (sort { $a <=> $b } @pc)[0, -1];
printf "oracle_full_N4: min=%.2f max=%.2f spread=%.1f%%\n", $omin, $omax, 100*($omax-$omin)/$omin;
printf "phpr_full_N4: min=%.2f max=%.2f spread=%.1f%%\n", $pmin, $pmax, 100*($pmax-$pmin)/$pmin;
printf "full_intervallo=%.3f-%.3f (min/max su tutte le 16 celle)\n", $pmin/$omax, $pmax/$omin;
for my $mode (qw(off on)) {
  my @r = map { $pm{"$mode$_"}{cpu}/$om{"$mode$_"}{cpu} } (1, 2);
  printf "media_%s_ratio_N2=%.3f/%.3f\n", $mode, @r;
  my @pk = map { $p{"$mode$_"}{pf}/1048576 } (1, 2);
  printf "full_peak_%s_MiB_N2=%.1f/%.1f\n", $mode, @pk;
}
PERL
cat "$OUT/cross-ratios.out"
echo "rc=$RC" > "$OUT/pair-intercal.done"
step "DONE rc=$RC"
exit "$RC"
