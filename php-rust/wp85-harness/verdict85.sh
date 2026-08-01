#!/bin/bash
# verdict85.sh — S-85.0: MACHINE verdict over the measure85 campaign raws.
# Fail-closed (A-SK19). A-SK38 (Council WP-86): per-BLOCK fail flags — a
# PASS/derivata line is emitted ONLY from a block with zero fails; controls
# get the same validity checks as measures (KS-SK-86-4).
# A-DL26/A-SK40: memory figures BYTES-FIRST with verified companions.
# Verdicts:
#   VDL28 (A-DL28≡A-BB41, the discriminator KL-86-2/KB-86-2 demand):
#         calibration NET_H (hello, W=1) and NET_P (pad, W=1) from separate
#         runs; the W=2 canary run must show, on DISTINCT threads, the
#         hello row netting EXACTLY NET_H and the pad row EXACTLY NET_P
#         (attribution BY PATH in-band, KS-PP-86-3). Equal nets => frozen
#         window => VDL24 VOID (classification RITIRATA). NET_P == NET_H
#         in calibration => canary VACUOUS => FAIL.
#         Rows from a run with server_exit!=0 are VOID (KS-MS-86-1).
#         A-BB42: the legacy ambiguity bands are re-anchored to the
#         WP-83 record residue (7.349.977 B): [0.1x; 0.5x] =
#         [734.998; 3.674.989] B — reported as derivata; the exact
#         calibration match SUPERSEDES band classification.
#   VP    (A-BB40/KB-86-1): NINE peak logs W=ncpu; min-of-R; the FIRST run
#         NAMED with its wall time (cold-start attribution, KL-86-1);
#         envelope max reported; outcome vs the S-82.0 pin 232±1 MiB
#         NAMED either way. Scope named per A-BG38: arm, W, R, fixture,
#         driver_sha.
#   Guard KS-PP-85-1/A-PP31: any reqns line without `w=1` in-band voids
#         slope samples — no slope phase ran; count must be 0.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
MOUT="$REPO/wp78-harness/measure-out"

perl - "$MOUT" <<'PERL'
use strict; use warnings;
my ($mout) = @ARGV;
my $fails = 0;
sub fail { print "FAIL: @_\n"; $fails++ }

print "# verdict85 — S-85.0 discrimination measures (Council WP-86 forms) — machine, fail-closed\n\n";

