#!/bin/bash
# verdict84.sh — S-84.0: MACHINE verdict over the measure84 campaign raws.
# Fail-closed (A-SK19): a missing raw/field FAILS, never skips.
# A-SK34 (Council WP-85, Klabnik Q3): NAMED-DEVIATION is TWO-TOOTHED — an
# INVALID measure (steady_n != 30, unparsable raw) is a FAIL, never a
# deviation; a model deviation is reachable ONLY from a valid measure.
# A-DL26: every memory figure prints BYTES FIRST, MiB second.
# Verdicts:
#   VP    A-BB34 rerun (KB-85-2): 3 peak logs W=ncpu; pin = 232±1 MiB
#         identity. Valid+inside => PIN CONFIRMED; valid+outside =>
#         NAMED-OUTCOME (pin moved, new value quoted — deliberation input);
#         invalid => FAIL.
#   VDL24 the ×W decision (KL-85-1): per-thread ord=1 net rows from the W=2
#         teardown dump; second thread's net(ord1) >= 3,000,000 B =>
#         PER-THREAD (budget ×W holds); <= 500,000 B => PER-PROCESS
#         (budget rewrites: residue + W×(budget−residue)); in between =>
#         FAIL (ambiguous — no silent classification).
#   VA2   register split with A-DS29 refusal (KS-DS-85-1): every cited
#         fixture must carry a ledgered full-body PASS at THIS campaign rev
#         BEFORE the counted window — else the per-fixture figure is VOID
#         and the verdict FAILS.
#   VW2   bisection at the NAMED site (A-BB38/KB-85-3): site = std
#         run_path_with_cstr MAX_STACK_ALLOCATION=384 (heap CString(len+1)
#         fallback), reached by the TWO path syscalls in main_unit_key
#         (fs::canonicalize + fs::metadata). Model: a_calls=2+2·[len>=384],
#         a_bytes=2·len+2·(len+1)·[len>=384]. KB-85-3: a_calls not in
#         {2,4} or a_bytes off-model at EITHER side => model REJECTED =>
#         NAMED-DEVIATION. Control: hello (len=98) from phase A2.
#   Guard KS-PP-85-1: any reqns line in this campaign's raws without
#         `w=1` in-band would void per-request slope samples — no slope
#         phase ran; the guard reports count found (expected 0).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
MOUT="$REPO/wp78-harness/measure-out"
AN="$REPO/wp80-harness/analyze80.pl"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
FLEDGER="$HERE/evidence/fixture-oracle.ledger"
D1="vw84_bisect_$(printf 'a%.0s' $(seq 1 88))"
D2="vw84_pad_$(printf 'b%.0s' $(seq 1 91))"
ABS383="$FIXDIR/$D1/$D2/f$(printf 'c%.0s' $(seq 1 87)).php"
ABS384="$FIXDIR/$D1/$D2/f$(printf 'c%.0s' $(seq 1 88)).php"
ABSHELLO="$FIXDIR/hello.php"

perl - "$MOUT" "$AN" "$ABS383" "$ABS384" "$ABSHELLO" "$FLEDGER" <<'PERL'
use strict; use warnings;
my ($mout, $an, $abs383, $abs384, $abshello, $fledger) = @ARGV;
my $fails = 0;
sub fail { print "FAIL: @_\n"; $fails++ }

print "# verdict84 — S-84.0 peak-deliberation measures (Council WP-85 forms) — machine, fail-closed\n\n";

# ---- VP: A-BB34 rerun ---------------------------------------------------------
chomp(my $nc = `sysctl -n hw.ncpu`);
{
  my @pk;
  for my $r (1..3) {
    my $l = "$mout/axum.84p.w$nc.r$r.log";
    open my $fh, '<', $l or do { fail "VP: missing peak log $l"; next };
    my $pk; while (<$fh>) { $pk = $1 if /(\d+)\s+maximum resident set size/ }
    defined $pk ? push @pk, $pk : fail "VP: no peak in $l";
  }
  if (@pk == 3) {
    my @mb = map { $_ / 1048576 } @pk;
    printf "VP data: peaks = %s B = %s MiB (W=%d, R=3, union post-partition)\n",
      join('/', @pk), join('/', map { sprintf '%.1f', $_ } @mb), $nc;
    my $inside = 1;
    for (@mb) { $inside = 0 if $_ < 231 || $_ > 233 }
    if ($inside) {
      printf "VP  PASS: A-BB34 pin CONFIRMED post-partition — all three peaks inside 232±1 MiB (KB-85-2 lifted: peak ×W figures citable from THIS rerun)\n";
    } else {
      printf "VP  NAMED-OUTCOME: pin MOVED — peaks outside 232±1 MiB; the deliberation uses THESE values (KB-85-2 satisfied by the rerun; no silent claim)\n";
    }
  }
}

