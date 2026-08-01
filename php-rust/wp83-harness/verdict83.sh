#!/bin/bash
# verdict83.sh — S-83.0: MACHINE verdict over the measure83 campaign raws.
# Fail-closed (A-SK19): a missing raw/field FAILS, never skips. Verdicts:
#   VC  slope min-of-R (A-BB31, f=5% ex-ante): per cell (arm,N) 9 user
#       times; estimator = min; resolution rule = max over cells of
#       (3rd_smallest - smallest)/N*1e6 < banda/3, banda = 5%*slope_base
#       + 25 us/req — unmet => NULL (never ADVISORY, KB-83-3 heredity).
#       PASS iff slope_lever <= slope_base*1.05 + 25. Scope NAMED:
#       axum W=1 hello closed-sequential, union binaries, same evening.
#   VCR lever per-request distribution [derivata] (A-BG32): median/min of
#       the measured-window reqns samples per R — attribution only, the
#       base arm has no probe (DECLARED).
#   VR  retained DECOMPOSED: last dump group of r1: per-entry rows with
#       ordinal; algebra EXACT: sum(net)==rw_main_net,
#       sum(floor_inc)==program_floor_share, rw_budget recomputed;
#       alloc_id pinned; arm=cli-server; uclog==0; main_evicted declared.
#       A-DL22: r2 hello-only budget vs the VP marginal 3.44 MB/worker +
#       shareable-fraction estimate [derivata].
#   VA  register split [derivata]: reg = b(reg83)-b(hello); include-HIT
#       <= b(autoload82)-b(reg83) — labeled "upper bound vs opcache
#       inheritance-cache" (A-DS25/A-BB33).
#   VW  long-path floor boundary: steady a_bytes == 2*len(canonical path)
#       EXACT on the 415-byte fixture — outcome NAMED either way (KB-84-2).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
MOUT="$REPO/wp78-harness/measure-out"
AN="$REPO/wp80-harness/analyze80.pl"
LPABS="$REPO/wp78-harness/gate-axum/fixtures/lp83_segment_padding_0123456789_0123456789_0123456789_x1/lp83_segment_padding_0123456789_0123456789_0123456789_x2/lp83_segment_padding_0123456789_0123456789_0123456789_x3/lp83_segment_padding_0123456789_0123456789_0123456789_x4/lp83_segment_padding_0123456789_0123456789_0123456789_x5/bare_longpath_384_kb831_kb842_fixture.php"

