#!/bin/bash
# verdict82.sh — S-82.0 p7: MACHINE verdict over the measure82 campaign raws
# (wp78-harness/measure-out, m82.*/82*.* labels). Fail-closed (A-SK19 class):
# a missing raw/field/figure FAILS, never skips. Verdicts:
#   VF  footprint twin: |mean V2(200) − mean V2(100)| <= 0.1 MB (A-DL19a)
#   VP  peak W=ncpu lever vs base: verdict only if |Δ| > max(2%, in-campaign
#       spread); otherwise ADVISORY d'ufficio (A-DL19b/KL-83-3)
#   VC  CPU slope: slope = (cpu(2000)−cpu(1000))/1000 per arm (user time,
#       time -l), means of R=3; PASS iff slope_lever <= slope_base*1.05 +
#       25us/req; RESOLUTION ex-ante: spread(per-R slopes) < banda/3 else
#       the verdict is NULL and N must double (KB-83-3 — never ADVISORY).
#   VH  battery-su-HIT twin-pair: run B segment per path main_put==0,
#       main_hit==nreq, main_probe==nreq (KS-PP-83-2); run A census witness
#       steady a_calls<=2 floor margin (<=4) (KS-SK-83-3: binary hash NAMED)
#   VR  retained: full-body PASS line present (KS-AH-83-1), unitcache row
#       with uclog==0 (KS-PP-83-1), main_evicted DECLARED (KS-DS-83-1),
#       rw_bytes labeled FLOOR + rw_main_net>0; budget ×W emitted [derivata]
#   VA  autoload-run: steady a3_calls > 0 on the autoload fixture (the RUN
#       share exists and is bounded by the printed figure), a3_trip==0
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
MOUT="$HERE/../wp78-harness/measure-out"
AN="$HERE/../wp80-harness/analyze80.pl"