# ---- VDL24: the ×W decision -----------------------------------------------------
{
  my $f = "$mout/m84.dl24.memcensus";
  open my $fh, '<', $f or do { fail "VDL24: missing $f" };
  my (%net, $meta);
  if ($fh) {
    while (<$fh>) {
      if (/tag=unitcache_main_entry thr=(\d+) ord=1 net=(\d+)/) {
        # one ord=1 row per thread; a duplicate thr would be a protocol
        # break (two mains in one worker cache) — fail loud.
        fail "VDL24: duplicate ord=1 row for thr=$1" if exists $net{$1};
        $net{$1} = $2;
        fail "VDL24: row without pinned alloc_id (KS-AH-85-3)" unless /alloc_id=memcount-v2-s82/;
        fail "VDL24: row without arm label" unless /arm=axum-worker/;
      }
      $meta = $_ if /phase=dl24/;
    }
    close $fh;
  }
  unless ($meta && $meta =~ /w=2/) { fail "VDL24: campaign meta line missing or not w=2" }
  my @thr = sort keys %net;
  if (@thr != 2) { fail "VDL24: ".scalar(@thr)." threads with ord=1 rows (want 2)" }
  else {
    my ($a, $b) = ($net{$thr[0]}, $net{$thr[1]});
    my ($first, $second) = $a >= $b ? ($a, $b) : ($b, $a);
    printf "VDL24 data: net(ord1) first=%d B = %.2f MiB, second=%d B = %.2f MiB (thr %s/%s)\n",
      $first, $first/1048576, $second, $second/1048576, $thr[0], $thr[1];
    if ($second >= 3_000_000) {
      printf "VDL24 PER-THREAD: the second worker RE-PAYS the first lower (%d B = %.2f MiB >= 3,000,000 B) — the ×W budget formula HOLDS: budget × W (KL-85-1 satisfied)\n",
        $second, $second/1048576;
    } elsif ($second <= 500_000) {
      printf "VDL24 PER-PROCESS: the second worker does NOT re-pay (%d B <= 500,000 B) — the budget REWRITES: residue %d B = %.2f MiB once + W × (budget − residue) (KL-85-1 satisfied, formula rewritten in the measured direction)\n",
        $second, $first - $second, ($first - $second)/1048576;
    } else {
      fail sprintf("VDL24: AMBIGUOUS — second thread net(ord1)=%d B sits between the 500,000/3,000,000 B decision bands; no silent classification (A-SK34 class)", $second);
    }
  }
}

