#!/bin/bash
# verdict86.sh — S-86.0: MACHINE verdict over the measure86 campaign raws.
# Fail-closed (A-SK19). EVERY block is COLLECT-THEN-EMIT (A-SK44/A-MS28,
# KS-SK-87-3): PASS/derivata lines flush only from a clean block.
# Teeth wired: A-SK45 (unexpected ord), A-PP36 (worker_dispatch in-band ==
# expected, KS-PP-87-3), KS-MS-86-1 (server_exit/panic), KS-AH-85-3
# (alloc_id), A-PP35 (reqns lines only via wp86-harness/reqns-guard.pl).
# Verdicts:
#   VCAL  — NET_H/NET_P on THIS binary (calibration base of every block).
#   VINV  — A-BB45 counter-proof: pad ord=1 == NET_P exact; hello ord=2 ==
#           NET_H − NET_P + 460.146 (the MODEL-GRADE pad-own under test).
#   VBURST— A-DL33: burst row == NET_H + 1.048.576 EXACT and thereby
#           DECLASSIFIED by the calibration tooth (pipeline fail-closed).
#   VOVL  — A-BB47: on the QUALIFIED attempt (spans intersect, distinct
#           tids) judge nets vs cals; every outcome NAMED; no qualified
#           attempt => OPEN by name.
#   VW123 — A-DL31 physical half (reduced: per-thread heap split refuted
#           by instrument): Δpeak per added worker vs 3.605.572 B ±5%.
#   VW500 — A-BB48: len=500 ⇒ a_calls=4, a_bytes=2.002; hello control.
#   VABBA — A-BB46: spread(A: purge=0) vs spread(B: default), interleaved;
#           attribution NAMED; NO pin founded here (KL-87-1/KB-87-2).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
MOUT="${MOUT86:-$REPO/wp78-harness/measure-out}"   # override only for bite-tests
AN="$REPO/wp80-harness/analyze80.pl"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
D500="vw86_len500_$(printf 'a%.0s' $(seq 1 150))"
N500="f$(printf 'c%.0s' $(seq 1 243)).php"
ABS500="$FIXDIR/$D500/$N500"
ABSHELLO="$FIXDIR/hello.php"

perl - "$MOUT" "$AN" "$ABS500" "$ABSHELLO" "$HERE/reqns-guard.pl" <<'PERL'
use strict; use warnings;
my ($mout, $an, $abs500, $abshello, $rguard) = @ARGV;
my $fails = 0;
sub fail { print "FAIL: @_\n"; $fails++ }

print "# verdict86 — S-86.0 attribution measures (Council WP-87 forms) — machine, fail-closed\n\n";

# parse one m86 memcensus -> {rows => {path=>{thr,ord,net}}, disp=>{thr=>n}, meta}
# Expected ords are BLOCK-SPECIFIC (the INV protocol expects ord=2): the
# caller passes a map path_suffix=>ord; any main row whose (path,ord) is
# not in the map fails the block (A-SK45 generalized).
sub parse86 {
    my ($bf, $label, $wexp, $ndispexp, $ordmap) = @_;
    my $f = "$mout/m86.$label.memcensus";
    open my $fh, '<', $f or do { $bf->("[$label] missing $f"); return undef };
    my (%rows, %disp, $meta);
    while (<$fh>) {
        s/\0//g;
        if (/tag=unitcache_main_entry thr=(\d+) ord=(\d+) net=(\d+).*alloc_id=(\S+) (.+)$/) {
            my ($thr, $ord, $net, $aid, $path) = ($1, $2, $3, $4, $5);
            $bf->("[$label] alloc_id $aid != memcount-v2-s82 (KS-AH-85-3)") unless $aid eq 'memcount-v2-s82';
            $bf->("[$label] row without arm label") unless /arm=axum-worker/;
            my ($fx) = $path =~ m{/([^/]+)$};
            if (!exists $ordmap->{$fx}) {
                $bf->("[$label] UNEXPECTED main row for $fx ord=$ord (A-SK45)");
            } elsif ($ordmap->{$fx} != $ord) {
                $bf->("[$label] $fx at ord=$ord, protocol expects ord=$ordmap->{$fx} (A-SK45)");
            }
            $bf->("[$label] duplicate row for $fx") if exists $rows{$fx};
            $rows{$fx} = { thr => $thr, ord => $ord, net => $net };
        }
        if (/tag=worker_dispatch thr=(\d+) count=(\d+)/) {
            $disp{$1} = $2;
        }
        $meta = $_ if /phase=$label /;
    }
    close $fh;
    unless ($meta) { $bf->("[$label] campaign meta line missing"); return undef }
    $bf->("[$label] meta not w=$wexp") unless $meta =~ /\bw=$wexp\b/;
    if ($meta =~ /server_exit=(\d+)/) {
        $bf->("[$label] server_exit=$1 != 0 (KS-MS-86-1)") unless $1 == 0;
    } else { $bf->("[$label] no server_exit in meta") }
    my $dsum = 0; $dsum += $_ for values %disp;
    $bf->("[$label] dispatch sum $dsum != expected $ndispexp (A-PP36/KS-PP-87-3)")
        unless $dsum == $ndispexp;
    return { rows => \%rows, disp => \%disp };
}

