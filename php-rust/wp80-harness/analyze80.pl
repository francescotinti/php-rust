#!/usr/bin/perl
# analyze80.pl — S-80.0.6: per-run steady-state means from measure78 census
# raws. Usage: analyze80.pl <run.census> [warmup]
# Steady = rows AFTER the first `warmup` (default 10) census lines. Emits one
# line: run, N_steady, mean of every numeric field found on the census rows
# (census: or census-cli: shape), max depth/inflight/trip observed.
# Figures are GROSS churn (A-DL1/A-DL10) — the label is carried through.
use strict; use warnings;
my $file = shift or die "usage: analyze80.pl <run.census> [warmup]\n";
my $warmup = shift // 10;
open my $fh, '<', $file or die "$file: $!";
my (@rows);
while (<$fh>) {
    chomp;
    next unless /^census(-cli)?: /;
    my %f;
    while (/([a-z0-9_]+)=(\d+)/g) { $f{$1} = $2 }
    push @rows, \%f;
}
close $fh;
my $n = @rows;
die "$file: 0 census rows\n" unless $n;
my @steady = @rows[$warmup .. $#rows];
my $ns = @steady;
die "$file: no steady rows (n=$n warmup=$warmup)\n" unless $ns;
my %sum; my %max;
for my $r (@steady) {
    for my $k (keys %$r) {
        $sum{$k} += $r->{$k};
        $max{$k} = $r->{$k} if !defined $max{$k} || $r->{$k} > $max{$k};
    }
}
my @order = qw(total_calls total_bytes a_calls a_bytes a1_calls a1_bytes
               a2_calls a2_bytes a3_calls a3_bytes b_calls b_bytes
               c_calls c_bytes resid_calls resid_bytes retain_len live_objs);
my $out = "$file steady_n=$ns gross=1";
for my $k (@order) {
    next unless defined $sum{$k};
    $out .= sprintf " %s=%.1f", $k, $sum{$k} / $ns;
}
for my $k (qw(depth_max inflight_max a3_trip)) {
    $out .= " ${k}_max=" . ($max{$k} // 'absent');
}
print "$out\n";