# ---- VA2: register split under A-DS29 refusal ------------------------------------
{
  # rev of THIS campaign from the dl24 meta line
  my $rev;
  if (open my $mh, '<', "$mout/m84.dl24.memcensus") {
    while (<$mh>) { $rev = $1 if /git=(\w+).*phase=dl24/ }
    close $mh;
  }
  fail "VA2: campaign rev unreadable from dl24 meta" unless $rev;
  my %ledgered;
  if (open my $lh, '<', $fledger) {
    while (<$lh>) {
      $ledgered{$1} = 1 if /fixture=(\S+) rev=$rev .*verdict=PASS/ && defined $rev;
    }
    close $lh;
  } else { fail "VA2: fixture-oracle ledger unreadable (A-DS29)" }
  my %B;
  for my $fx (qw(autoload82 autoload83_reg hello)) {
    unless ($ledgered{"$fx.php"}) {
      fail "VA2: fixture $fx.php has NO ledgered full-body PASS at rev ".($rev//'?')." — per-fixture figure VOID (KS-DS-85-1)";
      next;
    }
    my $line = `perl "$an" "$mout/census.84a.$fx.census" 2>/dev/null`;
    chomp $line;
    if ($line =~ /steady_n=(\d+).* a3_calls=([\d.]+).* b_calls=([\d.]+)/) {
      my ($sn, $a3, $b) = ($1, $2, $3);
      $B{$fx} = $b;
      fail "VA2: $fx steady_n=$sn != 30 — INVALID measure is FAIL, never deviation (A-SK34)" unless $sn == 30;
      fail "VA2: $fx steady a3=$a3 != 0 (cache broken for runtime includes)" unless $a3 == 0;
    } else { fail "VA2: census raw missing/unparsable for $fx (KS-SK-83-1)" }
  }
  if (defined $B{autoload82} && defined $B{autoload83_reg} && defined $B{hello} && !grep { !$ledgered{"$_.php"} } qw(autoload82 autoload83_reg hello)) {
    printf "VA2 PASS: steady a3==0 on all three arms, EVERY fixture body ledger-PASS at the campaign rev (KS-DS-85-1 lifted — the WP-83 VOID is cured)\n";
    printf "VA2 [derivata] register share = b(reg83)-b(hello) = %+.1f calls/req; include-HIT share <= b(autoload82)-b(reg83) = %+.1f calls/req — UPPER BOUND vs opcache inheritance-cache (A-DS25/A-BB33)\n",
      $B{autoload83_reg} - $B{hello}, $B{autoload82} - $B{autoload83_reg};
  }
}

# ---- VW2: bisection at the NAMED site ---------------------------------------------
{
  my %want = (
    len383 => { len => length($abs383), calls => 2 },
    len384 => { len => length($abs384), calls => 4 },
  );
  fail "VW2: fixture length drifted (abs383=".length($abs383).")" unless length($abs383) == 383;
  fail "VW2: fixture length drifted (abs384=".length($abs384).")" unless length($abs384) == 384;
  my $model_ok = 1; my $valid = 1;
  for my $side (qw(len383 len384)) {
    my $line = `perl "$an" "$mout/census.84w.$side.census" 2>/dev/null`;
    chomp $line;
    unless ($line =~ /steady_n=(\d+).* a_calls=([\d.]+).* a_bytes=([\d.]+)/) {
      fail "VW2: census raw missing/unparsable for $side"; $valid = 0; next;
    }
    my ($sn, $ac, $ab) = ($1, $2, $3);
    if ($sn != 30) { fail "VW2: $side steady_n=$sn != 30 — INVALID measure is FAIL (A-SK34)"; $valid = 0; next }
    my $len = $want{$side}{len};
    my $exp_calls = $want{$side}{calls};
    my $exp_bytes = $exp_calls == 2 ? 2*$len : 2*$len + 2*($len+1);
    printf "VW2 data %s: len=%d a_calls=%.1f a_bytes=%.0f (model expects calls=%d bytes=%d)\n",
      $side, $len, $ac, $ab, $exp_calls, $exp_bytes;
    $model_ok = 0 if $ac != $exp_calls || $ab != $exp_bytes;
    $model_ok = 0 if $ac != 2 && $ac != 4;   # KB-85-3 letter
  }
  # control: hello len=98 from phase A2 (below threshold)
  my $hline = `perl "$an" "$mout/census.84a.hello.census" 2>/dev/null`;
  chomp $hline;
  if ($hline =~ /a_calls=([\d.]+).* a_bytes=([\d.]+)/) {
    my ($hc, $hb) = ($1, $2);
    my $hlen = length($abshello);
    printf "VW2 control hello: len=%d a_calls=%.1f a_bytes=%.0f (floor model: 2/%d)\n", $hlen, $hc, $hb, 2*$hlen;
    $model_ok = 0 if $hc != 2 || $hb != 2*$hlen;
  } else { fail "VW2: hello control raw unparsable"; $valid = 0 }
  if ($valid) {
    if ($model_ok) {
      print "VW2 PASS: piecewise model CONFIRMED at the bisection — site NAMED: std run_path_with_cstr MAX_STACK_ALLOCATION=384 heap-CString(len+1) fallback, reached by fs::canonicalize + fs::metadata in main_unit_key; a_calls=2+2*[len>=384], a_bytes=2*len+2*(len+1)*[len>=384] (A-BB38 pin ARMED; KB-84-2 model retired->replaced)\n";
    } else {
      print "VW2 NAMED-DEVIATION: model REJECTED at the bisection (KB-85-3) — measures VALID, the piecewise form does not hold; back to named-deviation, no pin\n";
    }
  }
}

# ---- guard: KS-PP-85-1 (no unlabeled reqns samples in this campaign) --------------
{
  my $bad = 0;
  for my $f (glob "$mout/axum.84*.log") {
    open my $fh, '<', $f or next;
    while (<$fh>) { $bad++ if /^reqns: / && !/ w=1 / }
    close $fh;
  }
  if ($bad) { fail "guard: $bad reqns line(s) without w=1 in-band (KS-PP-85-1 — samples VOID)" }
  else { print "guard KS-PP-85-1: 0 unlabeled reqns lines in campaign raws (no slope phase ran)\n" }
}

if ($fails == 0) { print "\n== VERDICT84 PASS ==\n"; exit 0 }
else             { print "\n== VERDICT84 FAIL($fails) ==\n"; exit 1 }
PERL