my ($NETH, $NETP);

# ---- VCAL --------------------------------------------------------------------
{
    my $bfail = 0; my @emit;
    my $bf = sub { print "FAIL: @_\n"; $fails++; $bfail++ };
    my $ch = parse86($bf, 'cal-h', 1, 1, { 'hello.php' => 1 });
    my $cp = parse86($bf, 'cal-p', 1, 1, { 'hello_pad85.php' => 1 });
    if (!$bfail && $ch && $cp) {
        $NETH = $ch->{rows}{'hello.php'}{net};
        $NETP = $cp->{rows}{'hello_pad85.php'}{net};
        if (defined $NETH && defined $NETP) {
            push @emit, sprintf "VCAL: NET_H = %d B = %.2f MiB · NET_P = %d B = %.2f MiB (W=1, this binary)",
                $NETH, $NETH/1048576, $NETP, $NETP/1048576;
            push @emit, sprintf "VCAL reproduction vs WP-85 record: hello %s · pad %s",
                ($NETH == 7349977 ? "IDENTICAL (7.349.977 B)" : "MOVED ($NETH B vs 7.349.977 B)"),
                ($NETP == 7803281 ? "IDENTICAL (7.803.281 B)" : "MOVED ($NETP B vs 7.803.281 B)");
            $bf->("VCAL: canary VACUOUS — NET_P == NET_H") if $NETP == $NETH;
        } else { $bf->("VCAL: calibration rows missing") }
    }
    if ($bfail == 0) { print "$_\n" for @emit }
    else { print "VCAL: $bfail fail(s) — buffered lines SUPPRESSED (A-SK44)\n" }
}