# ---- VDL28: the discriminator --------------------------------------------------
{
  my $bfail = 0;
  my $bf = sub { print "FAIL: @_\n"; $fails++; $bfail++ };
  # parse one memcensus file -> { path_suffix => {thr, net}, meta }
  my $parse = sub {
    my ($label, $wexp) = @_;
    my $f = "$mout/m85.$label.memcensus";
    open my $fh, '<', $f or do { $bf->("VDL28: missing $f"); return undef };
    my (%rows, $meta);
    while (<$fh>) {
      # NB (S-85.0 parser fix, declared): the fixture path is the LAST field
      # and the repo path contains a SPACE ("Extreme Pro") — (.+)$ not (\S+)$.
      if (/tag=unitcache_main_entry thr=(\d+) ord=1 net=(\d+).*alloc_id=(\S+) (.+)$/) {
        my ($thr, $net, $aid, $path) = ($1, $2, $3, $4);
        $bf->("VDL28[$label]: alloc_id $aid != memcount-v2-s82 (KS-AH-85-3)") unless $aid eq 'memcount-v2-s82';
        $bf->("VDL28[$label]: row without arm label") unless /arm=axum-worker/;
        my ($fx) = $path =~ m{/([^/]+)$};
        $bf->("VDL28[$label]: duplicate ord=1 row for fixture $fx") if exists $rows{$fx};
        $rows{$fx} = { thr => $thr, net => $net };
      }
      $meta = $_ if /phase=$label/;
    }
    close $fh;
    unless ($meta) { $bf->("VDL28[$label]: campaign meta line missing"); return undef }
    $bf->("VDL28[$label]: meta not w=$wexp") unless $meta =~ /\bw=$wexp\b/;
    if ($meta =~ /server_exit=(\d+)/) {
      $bf->("VDL28[$label]: server_exit=$1 != 0 — rows VOID (KS-MS-86-1)") unless $1 == 0;
    } else { $bf->("VDL28[$label]: no server_exit in meta (KS-MS-86-1)") }
    return \%rows;
  };
  my $calh = $parse->('cal-h', 1);
  my $calp = $parse->('cal-p', 1);
  # S-85.0 DECLARED SUPERSEDE: the campaign's m85.dl28 raw is VOID by
  # protocol break (hello dispatched twice: up-probe + oracle check, so
  # both workers' request 1 was hello and the pad landed as thr0's SECOND
  # main, ord=2 net=460.146 B — which incidentally CLOSED the additive
  # algebra: residue 7.343.135 + hello-own 6.842 = NET_H, residue +
  # pad-own 460.146 = NET_P, byte-exact). The canary verdict reads the
  # ONE-DISPATCH-PER-WORKER rerun (dl28-supplement.sh, label m85s.dl28);
  # the superseded raw stays in place, never rm'd (KS-AH-83-2).
  my $dl   = $parse->('dl28s', 2);
  if (!$bfail && $calh && $calp && $dl) {
    my $neth = $calh->{'hello.php'}{net};
    my $netp = $calp->{'hello_pad85.php'}{net};
    $bf->("VDL28: cal-h lacks the hello.php ord=1 row") unless defined $neth;
    $bf->("VDL28: cal-p lacks the hello_pad85.php ord=1 row") unless defined $netp;
    $bf->("VDL28: cal-h carries foreign rows") if scalar(keys %$calh) != 1;
    $bf->("VDL28: cal-p carries foreign rows") if scalar(keys %$calp) != 1;
    if (defined $neth && defined $netp) {
      printf "VDL28 cal: NET_H = %d B = %.2f MiB (hello, W=1) · NET_P = %d B = %.2f MiB (pad, W=1)\n",
        $neth, $neth/1048576, $netp, $netp/1048576;
      $bf->("VDL28: canary VACUOUS — NET_P == NET_H, the fixtures do not discriminate") if $netp == $neth;
      # A-BB42 bands, re-anchored to the record residue (derivata):
      printf "VDL28 bands [derivata: 0.1x/0.5x of record residue 7.349.977 B, A-BB42]: 734.998 B / 3.674.989 B — SUPERSEDED here by exact calibration match\n";
      my $dh = $dl->{'hello.php'};
      my $dp = $dl->{'hello_pad85.php'};
      $bf->("VDL28: W=2 run lacks the hello row") unless $dh;
      $bf->("VDL28: W=2 run lacks the pad row")   unless $dp;
      if ($dh && $dp && !$bfail) {
        printf "VDL28 W=2: hello thr=%s net=%d B = %.2f MiB · pad thr=%s net=%d B = %.2f MiB\n",
          $dh->{thr}, $dh->{net}, $dh->{net}/1048576, $dp->{thr}, $dp->{net}, $dp->{net}/1048576;
        $bf->("VDL28: both fixtures landed on thr=$dh->{thr} — protocol break, rerun (fail-closed, never classified)")
          if $dh->{thr} eq $dp->{thr};
        if ($dh->{net} == $dp->{net}) {
          $bf->("VDL28: FROZEN WINDOW — the two threads net IDENTICALLY under DIFFERENT fixtures: the window does not attribute; VDL24 PER-THREAD RITIRATA (KL-86-2/KB-86-2)");
        } elsif ($dh->{net} == $neth && $dp->{net} == $netp) {
          print "VDL28 PASS: PER-THREAD CONFIRMED by the monolateral canary — each thread nets EXACTLY its own fixture's calibrated figure (byte-exact); the x W budget formula is PROMOTED from IPOTESI CANDIDATA (KL-86-2 satisfied, KB-86-2 unblocked on this precondition; registry still needs A-MS27, KS-MS-86-2)\n";
        } else {
          $bf->(sprintf "VDL28: calibration MISMATCH — hello %d vs NET_H %d, pad %d vs NET_P %d: the window is not attributable per-thread as-is; VDL24 stays CANDIDATE",
            $dh->{net}, $neth, $dp->{net}, $netp);
        }
      }
    }
  }
}

