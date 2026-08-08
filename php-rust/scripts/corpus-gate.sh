#!/bin/bash
# corpus-gate.sh — gate corpus CANONICO per pin/promozione (az. rev. S-117 n.1-2):
# giudica NOMI (== congelato s109, 1415 per modo) E CONTENUTO (digest per test
# == golden wp109-harness/corpus-gate/golden-content-<modo>.tsv, nato dai raw
# del pin s117 verificati dal revisore) E off↔on byte-id — macchina chunk s102,
# carve-out = lista chiusa 3 settype-NaN. Contratto di modo A-KL-101-6 (valore
# ESPLICITO), assert conteggi↔nomi KS-KL-101-3 (VOID), rc SEMPRE in file.
# USO:  corpus-gate.sh <runner-bin> <outdir>            (run live ×2 modi)
#       corpus-gate.sh --replay <off.norm> <on.norm> <outdir>   (giudica raw esistenti)
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
CORPUS="/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
FROZ="$REPO/wp109-harness/corpus-gate"
GRC=0

if [ "${1:-}" = "--replay" ]; then
  NORM_OFF="${2:?uso: --replay <off.norm> <on.norm> <outdir>}"
  NORM_ON="${3:?}"; OUT="${4:?}"; mkdir -p "$OUT"
  MODE_SRC="replay"
else
  RUNNER="${1:?uso: corpus-gate.sh <runner-bin> <outdir> | --replay <off.norm> <on.norm> <outdir>}"
  OUT="${2:?}"; mkdir -p "$OUT"
  MODE_SRC="live runner=$(shasum -a 256 "$RUNNER" | cut -c1-16)"
  for m in off on; do
    case "$m" in off) reg=0 ;; on) reg=1 ;; esac
    /usr/bin/env PHPR_REG_LOWER="$reg" "$RUNNER" --isolate --list-fails "$CORPUS" \
      > "$OUT/$m.log" 2>&1
    echo $? > "$OUT/$m.rc"
    tr -d '\0' < "$OUT/$m.log" > "$OUT/$m.norm"
  done
  NORM_OFF="$OUT/off.norm"; NORM_ON="$OUT/on.norm"
fi
VERD="$OUT/corpus-gate.out"
{ echo "corpus-gate ($MODE_SRC) $(date '+%F %T')"; } > "$VERD"
note(){ echo "$1" | tee -a "$VERD"; }

# --- per modo: nomi vs congelato + contenuto vs golden (macchina chunk) ---
for m in off on; do
  NORM_VAR="NORM_OFF"; [ "$m" = on ] && NORM_VAR="NORM_ON"
  eval "NORM=\$$NORM_VAR"
  perl - "$NORM" "$FROZ/corpus-s109-$m.fails" "$FROZ/golden-content-$m.tsv" "$OUT" "$m" >> "$VERD" 2>&1 <<'PERL'