# ---- VINV (A-BB45) -----------------------------------------------------------
{
    my $bfail = 0; my @emit;
    my $bf = sub { print "FAIL: @_\n"; $fails++; $bfail++ };
    if (!defined $NETH || !defined $NETP) { $bf->("VINV: calibrations unavailable — block cannot judge") }
    my $inv = parse86($bf, 'inv', 1, 2, { 'hello_pad85.php' => 1, 'hello.php' => 2 });
    if (!$bfail && $inv) {
        my $p1 = $inv->{rows}{'hello_pad85.php'};
        my $h2 = $inv->{rows}{'hello.php'};
        if ($p1 && $h2) {
            $bf->("VINV: pad and hello on DIFFERENT threads ($p1->{thr} vs $h2->{thr}) — same-thread protocol broken")
                unless $p1->{thr} eq $h2->{thr};
            my $pred = $NETH - $NETP + 460146;
            push @emit, sprintf "VINV data: pad ord=1 net=%d B · hello ord=2 net=%d B · prediction (named ex-ante): %d B [derivata: NET_H−NET_P+460.146, pad-own MODEL-GRADE(src=m85.dl28) under test]",
                $p1->{net}, $h2->{net}, $pred;
            if ($p1->{net} != $NETP) {
                $bf->(sprintf "VINV: pad ord=1 net %d != NET_P %d — thread residue not reproduced under inverted order", $p1->{net}, $NETP);
            } elsif ($h2->{net} == $pred) {
                push @emit, "VINV PASS: INVERTED-ORDER counter-proof CONFIRMS the additive decomposition — hello-own(ord2) matches the named prediction byte-exact; the shared-node confound (996.838 B) did not bite on this pair (A-BB45; promotion of the decomposition is a Council WP-88 deliberation, KB-87-1 until then)";
            } else {
                push @emit, sprintf "VINV NAMED-DEVIATION: hello ord=2 net %d != predicted %d (Δ=%d B) — the decomposition stays MODEL-GRADE, never a budget addend (KB-87-1); deviation to Council WP-88", $h2->{net}, $pred, $h2->{net} - $pred;
            }
        } else { $bf->("VINV: expected rows missing") }
    }
    if ($bfail == 0) { print "$_\n" for @emit }
    else { print "VINV: $bfail fail(s) — buffered lines SUPPRESSED (A-SK44)\n" }
}

# ---- VBURST (A-DL33) ---------------------------------------------------------
{
    my $bfail = 0; my @emit;
    my $bf = sub { print "FAIL: @_\n"; $fails++; $bfail++ };
    if (!defined $NETH) { $bf->("VBURST: NET_H unavailable") }
    my $b = parse86($bf, 'burst', 1, 1, { 'hello.php' => 1 });
    if (!$bfail && $b) {
        my $r = $b->{rows}{'hello.php'};
        my $f = "$mout/m86.burst.memcensus";
        open my $fh, '<', $f; my $armed = 0;
        while (<$fh>) { $armed = 1 if /burst=1/ } close $fh;
        $bf->("VBURST: burst env not stamped in meta (burst=1)") unless $armed;
        if ($r) {
            my $want = $NETH + 1048576;
            push @emit, sprintf "VBURST data: net=%d B · NET_H+1 MiB = %d B", $r->{net}, $want;
            if ($r->{net} == $want) {
                push @emit, "VBURST PASS: the 1 MiB in-bracket burst moved the row EXACTLY +1.048.576 B AND the calibration tooth DECLASSIFIES it (net != NET_H) — the pipeline rejects a polluted row, fail-closed PROVEN (A-DL33); this run is never a measurement";
            } else {
                $bf->(sprintf "VBURST: net %d != NET_H+1MiB %d — burst not counted exactly (A-DL33)", $r->{net}, $want);
            }
        }
    }
    if ($bfail == 0) { print "$_\n" for @emit }
    else { print "VBURST: $bfail fail(s) — buffered lines SUPPRESSED (A-SK44)\n" }
}