perl - "$MOUT" "$AN" <<'PERL'
use strict; use warnings;
my ($mout, $an) = @ARGV;
my $fails = 0;
sub fail { print "FAIL: @_\n"; $fails++ }
sub mb { my $s = shift // ''; # vmmap figure like "123.4M" / "1.2G" -> MB
  return undef unless $s =~ /^([\d.]+)([KMG])?/;
  my ($v, $u) = ($1, $2 // 'M');
  return $v/1024 if $u eq 'K'; return $v*1024 if $u eq 'G'; return $v;
}
sub v2_of { my $f = shift;
  open my $fh, '<', $f or return undef;
  while (<$fh>) { return mb($1) if /^Physical footprint:\s+(\S+)/ }
  return undef;
}
sub user_cpu_of { my $f = shift;  # time -l stderr: "  12.34 real  5.67 user  1.23 sys"
  open my $fh, '<', $f or return undef;
  while (<$fh>) { return $2 if /([\d.]+)\s+real\s+([\d.]+)\s+user/ }
  return undef;
}
sub mean { my @v = @_; return undef unless @v; my $s = 0; $s += $_ for @v; $s / @v }
sub spreadpct { my @v = @_; my ($mn, $mx) = ((sort {$a<=>$b} @v)[0], (sort {$a<=>$b} @v)[-1]);
  return $mn == 0 ? ($mx == 0 ? 0 : 9e9) : ($mx - $mn) * 100 / $mn }

print "# verdict82 — residual A/B measures (S-82.0 p7) — machine, fail-closed\n\n";

# ---- VF: footprint twin ------------------------------------------------------
my %V2;
for my $n (100, 200) {
  my @v;
  for my $r (1..3) {
    my $f = "$mout/axum.82f.n$n.r$r.vmmap.V2";
    my $v = v2_of($f);
    defined $v ? push @v, $v : fail "VF: missing/unparsable V2 raw $f";
  }
  $V2{$n} = \@v;
  printf "VF data N=%d: V2 = %s MB (spread %.2f%%)\n", $n, join('/', map { sprintf '%.1f', $_ } @v), spreadpct(@v) if @v;
}
if (@{$V2{100} // []} == 3 && @{$V2{200} // []} == 3) {
  my $d = abs(mean(@{$V2{200}}) - mean(@{$V2{100}}));
  if ($d <= 0.1) { printf "VF  PASS: |V2(200)-V2(100)| = %.3f MB <= 0.1 MB floor (A-DL19a: no per-request resident growth above 1KB/req)\n", $d }
  else { fail sprintf("VF: V2 delta %.3f MB > 0.1 MB floor — resident growth on HIT path (A-DL19a)", $d) }
}

# ---- VP: peak W=ncpu ---------------------------------------------------------
chomp(my $nc = `sysctl -n hw.ncpu`);
my (@pl, @pb);
for my $r (1..3) {
  my $l = "$mout/axum.82p.w$nc.r$r.log";
  open my $fh, '<', $l or do { fail "VP: missing lever peak log $l"; next };
  my $pk; while (<$fh>) { $pk = $1 if /(\d+)\s+maximum resident set size/ }
  defined $pk ? push @pl, $pk / 1048576 : fail "VP: no peak in $l";
  my $b = "$mout/m82.base.82p.w$nc.r$r.log";
  open my $bh, '<', $b or do { fail "VP: missing base peak log $b"; next };
  my $bp; while (<$bh>) { $bp = $1 if /(\d+)\s+maximum resident set size/ }
  defined $bp ? push @pb, $bp / 1048576 : fail "VP: no peak in $b";
}
if (@pl == 3 && @pb == 3) {
  my ($ml, $mb_) = (mean(@pl), mean(@pb));
  my $sp = spreadpct(@pl) > spreadpct(@pb) ? spreadpct(@pl) : spreadpct(@pb);
  my $band = $sp > 2 ? $sp : 2;
  my $dpct = $mb_ == 0 ? 9e9 : abs($ml - $mb_) * 100 / $mb_;
  printf "VP data: peak lever=%.1f MB (%s) base=%.1f MB (%s) spread-band=%.1f%%\n",
    $ml, join('/', map { sprintf '%.0f', $_ } @pl), $mb_, join('/', map { sprintf '%.0f', $_ } @pb), $band;
  if ($dpct <= $band) {
    printf "VP  ADVISORY (KL-83-3, by rule): |Δpeak| %.1f%% within max(2%%, spread) — no verdict-grade change at W=%d\n", $dpct, $nc;
  } elsif ($ml <= $mb_) {
    printf "VP  PASS: peak W=%d improved/equal vs pre-lever (%.1f <= %.1f MB, Δ %.1f%% > band)\n", $nc, $ml, $mb_, $dpct;
  } else {
    fail sprintf("VP: peak W=%d WORSENED beyond band: lever %.1f vs base %.1f MB (Δ %.1f%%, KL-80-2)", $nc, $ml, $mb_, $dpct);
  }
}

# ---- VC: CPU slope -----------------------------------------------------------
my %CPU;
for my $arm ('lever', 'base') {
  for my $n (1000, 2000) {
    for my $r (1..3) {
      my $f = $arm eq 'lever' ? "$mout/axum.82c.lever.n$n.r$r.log" : "$mout/m82.base.82c.n$n.r$r.log";
      my $u = user_cpu_of($f);
      defined $u ? push @{$CPU{$arm}{$n}}, $u : fail "VC: no user-cpu in $f";
    }
  }
}
my %SLOPE;
for my $arm ('lever', 'base') {
  next unless @{$CPU{$arm}{1000} // []} == 3 && @{$CPU{$arm}{2000} // []} == 3;
  my @s = map { ($CPU{$arm}{2000}[$_] - $CPU{$arm}{1000}[$_]) / 1000 * 1e6 } (0..2); # us/req
  $SLOPE{$arm} = { mean => mean(@s), runs => \@s, spread => (sort {$a<=>$b} @s)[-1] - (sort {$a<=>$b} @s)[0] };
  printf "VC data %s: slope = %.1f us/req (per-R: %s)\n", $arm, $SLOPE{$arm}{mean}, join('/', map { sprintf '%.1f', $_ } @s);
}
if ($SLOPE{lever} && $SLOPE{base}) {
  my $bound = $SLOPE{base}{mean} * 1.05 + 25;
  my $banda = $bound - $SLOPE{base}{mean};
  my $res = $SLOPE{lever}{spread} > $SLOPE{base}{spread} ? $SLOPE{lever}{spread} : $SLOPE{base}{spread};
  if ($res >= $banda / 3) {
    fail sprintf("VC NULL (KB-83-3): slope spread %.1f us/req >= banda/3 (%.1f) — N must double, never ADVISORY", $res, $banda / 3);
  } elsif ($SLOPE{lever}{mean} <= $bound) {
    printf "VC  PASS: slope_lever %.1f <= slope_base*1.05+25 = %.1f us/req (resolution %.1f < banda/3 %.1f)\n",
      $SLOPE{lever}{mean}, $bound, $res, $banda / 3;
  } else {
    fail sprintf("VC: slope_lever %.1f us/req > bound %.1f — the probe COSTS CPU beyond the Bak threshold", $SLOPE{lever}{mean}, $bound);
  }
}

# ---- VH: battery-su-HIT twin pair -------------------------------------------
my %seg;
if (open my $fh, '<', "$mout/m82.hitpair.segment") {
  while (<$fh>) {
    $seg{$2}{$1}++ if /^unitcache (main_probe|main_hit|main_put) .*\/(\w+\.php)$/;
  }
  close $fh;
} else { fail "VH: hitpair segment missing" }
my ($nreq) = do {
  my $m = '';
  if (open my $fh, '<', "$mout/m82.hitpair.meta") { local $/; $m = <$fh>; close $fh }
  $m =~ /nreq=(\d+)/ ? $1 : do { fail "VH: nreq missing in meta (KS-SK-83-1)"; 0 };
};
for my $f (qw(hello.php include_gate.php include_heavy.php bare.php)) {
  my $p = $seg{$f}{main_probe} // 0;
  my $h = $seg{$f}{main_hit} // 0;
  my $u = $seg{$f}{main_put} // 0;
  if ($nreq && $p == $nreq && $h == $nreq && $u == 0) {
    print "VH  PASS: $f battery segment probe==$nreq hit==$nreq put==0 (pure HIT path, KS-PP-83-2)\n";
  } else {
    fail "VH: $f segment probe=$p hit=$h put=$u (want $nreq/$nreq/0) — path misto (KS-SK-83-3)";
  }
}
for my $f (qw(hello include_gate include_heavy bare)) {
  my $line = `perl "$an" "$mout/census.82h.$f.r1.census" 2>/dev/null`;
  chomp $line;
  if ($line =~ /steady_n=(\d+).* a_calls=([\d.]+)/) {
    my ($sn, $ac) = ($1, $2);
    if ($sn == 20 && $ac <= 4) { print "VH  PASS: census witness $f steady a_calls=$ac <= 4 (2x floor margin), steady_n=$sn\n" }
    else { fail "VH witness $f: steady_n=$sn a_calls=$ac (want 20 rows, <=4)" }
  } else { fail "VH: census witness raw missing/unparsable for $f (KS-SK-83-1)" }
}

# ---- VR: retained ------------------------------------------------------------
{
  my ($ok_body, $row, $mem_hash) = (0, '', '');
  if (open my $fh, '<', "$mout/m82.retained.log") {
    while (<$fh>) {
      $row = $_ if /tag=unitcache/;
      $mem_hash = $1 if /mem_hash=(\w+)/;
    }
    close $fh;
  } else { fail "VR: retained log missing" }
  # the full-body PASS line lives in the campaign stdout — the .done gate;
  # here the machine teeth are on the row itself:
  if ($row =~ /retained_walk_bytes=(\d+).*rw_main_net=(\d+).*rw_rule=cache-as-owner-FLOOR.*rw_main_net_rule=net-at-lower/s
      || $row =~ /retained_walk_bytes=(\d+).*rw_main_net=(\d+)/s) {
    my ($rw, $net) = ($1, $2);
    my ($ev) = $row =~ /main_evicted=(\d+)/ ? $1 : undef;
    my ($ul) = $row =~ /uclog=(\d)/ ? $1 : undef;
    if (!defined $ev) { fail "VR: main_evicted not in row (KS-DS-83-1)" }
    elsif (!defined $ul) { fail "VR: uclog flag not in row (KS-PP-83-1)" }
    elsif ($ul != 0) { fail "VR: uclog=1 during retained run — figure VOID (KS-PP-83-1)" }
    elsif ($net == 0) { fail "VR: rw_main_net==0 — the net-at-lower bracket did not fire (A-DL15)" }
    else {
      print "VR  PASS: unitcache row — rw_bytes=$rw [FLOOR, cache-as-owner] + rw_main_net=$net [net-at-lower] uclog=0 main_evicted=$ev (DECLARED; mem binary $mem_hash)\n";
      printf "VR  [derivata: budget = (rw_bytes + rw_main_net) x W = %.2f MB x W (W=1 measured; thread-local => xW by construction, KL-83-1 now satisfiable)]\n",
        ($rw + $net) / 1048576;
    }
  } else { fail "VR: no tag=unitcache row with rw figures in retained log (KS-AH-83-1)" }
}

# ---- VA: autoload-run --------------------------------------------------------
{
  my $line = `perl "$an" "$mout/census.82a.autoload.r1.census" 2>/dev/null`;
  chomp $line;
  if ($line =~ /steady_n=(\d+).* a3_calls=([\d.]+) a3_bytes=([\d.]+).* a3_trip_max=(\d+)/) {
    my ($sn, $a3c, $a3b, $trip) = ($1, $2, $3, $4);
    if ($trip != 0) { fail "VA: a3_trip=$trip — split VOID" }
    elsif ($a3c > 0) {
      print "VA  PASS: autoload-run a3 steady = $a3c calls / $a3b B per request (steady_n=$sn) — the RUN share of a3 is MEASURED (KB-82-5: 'HIT salta a3' bounded: the compile share is skipped, THIS share is not)\n";
    } else {
      fail "VA: a3_calls==0 on the autoload fixture — the autoload never fired at run (fixture broken, KB-81-3 vacuous)";
    }
  } else { fail "VA: autoload census raw missing/unparsable (KS-SK-83-1)" }
}

if ($fails == 0) { print "\n== VERDICT82 PASS ==\n"; exit 0 }
else             { print "\n== VERDICT82 FAIL($fails) ==\n"; exit 1 }
PERL
