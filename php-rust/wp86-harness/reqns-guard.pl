#!/usr/bin/perl
# reqns-guard.pl — A-PP35 (Council WP-87, Pedersen): the SHARED reqns
# parser. The old inheritance sweep accepted any script containing the
# token `w=1` — satisfied by `w=10` or a comment (token-presence is not a
# rejection, KS-PP-87-1). This guard is the single sourcing point for
# future verdicts: it emits ONLY `reqns: ` lines carrying a space/EOL
# delimited `w=1` stamp and exits non-zero if ANY reqns line lacks it
# (fail-closed: one unlabeled sample voids the run's slope samples).
# Usage:
#   reqns-guard.pl <raw-file ...>   # prints legal lines; exit 1 on violation
#   reqns-guard.pl --selftest       # synthetic w=2 raw MUST void (bite-test)
use strict;
use warnings;

if (($ARGV[0] // '') eq '--selftest') {
    my $tmp = ($ENV{TMPDIR} // '/tmp') . "/reqns-guard-selftest.$$";
    # (a) a w=2 raw MUST be refused — the tooth bites
    open my $fh, '>', $tmp or die "selftest tmp: $!";
    print $fh "reqns: 12345 w=2 arm=union\n";
    close $fh;
    my $rc_bad = system($^X, $0, $tmp) >> 8;
    # (b) a w=10 raw MUST be refused — the digit-guard discriminates
    open $fh, '>', $tmp or die "selftest tmp: $!";
    print $fh "reqns: 12345 w=10 arm=union\n";
    close $fh;
    my $rc_ten = system($^X, $0, $tmp) >> 8;
    # (c) a legal w=1 raw MUST pass
    open $fh, '>', $tmp or die "selftest tmp: $!";
    print $fh "reqns: 12345 w=1 arm=union\n";
    close $fh;
    my $rc_ok = system($^X, $0, $tmp) >> 8;
    unlink $tmp;
    if ($rc_bad == 0) { print "SELFTEST FAIL: w=2 raw passed the guard (A-PP35)\n"; exit 1 }
    if ($rc_ten == 0) { print "SELFTEST FAIL: w=10 raw passed the guard (A-PP35/KS-PP-87-1)\n"; exit 1 }
    if ($rc_ok != 0)  { print "SELFTEST FAIL: legal w=1 raw refused (A-PP35)\n"; exit 1 }
    print "SELFTEST PASS: reqns-guard voids w!=1 (incl. w=10) and passes w=1 (A-PP35)\n";
    exit 0;
}

my $fails = 0;
while (<>) {
    next unless /^reqns: /;
    if (/ w=1(?:[^0-9]|$)/) {
        print;
    } else {
        warn "VOID: reqns line without space-delimited w=1 in-band (A-PP35): $_";
        $fails++;
    }
}
exit($fails ? 1 : 0);