# ---- VOVL (A-BB47) -----------------------------------------------------------
{
    my $bfail = 0; my @emit;
    my $bf = sub { print "FAIL: @_\n"; $fails++; $bfail++ };
    my $led = "$mout/m86.ovl.attempts.ledger";
    open my $lh, '<', $led or $bf->("VOVL: attempts ledger missing (KG-87-1)");
    my $qual;
    if ($lh) {
        while (<$lh>) { $qual = $1 if /attempt=(\d+) QUALIFIED/ }
        close $lh;
    }
    if (!defined $qual) {
        if ($bfail == 0) {
            print "VOVL OPEN: no qualifying overlap in the ledgered attempts — A-BB47 stays OPEN by name; per-thread under concurrency remains CANDIDATE (KB-87-3/KL-87-2)\n";
        } else { print "VOVL: $bfail fail(s) — buffered lines SUPPRESSED (A-SK44)\n" }
    } else {
        my $o = parse86($bf, "ovl.a$qual", 2, 2, { 'hello.php' => 1, 'hello_pad85.php' => 1 });
        # recompute the overlap from the raw — never trust the campaign's own qualify
        my $f = "$mout/m86.ovl.a$qual.memcensus";
        my ($ht0,$ht1,$htid,$pt0,$pt1,$ptid);
        if (open my $fh, '<', $f) {
            while (<$fh>) {
                s/\0//g;
                next unless /tag=lower_span tid=(\S+) t0_us=(\d+) t1_us=(\d+) (.+)$/;
                my ($tid,$t0,$t1,$p) = ($1,$2,$3,$4);
                if ($p =~ /hello_pad85[.]php$/) { ($pt0,$pt1,$ptid) = ($t0,$t1,$tid) }
                elsif ($p =~ /hello[.]php$/)    { ($ht0,$ht1,$htid) = ($t0,$t1,$tid) }
            }
            close $fh;
        }
        if (!defined $htid || !defined $ptid) { $bf->("VOVL: lower_span rows missing in qualified raw") }
        elsif ($htid eq $ptid) { $bf->("VOVL: spans on the SAME tid — not concurrent") }
        elsif (!($ht0 < $pt1 && $pt0 < $ht1)) { $bf->("VOVL: spans do NOT intersect — attempt was not an overlap (recomputed)") }
        if (!$bfail && $o) {
            my $h = $o->{rows}{'hello.php'}; my $p = $o->{rows}{'hello_pad85.php'};
            push @emit, sprintf "VOVL data (attempt %d, spans intersect %d..%d ∩ %d..%d µs, distinct tids): hello thr=%s net=%d B · pad thr=%s net=%d B",
                $qual, $ht0, $ht1, $pt0, $pt1, $h->{thr}, $h->{net}, $p->{thr}, $p->{net};
            $bf->("VOVL: both fixtures on one thread") if $h->{thr} eq $p->{thr};
            if (!$bfail) {
                if ($h->{net} == $NETH && $p->{net} == $NETP) {
                    push @emit, "VOVL PASS: PER-THREAD holds UNDER REAL OVERLAP — each thread nets its own calibrated figure byte-exact with intersecting lowering spans (A-BB47 satisfied; KB-87-3 lifted for THIS workload class — destructor-carrying fixtures still need their own canary)";
                } else {
                    push @emit, sprintf "VOVL NAMED-OUTCOME: nets under overlap DIVERGE from calibrations (hello %d vs %d, pad %d vs %d) — the process-counter window does NOT attribute under concurrency: per-thread stays SEQUENTIAL-ONLY (KL-87-2 stands, ×W budget remains sequential-protocol-bound)", $h->{net}, $NETH, $p->{net}, $NETP;
                }
            }
        }
        if ($bfail == 0) { print "$_\n" for @emit }
        else { print "VOVL: $bfail fail(s) — buffered lines SUPPRESSED (A-SK44)\n" }
    }
}

# ---- VW123 (A-DL31 physical half, reduced) -----------------------------------
{
    my $bfail = 0; my @emit;
    my $bf = sub { print "FAIL: @_\n"; $fails++; $bfail++ };
    my %pk;
    for my $w (1, 2, 3) {
        my $l = "$mout/m86.w123.w$w.log";
        open my $fh, '<', $l or do { $bf->("VW123: missing $l"); next };
        while (<$fh>) { $pk{$w} = $1 if /(\d+)\s+maximum resident set size/ }
        close $fh;
        $bf->("VW123: no peak in $l") unless defined $pk{$w};
    }
    if (!$bfail) {
        my $d2 = $pk{2} - $pk{1};
        my $d3 = $pk{3} - $pk{2};
        my ($lo, $hi) = (3605572 * 0.95, 3605572 * 1.05);
        push @emit, sprintf "VW123 data: peaks %d / %d / %d B · Δw2=%d B = %.2f MiB · Δw3=%d B = %.2f MiB [derivata: differenze fra i tre peak]",
            $pk{1}, $pk{2}, $pk{3}, $d2, $d2/1048576, $d3, $d3/1048576;
        push @emit, "VW123 NOTE: per-thread heap split REFUTED BY INSTRUMENT (mimalloc v3 shared default heap, heap= tooth) — reduced Δpeak form; full counted-vs-physical closure to Council WP-88";
        if ($d2 >= $lo && $d2 <= $hi && $d3 >= $lo && $d3 <= $hi) {
            push @emit, "VW123 PASS: both per-added-worker physical deltas INSIDE 3.605.572 B ±5% — the KL-85-2 physical half holds at W=1/2/3 (A-DL31 reduced form)";
        } else {
            push @emit, sprintf "VW123 NAMED-DEVIATION: Δ per added worker OUTSIDE 3.605.572 B ±5%% [%.0f; %.0f] — physical prediction does not hold in this form; to Council WP-88 (A-DL31)", $lo, $hi;
        }
    }
    if ($bfail == 0) { print "$_\n" for @emit }
    else { print "VW123: $bfail fail(s) — buffered lines SUPPRESSED (A-SK44)\n" }
}

