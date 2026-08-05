#!/bin/bash
# s100-corpus-diff.sh — A-KL-101-4: la DEFINIZIONE OPERATIVA del «diff
# riga-per-riga off↔on» dei corpus (gate della promozione, KS-KL-100-1).
#
# Definizione (pre-registrata; criterio = ZERO differenze):
#   1. Il corpus Zend gira nei DUE modi (PHPR_REG_LOWER=0 e =1, valori
#      ESPLICITI della lista chiusa) con `--isolate --list-fails`.
#   2. Un test PASS in un modo e' un test il cui output combacia con
#      l'EXPECT: due PASS sono per costruzione equivalenti riga-per-riga.
#      Quindi il confronto per-test si decide su: (a) INSIEME dei nomi
#      FAIL identico nei due modi; (b) per ogni FAIL comune, il DIFF
#      actual-vs-expected stampato dal runner byte-IDENTICO nei due modi.
#   3. Normalizzazione: tr -d '\0' (i log contengono NUL). Nessun'altra:
#      i chunk `--- path ---` del runner non contengono tempi.
#   4. A-KL-101-3 / KS-KL-101-3: i nomi estratti quadrano col conteggio
#      `failures: N` dichiarato dallo strumento, pena gate VOID.
# Ogni differenza esce PER NOME (gate-diff-fail-set-not-count).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
RUNNER="$HOME/Claude/php-rust-output/release/phpt-runner"
CORPUS="/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
OUT="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp100-harness/corpus-diff"
mkdir -p "$OUT"
step(){ echo "$(date +%H:%M:%S) $1" | tee -a "$OUT/progress.txt"; }

for mode in off on; do
  case "$mode" in off) reg=0 ;; on) reg=1 ;; esac
  step "corpus $mode (PHPR_REG_LOWER=$reg esplicito): run --list-fails"
  /usr/bin/env PHPR_REG_LOWER="$reg" "$RUNNER" --isolate --list-fails "$CORPUS" \
    > "$OUT/corpus-diff-$mode.log" 2>&1
  tr -d '\0' < "$OUT/corpus-diff-$mode.log" > "$OUT/corpus-diff-$mode.norm"
done

step "confronto per-test dei chunk FAIL"
set -o pipefail  # il rc del gate e' quello del perl, non del tee
perl - "$OUT/corpus-diff-off.norm" "$OUT/corpus-diff-on.norm" <<'PERL' | tee -a "$OUT/progress.txt"
use strict; use warnings;
my ($f_off, $f_on) = @ARGV;
sub chunks {
  my ($f) = @_;
  open my $h, '<', $f or die "$f: $!";
  local $/; my $all = <$h>;
  # A-KL-101-3: conteggio dichiarato dallo strumento.
  my ($decl) = $all =~ /^failures: (\d+)$/m;
  my %c;
  # Ogni chunk: "--- <path> ---\n<detail>" fino al prossimo "--- " o alla fine.
  while ($all =~ /^--- (.+?) ---\n(.*?)(?=^--- .+? ---$|\z)/msg) {
    $c{$1} = $2;
  }
  return (\%c, $decl // -1);
}
my ($off, $n_off) = chunks($f_off);
my ($on,  $n_on)  = chunks($f_on);
my $void = 0;
for ([$n_off, scalar keys %$off, 'off'], [$n_on, scalar keys %$on, 'on']) {
  my ($decl, $extr, $m) = @$_;
  if ($decl != $extr) { print "GATE VOID ($m): chunk estratti $extr != dichiarati $decl (KS-KL-101-3)\n"; $void = 1; }
}
exit 65 if $void;
my @only_off = sort grep { !exists $on->{$_} } keys %$off;
my @only_on  = sort grep { !exists $off->{$_} } keys %$on;
my @differ   = sort grep { exists $on->{$_} && $off->{$_} ne $on->{$_} } keys %$off;
print "fail solo OFF: ", scalar @only_off, "\n";  print "  $_\n" for @only_off;
print "fail solo ON: ",  scalar @only_on,  "\n";  print "  $_\n" for @only_on;
print "diff per-test DIVERSO: ", scalar @differ, "\n";  print "  $_\n" for @differ;
if (@only_off + @only_on + @differ) { print "CORPUS-DIFF: DIVERSO\n"; exit 1 }
print "CORPUS-DIFF: ZERO differenze per-test off<->on (criterio A-KL-101-4 SODDISFATTO)\n";
PERL
rc=$?
step "CORPUS-DIFF DONE rc=$rc"
echo "rc=$rc $(date +%T)" > "$OUT/corpus-diff.done"
exit "$rc"
