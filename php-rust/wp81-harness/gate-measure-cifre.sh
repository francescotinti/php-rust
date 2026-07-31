#!/bin/bash
# gate-measure-cifre.sh — KG-83-3 (Council WP-83): every numeric figure in a
# MEASURE document must match a line of COMMITTED machine output (.out,
# .summary, .matrix, .idle, raw .census/.log) or carry the A-BG26 tag
# `[derivata: ...]` on its line, or be a NAMED protocol constant in the
# allowlist below. Machine check, not promise.
#
# SCOPE (declared): tokens of >=3 digits (after normalizing the Italian
# formatting: thousands '.' stripped, decimal ',' -> '.'). 1-2 digit tokens
# are protocol/threshold small constants judged by eye — binding them would
# drown the gate in noise (every R=3, W=1, §N). Hex identities (git revs,
# sha fingerprints containing [a-f]) are excluded here: their truth is
# enforced by the feature-matrix/driver identity gates, not by this one.
#
# Self-test: --selftest plants a fabricated figure in a copy of the target
# and expects FAIL (a gate that cannot bite is vacuous).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
TARGET="${1:-$HERE/MEASURE81_RESULTS.md}"
MOUT="$HERE/../wp78-harness/measure-out"

if [ "${1:-}" = "--selftest" ]; then
  TMP=$(mktemp -d)
  cp "$HERE/MEASURE81_RESULTS.md" "$TMP/doctored.md"
  echo "smuggled figure: a_calls was 123457 on a good day" >> "$TMP/doctored.md"
  if bash "$0" "$TMP/doctored.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: smuggled 123457 was NOT caught (KG-83-3)"
    rm -rf "$TMP"; exit 1
  fi
  rm -rf "$TMP"
  echo "SELFTEST PASS: smuggled figure caught (KG-83-3 bites)"
  exit 0
fi

perl - "$TARGET" "$HERE" "$MOUT" <<'PERL'
use strict; use warnings;
use File::Glob qw(bsd_glob);   # plain glob() splits patterns on spaces — the
                               # repo lives under "/Volumes/Extreme Pro/"
my ($target, $here, $mout) = @ARGV;

# NAMED protocol constants (>=3 digits) that are neither measures nor
# derivates — each with its reason:
my %ALLOW = map { $_ => 1 } qw(
  110
  384
  6910767
  4000
  8048
  5000
);
# 110    = righe-per-run ENFORCE del protocollo (design78 driver)
# 384    = bound del buffer CString di stack in std (A-BB27, costante di libreria)
# 6910767= git rev baseline WP-80 (identita', enforced dal driver MEASURE80)
# 4000/8048/5000 = soglie ex-ante design79 par.10 (P1a/P1b/P5b) — anche nel
#                  corpus via verdict81.out, tenute qui per robustezza

# ---- corpus: committed machine outputs -------------------------------------
my @sources;
push @sources, bsd_glob("$here/*.out"), bsd_glob("$here/../wp82-harness/*.out");
push @sources, bsd_glob("$mout/*81*.summary"), bsd_glob("$mout/*81*.matrix"),
               bsd_glob("$mout/*81*.idle"),    bsd_glob("$mout/*81*.census"),
               bsd_glob("$mout/*81*.log");
# S-82.0: the measure82 campaign raws (82*/m82*/m82r* labels)
push @sources, bsd_glob("$mout/*82*.summary"), bsd_glob("$mout/*82*.census"),
               bsd_glob("$mout/*82*.log"),     bsd_glob("$mout/m82*"),
               bsd_glob("$mout/axum.82*");
push @sources, bsd_glob("$here/evidence/*");
die "gate-measure-cifre: EMPTY corpus (no committed sources found)\n" unless @sources;
my (%corpus, %corpus_count);
for my $f (@sources) {
  next if $f =~ m{/\._};             # AppleDouble
  open my $fh, '<', $f or next;
  while (my $l = <$fh>) {
    while ($l =~ /(\d[\d.]*\d|\d)/g) {
      my $t = $1;
      $corpus{$t} = 1;
      (my $noint = $t) =~ s/\.0$//;  # 43463.0 -> 43463
      $corpus{$noint} = 1;
      (my $nodot = $t) =~ s/\.//g;   # tolerate grouped copies
      $corpus{$nodot} = 1;
    }
  }
  close $fh;
}

# Line counts of committed evidence sets are machine-derivable figures
# (corpus 1418 = wc -l corpus81.fails, refl 290 = wc -l refl81.fails):
for my $f (bsd_glob("$here/evidence/*.fails")) {
  next if $f =~ m{/\._};
  open my $cfh, '<', $f or next;
  my $n = 0; $n++ while <$cfh>; close $cfh;
  $corpus_count{$n} = 1;
}

# ---- scan target -----------------------------------------------------------
open my $fh, '<', $target or die "cannot open $target\n";
my ($ln, @miss) = (0);
while (my $line = <$fh>) {
  $ln++;
  next if $line =~ /\[derivata/;                 # A-BG26 tagged line
  my $probe = $line;
  # remove hex identities and alphanumeric IDs so their digits don't tokenize
  $probe =~ s/\b(?=[0-9a-f]*[a-f])[0-9a-f]{7,}\b//gi;   # git/sha fragments (7+ hex with a letter)
  $probe =~ s/[A-Za-z_][A-Za-z0-9_-]*[0-9][A-Za-z0-9_-]*//g; # KS-AH-83-1, F13, wp81...
  while ($probe =~ /(?<![\dA-Za-z,.])(\d{1,3}(?:\.\d{3})+(?:,\d+)?|\d+,\d+|\d{3,})(?![\dA-Za-z])/g) {
    my $raw = $1;
    my $norm = $raw;
    $norm =~ s/\.(?=\d{3}\b)//g;   # thousands dots
    $norm =~ s/,/./;               # Italian decimal
    next if $ALLOW{$norm};
    next if $corpus{$norm};
    next if $corpus_count{$norm};
    (my $noint = $norm) =~ s/\.0$//;
    next if $corpus{$noint};
    push @miss, "line $ln: '$raw' (norm '$norm') not in committed corpus: $line";
  }
}
close $fh;

if (@miss) {
  print "FAIL gate-measure-cifre (KG-83-3): ".scalar(@miss)." unmatched figure(s):\n";
  print "  $_" for @miss;
  exit 1;
}
print "PASS gate-measure-cifre (KG-83-3): every bound figure in ".
      "$target matches committed machine output (or carries [derivata]/named-constant)\n";
exit 0;
PERL
