#!/bin/bash
# s150-flip-handler.sh <corpus-out-dir> — gestione FAIL-CLOSED dei flip corpus
# della leva BT1 (criterio s150-criterio-promo.md p.4). Legge l'esito del
# corpus-gate live, esige che OGNI divergenza sia: (a) un nome della famiglia
# backtrace congelata in s150-flip-famiglia.txt, (b) mai un EXTRA (regressione).
# Poi aggiorna congelato (.fails) e golden (.tsv) SOLO per quei nomi, con la
# stessa macchina-chunk di corpus-gate.sh (copia dichiarata del parsing), e
# committa l'atto con l'elenco per NOME. Il chiamante DEVE poi rieseguire
# corpus-gate.sh --replay e ottenere rc=0. rc: 0=aggiornato; 1=violazione.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
FROZ="$REPO/wp109-harness/corpus-gate"
FAM="$REPO/wp150-harness/s150-flip-famiglia.txt"
CO="${1:?uso: s150-flip-handler.sh <corpus-out-dir>}"
OUTV="$REPO/wp150-harness/promo-out/flip-handler.out"

perl - "$CO" "$FROZ" "$FAM" > "$OUTV" 2>&1 <<'PERL'
use strict; use warnings; use Digest::SHA qw(sha256_hex);
my ($co, $froz, $famf) = @ARGV;
my %fam; { open my $h, '<', $famf or die "$famf: $!";
           while (<$h>) { chomp; $fam{$_} = 1 if length } }
my $viol = 0;
my %flip_all;
for my $mode (qw(off on)) {
  my $norm = "$co/$mode.norm";
  my $all; { open my $h, '<', $norm or die "$norm: $!"; local $/; $all = <$h>; }
  my %c; while ($all =~ /^--- (.+?) ---\n(.*?)(?=^--- .+? ---$|\z)/msg) { $c{$1} = $2; }
  my %froznames; { open my $f, '<', "$froz/corpus-s109-$mode.fails" or die $!;
                   while (<$f>) { chomp; $froznames{$_} = 1 if length } }
  my @extra   = sort grep { !$froznames{$_} } keys %c;
  my @flipped = sort grep { !exists $c{$_} } keys %froznames;
  if (@extra) { print "VIOLAZIONE($mode): EXTRA (regressione) non ammessi:\n";
                print "  +$_\n" for @extra; $viol = 1; }
  for my $n (@flipped) {
    if (!$fam{$n}) { print "VIOLAZIONE($mode): flip FUORI famiglia: $n\n"; $viol = 1; }
  }
  # contenuto mutato nei test ANCORA falliti: ammesso solo dentro la famiglia
  my %gold; { open my $g, '<', "$froz/golden-content-$mode.tsv" or die $!;
              while (<$g>) { chomp; my ($d,$n) = split /\t/, $_, 2; $gold{$n} = $d if defined $n } }
  my @mutati;
  for my $k (sort keys %c) {
    next unless exists $gold{$k};
    my $d = sha256_hex($c{$k});
    if ($gold{$k} ne $d) {
      if ($fam{$k}) { push @mutati, $k }
      # i 3 settype-NaN sono il carve-out storico: contenuto instabile, golden non arbitra
      elsif ($k =~ m{/settype_(?:bool|int|string)_nan_with_error_handler3\.phpt$}) { }
      else { print "VIOLAZIONE($mode): contenuto mutato FUORI famiglia: $k\n"; $viol = 1; }
    }
  }
  if ($viol) { next }
  # atto di aggiornamento per questo modo
  { open my $f, '>', "$froz/corpus-s109-$mode.fails.new" or die $!;
    for my $n (sort keys %froznames) { next if !exists $c{$n}; print $f "$n\n" }
    close $f; }
  { open my $g, '>', "$froz/golden-content-$mode.tsv.new" or die $!;
    open my $old, '<', "$froz/golden-content-$mode.tsv" or die $!;
    my %mut = map { $_ => 1 } @mutati;
    while (<$old>) { chomp; my ($d,$n) = split /\t/, $_, 2;
      next unless defined $n;
      next if !exists $c{$n};                      # flippato: riga via
      $d = sha256_hex($c{$n}) if $mut{$n};         # mutato in famiglia: digest nuovo
      print $g "$d\t$n\n"; }
    close $old; close $g; }
  print "FLIP($mode): ", scalar(@flipped), " flippati PASS (per NOME):\n";
  print "  -$_\n" for @flipped;
  print "MUTATI($mode) in famiglia: ", scalar(@mutati), "\n";
  print "  ~$_\n" for @mutati;
  $flip_all{$_} = 1 for @flipped;
  my $tot = scalar(grep { exists $c{$_} } keys %froznames);
  print "CONGELATO($mode): ", scalar(keys %froznames), " -> $tot\n";
}
if ($viol) { print "FLIP-HANDLER rc=1 (violazione: NIENTE aggiornato)\n"; exit 1 }
for my $mode (qw(off on)) {
  rename "$froz/corpus-s109-$mode.fails.new", "$froz/corpus-s109-$mode.fails" or die $!;
  rename "$froz/golden-content-$mode.tsv.new", "$froz/golden-content-$mode.tsv" or die $!;
}
print "FLIP-HANDLER rc=0\n";
PERL
rc=$?
cat "$OUTV"
[ "$rc" = 0 ] || exit 1
cd "$REPO" || exit 4
git add wp109-harness/corpus-gate/corpus-s109-off.fails \
        wp109-harness/corpus-gate/corpus-s109-on.fails \
        wp109-harness/corpus-gate/golden-content-off.tsv \
        wp109-harness/corpus-gate/golden-content-on.tsv || exit 4
M=$(mktemp)
{ echo "S-150 flip corpus BT1 dichiarati per NOME (handler fail-closed; criterio p.4)"
  echo
  grep -E '^(  -|  ~|CONGELATO)' "$OUTV"; } > "$M"
git commit -F "$M" > /dev/null && git push > /dev/null
rm -f "$M"
echo "FLIP-HANDLER: congelato+golden aggiornati e committati"
exit 0
