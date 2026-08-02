#!/usr/bin/perl
# putord-pair-guard.pl — A-DS38 (Council WP-88, Stogov): the SHARED
# eviction-pair checker. An armed uc_log claims per-put emission order
# (A-DS31/A-DS36) — but the in-cargo pin covered only the FIRST pair: a
# batched flush preserving pair 1 and reordering from pair 2 passed
# everything. This guard enforces the invariant on EVERY pair of EVERY
# armed log: each `unitcache main_evicted ... putord=N` row must be
# IMMEDIATELY followed by a `unitcache evict fp ... putord=N` row.
# Verdicts consume it on every armed log (KS-DS-88-1: a log with
# main_evicted rows that has not passed this guard is order-ADVISORY,
# never verdict-grade).
# Usage:
#   putord-pair-guard.pl <uc-log ...>   # exit 0 = all pairs hold (count on stdout)
#   putord-pair-guard.pl --selftest     # reordered pair-2 log MUST void
use strict;
use warnings;

sub check_file {
    my ($path) = @_;
    open my $fh, '<', $path or do { warn "putord-pair-guard: open $path: $!\n"; return (0, 1) };
    my ($pairs, $fails) = (0, 0);
    my $pending;    # putord of a main_evicted row awaiting its evict-fp
    my $pending_ln = 0;
    while (my $l = <$fh>) {
        $l =~ s/\0//g and do { warn "VOID: NUL byte in armed log $path (KS-SK-88-3)\n"; $fails++ };
        next unless $l =~ /^unitcache /;
        if (defined $pending) {
            if ($l =~ /^unitcache evict fp .*putord=(\d+)/ && $1 == $pending) {
                $pairs++;
            } else {
                warn "VOID: main_evicted (line $pending_ln, putord=$pending) not followed by its evict-fp (A-DS38/KS-DS-88-1): $l";
                $fails++;
            }
            undef $pending;
            next;
        }
        if ($l =~ /^unitcache main_evicted .*putord=(\d+)/) {
            $pending = $1;
            $pending_ln = $.;
        } elsif ($l =~ /^unitcache main_evicted /) {
            warn "VOID: main_evicted row without putord= in-band (A-DS36/A-DS38): $l";
            $fails++;
        }
    }
    if (defined $pending) {
        warn "VOID: trailing main_evicted (putord=$pending) with no evict-fp row (A-DS38)\n";
        $fails++;
    }
    close $fh;
    return ($pairs, $fails);
}

if (($ARGV[0] // '') eq '--selftest') {
    my $tmp = ($ENV{TMPDIR} // '/tmp') . "/putord-pair-guard-selftest.$$";
    # (a) both pairs intact MUST pass
    open my $fh, '>', $tmp or die "selftest tmp: $!";
    print $fh "unitcache main_evicted putord=3 /a.php\n";
    print $fh "unitcache evict fp 00ff putord=3 /a.php\n";
    print $fh "unitcache main_evicted putord=7 /b.php\n";
    print $fh "unitcache evict fp 00aa putord=7 /b.php\n";
    close $fh;
    my $rc_ok = system($^X, $0, $tmp) >> 8;
    # (b) pair 1 intact, pair 2 REORDERED (the exact evasion) MUST void
    open $fh, '>', $tmp or die "selftest tmp: $!";
    print $fh "unitcache main_evicted putord=3 /a.php\n";
    print $fh "unitcache evict fp 00ff putord=3 /a.php\n";
    print $fh "unitcache main_evicted putord=7 /b.php\n";
    print $fh "unitcache hit intra /c.php\n";
    print $fh "unitcache evict fp 00aa putord=7 /b.php\n";
    close $fh;
    my $rc_reord = system($^X, $0, $tmp) >> 8;
    # (c) putord mismatch across the pair MUST void
    open $fh, '>', $tmp or die "selftest tmp: $!";
    print $fh "unitcache main_evicted putord=3 /a.php\n";
    print $fh "unitcache evict fp 00ff putord=4 /a.php\n";
    close $fh;
    my $rc_mism = system($^X, $0, $tmp) >> 8;
    unlink $tmp;
    if ($rc_ok != 0)    { print "SELFTEST FAIL: intact pairs refused (A-DS38)\n"; exit 1 }
    if ($rc_reord == 0) { print "SELFTEST FAIL: reordered pair-2 passed — first-position blindness regressed (A-DS38)\n"; exit 1 }
    if ($rc_mism == 0)  { print "SELFTEST FAIL: putord-mismatched pair passed (A-DS38)\n"; exit 1 }
    print "SELFTEST PASS: putord-pair-guard enforces ALL pairs (reorder + mismatch void; intact passes) (A-DS38)\n";
    exit 0;
}

@ARGV or die "usage: $0 <uc-log ...> | --selftest\n";
my ($tp, $tf) = (0, 0);
for my $f (@ARGV) {
    my ($p, $x) = check_file($f);
    $tp += $p;
    $tf += $x;
}
print "putord-pair-guard: $tp pair(s) verified, $tf violation(s)\n";
exit($tf ? 1 : 0);