# ---- VW500 (A-BB48) ----------------------------------------------------------
{
    my $bfail = 0; my @emit;
    my $bf = sub { print "FAIL: @_\n"; $fails++; $bfail++ };
    $bf->("VW500: fixture length drifted (".length($abs500).")") unless length($abs500) == 500;
    my $line = `perl "$an" "$mout/census.86w.len500.census" 2>/dev/null`;
    chomp $line;
    if ($line =~ /steady_n=(\d+).* a_calls=([\d.]+).* a_bytes=([\d.]+)/) {
        my ($sn, $ac, $ab) = ($1, $2, $3);
        $bf->("VW500: steady_n=$sn != 30 (A-SK34)") unless $sn == 30;
        push @emit, sprintf "VW500 data: len=500 a_calls=%.1f a_bytes=%.0f (named ex-ante: 4 / 2.002 B [derivata: 2·500+2·501])", $ac, $ab;
        if (!$bfail) {
            if ($ac == 4 && $ab == 2002) {
                push @emit, "VW500 PASS: third point len=500 ON the piecewise model — a_calls=4, a_bytes=2.002 B exact (A-BB44 executed, A-BB48 retro-declaration closed)";
            } else {
                push @emit, "VW500 NAMED-DEVIATION: len=500 off-model — piecewise form does not extend beyond the bisection (KB-85-3 class); to Council WP-88";
            }
        }
    } else { $bf->("VW500: census raw missing/unparsable") }
    my $hline = `perl "$an" "$mout/census.86w.hello.census" 2>/dev/null`;
    chomp $hline;
    if ($hline =~ /a_calls=([\d.]+).* a_bytes=([\d.]+)/) {
        my ($hc, $hb) = ($1, $2);
        my $hlen = length($abshello);
        push @emit, sprintf "VW500 control hello: len=%d a_calls=%.1f a_bytes=%.0f (floor model: 2 / %d)", $hlen, $hc, $hb, 2*$hlen;
        $bf->("VW500: hello control off floor model") unless $hc == 2 && $hb == 2*$hlen;
    } else { $bf->("VW500: hello control raw unparsable") }
    if ($bfail == 0) { print "$_\n" for @emit }
    else { print "VW500: $bfail fail(s) — buffered lines SUPPRESSED (A-SK44)\n" }
}