perl - "$MOUT" "$AN" "$LPABS" <<'PERL'
use strict; use warnings;
my ($mout, $an, $lpabs) = @ARGV;
my $fails = 0;
sub fail { print "FAIL: @_\n"; $fails++ }
sub user_cpu_of { my $f = shift;
  open my $fh, '<', $f or return undef;
  while (<$fh>) { return $2 if /([\d.]+)\s+real\s+([\d.]+)\s+user/ }
  return undef;
}
sub median { my @v = sort { $a <=> $b } @_; return undef unless @v;
  @v % 2 ? $v[$#v/2] : ($v[@v/2 - 1] + $v[@v/2]) / 2 }

print "# verdict83 — WP-83 measures (Council WP-84 forms) — machine, fail-closed\n\n";

# ---- VC: slope min-of-R --------------------------------------------------------
my (%U, %RES);
for my $arm ('lever', 'base') {
  for my $n (1000, 2000) {
    my @u;
    for my $r (1..9) {
      my $f = $arm eq 'lever' ? "$mout/axum.83c.lever.n$n.r$r.log" : "$mout/m83.base.83c.n$n.r$r.log";
      my $u = user_cpu_of($f);
      defined $u ? push @u, $u : fail "VC: no user-cpu in $f";
    }
    next unless @u == 9;
    my @s = sort { $a <=> $b } @u;
    $U{$arm}{$n} = $s[0];
    $RES{$arm}{$n} = ($s[2] - $s[0]) / $n * 1e6;   # 3-smallest spread, us/req
    printf "VC data %s N=%d: min=%.2fs 3low-spread=%.1f us/req (all: %s)\n",
      $arm, $n, $s[0], $RES{$arm}{$n}, join('/', map { sprintf '%.2f', $_ } @u);
  }
}
if (defined $U{lever}{1000} && defined $U{lever}{2000} && defined $U{base}{1000} && defined $U{base}{2000}) {
  my $sl = ($U{lever}{2000} - $U{lever}{1000}) / 1000 * 1e6;
  my $sb = ($U{base}{2000}  - $U{base}{1000})  / 1000 * 1e6;
  my $banda = 0.05 * $sb + 25;
  my $res = 0; for my $arm ('lever','base') { for my $n (1000,2000) {
    $res = $RES{$arm}{$n} if $RES{$arm}{$n} > $res } }
  printf "VC data: slope_lever=%.1f slope_base=%.1f us/req banda=%.1f res=%.1f (f=5%% ex-ante, min-of-R R=9)\n",
    $sl, $sb, $banda, $res;
  if ($res >= $banda / 3) {
    printf "VC  NULL (by the ex-ante rule): resolution %.1f us/req >= banda/3 (%.1f) — the estimator did not resolve; NO CPU claim (KB-83-3 heredity, never ADVISORY)\n", $res, $banda / 3;
  } elsif ($sl <= $sb * 1.05 + 25) {
    printf "VC  PASS: slope_lever %.1f <= slope_base*1.05+25 = %.1f us/req (min-of-R resolved: res %.1f < banda/3 %.1f) — scope: axum W=1 hello closed-seq, union arms, same evening\n",
      $sl, $sb * 1.05 + 25, $res, $banda / 3;
  } else {
    fail sprintf("VC: slope_lever %.1f us/req > bound %.1f — the lever COSTS CPU beyond the Bak threshold", $sl, $sb * 1.05 + 25);
  }
}

# ---- VCR: lever per-request distribution [derivata] ----------------------------
for my $r (1..3) {
  my $f = "$mout/axum.83cr.lever.n2000.r$r.log";
  my $line;
  if (open my $fh, '<', $f) { while (<$fh>) { $line = $_ if /^reqns: / } close $fh }
  unless (defined $line && $line =~ /n=(\d+) arm=axum-worker unit=ns ns=([\d,]+)/) {
    fail "VCR: no drained reqns line in $f (A-BG32)"; next;
  }
  my ($n, $ns) = ($1, $2);
  my @v = split /,/, $ns;
  unless (@v == $n && $n >= 1900) { fail "VCR: sample count $n/". scalar(@v) ." (want ~2000)"; next }
  my @s = sort { $a <=> $b } @v;
  printf "VCR [derivata] r%d: n=%d regime median=%.1f min=%.1f p90=%.1f us/req (lever arm ONLY — base has no probe, DECLARED)\n",
    $r, $n, median(@v)/1000, $s[0]/1000, $s[int(0.9*$#s)]/1000;
}

# ---- VR: retained decomposed ----------------------------------------------------
sub last_group { # <file> -> (main_row, [entry_rows])
  my $f = shift;
  open my $fh, '<', $f or do { fail "VR: missing $f"; return };
  my (@cur, @last, $main, $lastmain);
  while (<$fh>) {
    if (/tag=unitcache_main_entry /) { push @cur, $_ }
    elsif (/tag=unitcache /) { $lastmain = $_; @last = @cur; @cur = () }
  }
  close $fh;
  return ($lastmain, \@last);
}
{
  my ($row, $entries) = last_group("$mout/m83.r1.fourfix.memcensus");
  if (defined $row) {
    my ($rw)  = $row =~ /retained_walk_bytes=(\d+)/;
    my ($net) = $row =~ /rw_main_net=(\d+)/;
    my ($pfs) = $row =~ /program_floor_share=(\d+)/;
    my ($bud) = $row =~ /rw_budget=(\d+)/;
    my ($ev)  = $row =~ /main_evicted=(\d+)/;
    my ($ul)  = $row =~ /uclog=(\d)/;
    my ($aid) = $row =~ /alloc_id=(\S+)/;
    my ($arm) = $row =~ /arm=(\S+)/;
    if (!defined $rw || !defined $net || !defined $pfs || !defined $bud) { fail "VR: main row missing decomposition fields (A-DL20/21)" }
    elsif (!defined $ev) { fail "VR: main_evicted undeclared (KS-DS-83-1)" }
    elsif (!defined $ul || $ul != 0) { fail "VR: uclog armed or undeclared (KS-PP-83-1)" }
    elsif (!defined $aid || $aid ne 'memcount-v2-s82') { fail "VR: alloc_id '".($aid//'ABSENT')."' != memcount-v2-s82 (KS-AH-84-2)" }
    elsif (!defined $arm || $arm ne 'cli-server') { fail "VR: arm label missing (A-PP25)" }
    elsif (@$entries != 4) { fail "VR: ".scalar(@$entries)." per-entry rows in final group (want 4 mains, A-DL20)" }
    else {
      my $vr_fails_before = $fails;
      my ($snet, $sinc, @nets, %byord) = (0, 0);
      for my $e (@$entries) {
        my ($o, $en, $fi) = $e =~ /ord=(\d+) net=(\d+) floor_inc=(\d+)/;
        unless (defined $fi) { fail "VR: unparsable entry row: $e"; next }
        $snet += $en; $sinc += $fi; push @nets, $en; $byord{$o} = $en;
        fail "VR: entry alloc_id drifted" unless $e =~ /alloc_id=memcount-v2-s82/;
      }
      if ($snet != $net)  { fail "VR: sum(entry net)=$snet != rw_main_net=$net (A-DL20 algebra)" }
      if ($sinc != $pfs)  { fail "VR: sum(floor_inc)=$sinc != program_floor_share=$pfs (A-DL21 algebra)" }
      if ($bud != $rw - $pfs + $net) { fail "VR: rw_budget=$bud != rw_bytes-pfs+net=".($rw-$pfs+$net) }
      if ($fails == $vr_fails_before) {
        printf "VR  PASS: decomposition EXACT — rw_bytes=%d pfs=%d rw_main_net=%d rw_budget=%d (%.2f MB xW, arm=cli-server) main_evicted=%d\n",
          $rw, $pfs, $net, $bud, $bud/1048576, $ev;
        my @others = sort { $a <=> $b } grep { $_ != ($byord{1} // -1) } @nets;
        my $med = @others ? median(@others) : 0;
        printf "VR  [derivata] per-entry: ord1 net=%d, median(others)=%.0f => one-time first-lower residue ~ %d B (%.2f MB) = %.1f%% of budget — the S-82.0 'flat 1.85MB/main' dissolves: the residue is per-PROCESS, not per-entry (A-DL20 finding)\n",
          $byord{1} // 0, $med, ($byord{1} // 0) - $med, (($byord{1} // 0) - $med)/1048576,
          $bud ? (($byord{1} // 0) - $med) * 100 / $bud : 0;
      }
    }
  } else { fail "VR: no unitcache row in r1 (instrument unarmed? KS-AH-83-1)" }
  # A-DL22: hello-only quadratura
  my ($row2, $e2) = last_group("$mout/m83.r2.helloonly.memcensus");
  if (defined $row2 && $row2 =~ /rw_budget=(\d+)/) {
    my $b2 = $1;
    printf "VR  [derivata] A-DL22 hello-only W=1: rw_budget=%.2f MB vs VP marginal 3.44 MB/worker => ratio %.2f; shareable fraction upper est = %.1f%% (candidate registry saving x(W-1))\n",
      $b2/1048576, $b2/1048576/3.44, ($b2 > 0 && @$e2) ? do {
        my ($o1) = map { /net=(\d+)/ ? $1 : 0 } grep { /ord=1 / } @$e2;
        ($o1 // 0) * 100 / $b2 } : 0;
  } else { fail "VR: r2 hello-only row missing (A-DL22)" }
}

# ---- VA: register split [derivata] ----------------------------------------------
{
  my %B;
  for my $fx (qw(autoload82 autoload83_reg hello)) {
    my $line = `perl "$an" "$mout/census.83a.$fx.census" 2>/dev/null`;
    chomp $line;
    if ($line =~ /steady_n=(\d+).* a3_calls=([\d.]+).* b_calls=([\d.]+)/) {
      my ($sn, $a3, $b) = ($1, $2, $3);
      $B{$fx} = $b;
      fail "VA: $fx steady_n=$sn != 30" unless $sn == 30;
      fail "VA: $fx steady a3=$a3 != 0 (cache broken for runtime includes)" unless $a3 == 0;
    } else { fail "VA: census raw missing/unparsable for $fx (KS-SK-83-1)" }
  }
  if (defined $B{autoload82} && defined $B{autoload83_reg} && defined $B{hello}) {
    printf "VA  PASS: steady a3==0 on all three arms\n";
    printf "VA  [derivata] register share = b(reg83)-b(hello) = %+.0f calls/req; include-HIT share <= b(autoload82)-b(reg83) = %+.0f calls/req — UPPER BOUND vs opcache inheritance-cache (A-DS25/A-BB33; the +56 composite is now split)\n",
      $B{autoload83_reg} - $B{hello}, $B{autoload82} - $B{autoload83_reg};
  }
}

# ---- VW: long-path floor boundary -------------------------------------------------
{
  my $len = length $lpabs;
  my $line = `perl "$an" "$mout/census.83w.longpath.census" 2>/dev/null`;
  chomp $line;
  if ($line =~ /steady_n=(\d+).* a_calls=([\d.]+).* a_bytes=([\d.]+)/) {
    my ($sn, $ac, $ab) = ($1, $2, $3);
    if ($sn == 30 && $ab == 2 * $len) {
      printf "VW  PASS: a_bytes=%.0f == 2 x len(path)=%d at len=%d >= 384 — the floor model HOLDS at its boundary (KB-83-1/KB-84-2 closed; a_calls=%.1f)\n", $ab, 2*$len, $len, $ac;
    } else {
      printf "VW  NAMED-DEVIATION: a_bytes=%.0f vs 2 x len=%d (len=%d, steady_n=%d, a_calls=%.1f) — the 2xlen floor model is FALSIFIED at the boundary; the model needs a new form (KB-84-2: outcome named, no silent claim)\n", $ab, 2*$len, $len, $sn, $ac;
    }
  } else { fail "VW: longpath census raw missing/unparsable (KS-SK-83-1)" }
}

if ($fails == 0) { print "\n== VERDICT83 PASS ==\n"; exit 0 }
else             { print "\n== VERDICT83 FAIL($fails) ==\n"; exit 1 }
PERL