use strict; use warnings; use Digest::SHA qw(sha256_hex);
my ($norm, $frozen, $golden, $out, $mode) = @ARGV;
my %nondet = map { $_ => 1 } (
  '/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests/type_coercion/settype/settype_bool_nan_with_error_handler3.phpt',
  '/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests/type_coercion/settype/settype_int_nan_with_error_handler3.phpt',
  '/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests/type_coercion/settype/settype_string_nan_with_error_handler3.phpt',
);
my $all; { open my $h, '<', $norm or die "$norm: $!"; local $/; $all = <$h>; }
my ($decl) = $all =~ /^failures: (\d+)$/m;
my %c; while ($all =~ /^--- (.+?) ---\n(.*?)(?=^--- .+? ---$|\z)/msg) { $c{$1} = $2; }
my $rc = 0;
# VOID: conteggi vs chunk estratti (KS-KL-101-3)
my $extr = scalar keys %c;
if (!defined $decl || $decl != $extr) {
  print "gate CORPUS($mode): VOID — chunk estratti $extr != dichiarati @{[$decl // 'n/d']}\n"; $rc = 65;
} else {
  # nomi == congelato
  open my $f, '<', $frozen or die "$frozen: $!";
  my %froz; while (<$f>) { chomp; $froz{$_} = 1 if length }
  my @extra = sort grep { !$froz{$_} } keys %c;
  my @missing = sort grep { !exists $c{$_} } keys %froz;
  if (@extra || @missing) {
    print "gate CORPUS($mode) NOMI: DIVERGE dal congelato (extra @{[scalar @extra]}, mancanti @{[scalar @missing]})\n";
    print "  +$_\n" for @extra; print "  -$_\n" for @missing; $rc = 1;
  } else { print "gate CORPUS($mode) NOMI: == congelato s109 ($extr)\n"; }
  # contenuto == golden (digest per test, carve-out escluso)
  open my $g, '<', $golden or die "$golden: $!";
  my %gold; while (<$g>) { chomp; my ($d, $n) = split /\t/, $_, 2; $gold{$n} = $d if defined $n }
  my @cdiff;
  for my $k (sort keys %c) {
    next if $nondet{$k};
    my $d = sha256_hex($c{$k});
    push @cdiff, $k if !exists $gold{$k} || $gold{$k} ne $d;
  }
  if (@cdiff) {
    print "gate CORPUS($mode) CONTENUTO: ", scalar @cdiff, " test con chunk DIVERSO dal golden\n";
    print "  $_\n" for @cdiff; $rc = 1;
  } else { print "gate CORPUS($mode) CONTENUTO: digest == golden (", scalar(grep { !$nondet{$_} } keys %c), " test, carve-out 3)\n"; }
}
open my $r, '>', "$out/$mode.judge.rc" or die; print $r "$rc\n"; close $r;
exit 0;
PERL
  jrc=$(cat "$OUT/$m.judge.rc")
  [ "$jrc" != 0 ] && GRC=1
done

# --- off↔on byte-id fuori carve-out (macchina s102 invariata) ---
perl - "$NORM_OFF" "$NORM_ON" > "$OUT/offon.txt" 2>&1 <<'PERL'
use strict; use warnings;
my ($f_off, $f_on) = @ARGV;
sub chunks { my ($f) = @_; open my $h, '<', $f or die "$f: $!"; local $/; my $all = <$h>;
  my ($decl) = $all =~ /^failures: (\d+)$/m; my %c;
  while ($all =~ /^--- (.+?) ---\n(.*?)(?=^--- .+? ---$|\z)/msg) { $c{$1} = $2; }
  return (\%c, $decl // -1); }
my ($off, $n_off) = chunks($f_off);
my ($on,  $n_on)  = chunks($f_on);
my %nondet = map { $_ => 1 } (
  '/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests/type_coercion/settype/settype_bool_nan_with_error_handler3.phpt',
  '/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests/type_coercion/settype/settype_int_nan_with_error_handler3.phpt',
  '/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests/type_coercion/settype/settype_string_nan_with_error_handler3.phpt',
);
my $void = 0;
for ([$n_off, scalar keys %$off, 'off'], [$n_on, scalar keys %$on, 'on']) {
  my ($decl, $extr, $m) = @$_;
  if ($decl != $extr) { print "GATE VOID ($m): chunk estratti $extr != dichiarati $decl\n"; $void = 1; }
}
exit 65 if $void;
my @only_off = sort grep { !exists $on->{$_} } keys %$off;
my @only_on  = sort grep { !exists $off->{$_} } keys %$on;
my @differ   = grep { !$nondet{$_} }
               sort grep { exists $on->{$_} && $off->{$_} ne $on->{$_} } keys %$off;
print "fail solo OFF: ", scalar @only_off, "\n";  print "  $_\n" for @only_off;
print "fail solo ON: ",  scalar @only_on,  "\n";  print "  $_\n" for @only_on;
print "diff per-test DIVERSO: ", scalar @differ, "\n";  print "  $_\n" for @differ;
if (@only_off + @only_on + @differ) { print "CORPUS-DIFF: DIVERSO\n"; exit 1 }
print "CORPUS-DIFF: ZERO differenze per-test off<->on fuori carve-out\n";
PERL
orc=$?
echo "$orc" > "$OUT/offon.rc"
note "gate CORPUS off↔on: rc=$orc — $(tail -1 "$OUT/offon.txt")"
[ "$orc" != 0 ] && GRC=1

echo "$GRC" > "$OUT/corpus-gate.rc"
note "CORPUS-GATE COMPLESSIVO: rc=$GRC (nomi E contenuto E off↔on; rc in $OUT/corpus-gate.rc)"
exit "$GRC"