# ---- VP: A-BB40 R=9 -------------------------------------------------------------
chomp(my $nc = `sysctl -n hw.ncpu`);
{
  my $bfail = 0;
  my $bf = sub { print "FAIL: @_\n"; $fails++; $bfail++ };
  my (@pk, @wall, $driver_sha);
  for my $r (1..9) {
    my $l = "$mout/axum.85p.w$nc.r$r.log";
    open my $fh, '<', $l or do { $bf->("VP: missing peak log $l"); next };
    my ($pk, $wall);
    while (<$fh>) {
      $pk = $1 if /(\d+)\s+maximum resident set size/;
      $wall = $1 if /^\s*(\d+\.\d+)\s+real/;
      $driver_sha = $1 if /driver_sha=([0-9a-f]+)/ && !$driver_sha;
    }
    defined $pk ? push @pk, $pk : $bf->("VP: no peak in $l");
    push @wall, defined $wall ? $wall : -1;
  }
  if (!$bfail && @pk == 9) {
    my @mb = map { $_ / 1048576 } @pk;
    my @sorted = sort { $a <=> $b } @pk;
    my ($min, $max) = ($sorted[0], $sorted[-1]);
    printf "VP data: peaks = %s B (W=%d, R=9, arm=union, fixture=hello.php, driver_sha=%s — A-BG38 scope)\n",
      join('/', @pk), $nc, ($driver_sha // 'UNKNOWN');
    printf "VP r1 NAMED (KB-86-1/KL-86-1 cold-start attribution): peak %d B = %.1f MiB, wall %ss; r2..r9 walls: %s\n",
      $pk[0], $mb[0], $wall[0], join('/', @wall[1..8]);
    my @rest = @pk[1..8];
    my @rsort = sort { $a <=> $b } @rest;
    my $rspread = $rsort[-1] - $rsort[0];
    printf "VP min-of-R = %d B = %.1f MiB · envelope max = %d B = %.1f MiB · r2..r9 spread = %d B = %.2f MiB [derivata: max-min of r2..r9]\n",
      $min, $min/1048576, $max, $max/1048576, $rspread, $rspread/1048576;
    my $inside = 1;
    for (@mb) { $inside = 0 if $_ < 231 || $_ > 233 }
    if ($inside) {
      print "VP NAMED-OUTCOME: all nine peaks INSIDE the S-82.0 pin 232±1 MiB — the pin holds at R=9\n";
    } else {
      print "VP NAMED-OUTCOME: pin 232±1 MiB MOVED at R=9 — deliberation uses THESE nine values (KB-86-1 satisfied: R=9, first run named; driver_sha of both sides required for any cross-campaign comparison, KG-86-2)\n";
    }
  }
}

# ---- guard: KS-PP-85-1/A-PP31 (no unlabeled reqns samples) ----------------------
{
  my $bad = 0;
  for my $f (glob "$mout/axum.85*.log") {
    open my $fh, '<', $f or next;
    while (<$fh>) { $bad++ if /^reqns: / && !/ w=1 / }
    close $fh;
  }
  if ($bad) { fail "guard: $bad reqns line(s) without w=1 in-band (KS-PP-85-1 — samples VOID)" }
  else { print "guard KS-PP-85-1: 0 unlabeled reqns lines in campaign raws (no slope phase ran)\n" }
}

if ($fails == 0) { print "\n== VERDICT85 PASS ==\n"; exit 0 }
else             { print "\n== VERDICT85 FAIL($fails) ==\n"; exit 1 }
PERL