# ---- VABBA (A-BB46) ----------------------------------------------------------
{
    my $bfail = 0; my @emit;
    my $bf = sub { print "FAIL: @_\n"; $fails++; $bfail++ };
    my (%pkA, %pkB, $driver_sha);
    for my $r (1..9) {
        for my $side (['pa', \%pkA, '0'], ['pd', \%pkB, 'default']) {
            my ($tag, $h, $env) = @$side;
            my $l = "$mout/axum.86p.$tag.r$r.log";
            open my $fh, '<', $l or do { $bf->("VABBA: missing $l"); next };
            my ($pk, $penv, $popt);
            while (<$fh>) {
                $pk = $1 if /(\d+)\s+maximum resident set size/;
                $penv = $1 if /^purge_env=(\S+)/;
                $popt = $1 if /option 'purge_delay': (\d+)/;
                $driver_sha = $1 if /driver_sha=([0-9a-f]+)/ && !$driver_sha;
            }
            close $fh;
            defined $pk ? ($h->{$r} = $pk) : $bf->("VABBA: no peak in $l");
            $bf->("VABBA: $tag.r$r purge_env=$penv != $env (label)") unless defined $penv && $penv eq $env;
            if ($env eq '0') {
                $bf->("VABBA: $tag.r$r effective purge_delay != 0 (verbose option line)") unless defined $popt && $popt == 0;
            } else {
                $bf->("VABBA: $tag.r$r verbose option line missing") unless defined $popt;
                $bf->("VABBA: $tag.r$r 'default' arm shows purge_delay=0 — arms swapped") if defined $popt && $popt == 0;
            }
        }
    }
    if (!$bfail) {
        my @a = map { $pkA{$_} } (2..9);
        my @b = map { $pkB{$_} } (2..9);
        my @as = sort { $a <=> $b } @a;
        my @bs = sort { $a <=> $b } @b;
        my $sA = $as[-1] - $as[0];
        my $sB = $bs[-1] - $bs[0];
        push @emit, sprintf "VABBA data: arm A (purge=0) peaks r1..r9 = %s B · arm B (default) = %s B (W in-band, R=9 per arm, arm=union, fixture=hello.php, driver_sha=%s — A-BG38 scope; interleaved ABBA)",
            join('/', map { $pkA{$_} } (1..9)), join('/', map { $pkB{$_} } (1..9)), ($driver_sha // 'UNKNOWN');
        push @emit, sprintf "VABBA spreads (r2..r9): spread(A)=%d B = %.2f MiB · spread(B)=%d B = %.2f MiB [derivata: max−min r2..r9 per braccio]",
            $sA, $sA/1048576, $sB, $sB/1048576;
        push @emit, "VABBA NOTE (A-BG41): driver_sha of THIS campaign differs from measure85 (measure78.sh gained the purge parameter) — cross-campaign comparisons must name both sides and both R";
        if ($sA < 0.5 * $sB) {
            push @emit, "VABBA ATTRIBUTED: spread(A) < 0,5×spread(B) — the VP spread is ATTRIBUTED to purge timing (A-BB46 threshold met); pin refoundation stays a Council WP-88 deliberation (KL-87-1/KB-87-2: envelope-only until then)";
        } elsif ($sB < 0.5 * $sA) {
            push @emit, "VABBA INVERTED: spread(B) < 0,5×spread(A) — purge=0 WIDENS the spread; purge timing refuted as the driver in this direction; to Council WP-88";
        } else {
            push @emit, "VABBA INCONCLUSIVE: neither arm halves the other's spread — purge timing NOT attributed; spread stays unattributed, pin stays RETIRED (envelope-only, KL-87-1)";
        }
    }
    if ($bfail == 0) { print "$_\n" for @emit }
    else { print "VABBA: $bfail fail(s) — buffered lines SUPPRESSED (A-SK44)\n" }
}

# ---- guard: A-PP31/A-PP35 (reqns lines only via the shared guard) ------------
{
    my $bad = 0;
    for my $f (glob "$mout/axum.86*.log") {
        my $out = `perl "$rguard" "$f" 2>&1 >/dev/null`;
        $bad++ if ($? >> 8) != 0;
    }
    if ($bad) { fail "guard: $bad raw(s) with reqns lines rejected by reqns-guard.pl (A-PP35)" }
    else { print "guard A-PP35: every campaign raw passes the shared reqns-guard (0 violations)\n" }
}

if ($fails == 0) { print "\n== VERDICT86 PASS ==\n"; exit 0 }
else             { print "\n== VERDICT86 FAIL($fails) ==\n"; exit 1 }
PERL
