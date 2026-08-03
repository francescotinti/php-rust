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

# A-SK40 (Council WP-86, sana il buco GRAVE KS-SK-86-3): --all loops over the
# default target PLUS every MEASURE8[4-9]/MEASURE9x results doc in the wpNN
# harnesses — the battery must call THIS mode, never the bare default (which
# targets MEASURE81 only and left MEASURE84 outside the 15/15 perimeter).
if [ "${1:-}" = "--all" ]; then
  rc=0
  # A-SK55 perf note: the committed corpus is identical for every doc at
  # the same HEAD — build it once and share it (cache file keyed to the
  # HEAD rev inside; a different rev invalidates it, fail-closed).
  GATE_CIFRE_CORPUS_CACHE=$(mktemp /tmp/gate-cifre-corpus.XXXXXX)
  export GATE_CIFRE_CORPUS_CACHE
  bash "$0" "$HERE/MEASURE81_RESULTS.md" || rc=1
  for f in "$HERE"/../wp8[4-9]-harness/MEASURE8[4-9]*_RESULTS.md \
           "$HERE"/../wp9[0-9]-harness/MEASURE9[0-9]*_RESULTS.md; do
    [ -f "$f" ] || continue
    bash "$0" "$f" || rc=1
  done
  rm -f "$GATE_CIFRE_CORPUS_CACHE"
  [ "$rc" = 0 ] && echo "PASS gate-measure-cifre --all (A-SK40): every MEASURE doc in perimeter"
  exit $rc
fi

if [ "${1:-}" = "--selftest" ]; then
  TMP=$(mktemp -d)
  cp "$HERE/MEASURE81_RESULTS.md" "$TMP/doctored.md"
  echo "smuggled figure: a_calls was 123457 on a good day" >> "$TMP/doctored.md"
  if bash "$0" "$TMP/doctored.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: smuggled 123457 was NOT caught (KG-83-3)"
    rm -rf "$TMP"; exit 1
  fi
  # A-SK40 bytes-first teeth on a MEASURE84-class copy: (a) a unit figure
  # with NO companion, lowercase unit (the old check was case-sensitive and
  # line-wide); (b) a companion that does NOT verify numerically.
  M84="$HERE/../wp84-harness/MEASURE84_RESULTS.md"
  if [ -f "$M84" ]; then
    cp "$M84" "$TMP/MEASURE84_doctored.md"
    echo "note: the cache costs 5,00 mib steady, honest." >> "$TMP/MEASURE84_doctored.md"
    if bash "$0" "$TMP/MEASURE84_doctored.md" >/dev/null 2>&1; then
      echo "SELFTEST FAIL: naked lowercase 'mib' figure NOT caught (A-SK40)"
      rm -rf "$TMP"; exit 1
    fi
    cp "$M84" "$TMP/MEASURE84_doctored2.md"
    echo "note: 1.048.576 B = 2,00 MiB [derivata: selftest]" >> "$TMP/MEASURE84_doctored2.md"
    if bash "$0" "$TMP/MEASURE84_doctored2.md" >/dev/null 2>&1; then
      echo "SELFTEST FAIL: mismatching bytes companion NOT caught (A-SK40)"
      rm -rf "$TMP"; exit 1
    fi
  fi
  # A-SK55 bite (Council WP-90, Klabnik's live forge re-armed forever): an
  # UNCOMMITTED file in measure-out matching a corpus glob must NOT
  # legalize a figure — the corpus is HEAD-only.
  FORGE="$MOUT/m88.zzforge-selftest.tmp"
  echo "committed=987654321" > "$FORGE"
  cp "$HERE/MEASURE81_RESULTS.md" "$TMP/doctored55.md"
  echo "smuggled: campaign committed was 987654321 flat" >> "$TMP/doctored55.md"
  if bash "$0" "$TMP/doctored55.md" >/dev/null 2>&1; then
    rm -f "$FORGE"; rm -rf "$TMP"
    echo "SELFTEST FAIL: working-tree forge in measure-out legalized a figure (A-SK55/KS-SK-90-1)"
    exit 1
  fi
  rm -f "$FORGE"
  if [ -f "$M84" ]; then
    # A-SK56 bite: fabricated "N B = X MiB [derivata: companion]" — the
    # companion self-verifies; the MEASURED byte token must still be
    # judged against the corpus (figure-scope, not row-scope).
    cp "$M84" "$TMP/MEASURE84_doctored3.md"
    echo "W=9 999.948.288 B = 953,56 MiB [derivata: companion /1048576]" >> "$TMP/MEASURE84_doctored3.md"
    if bash "$0" "$TMP/MEASURE84_doctored3.md" >/dev/null 2>&1; then
      rm -rf "$TMP"
      echo "SELFTEST FAIL: fabricated byte token on [derivata] row NOT caught (A-SK56/KS-SK-90-2)"
      exit 1
    fi
    # A-SK53-bis bite: a '±' band on a line with NO [KMGT] unit escaped
    # the old row-scope tooth entirely — window scope must catch it.
    cp "$M84" "$TMP/MEASURE84_doctored4.md"
    echo "tolleranza dichiarata ±7% sul floor, onesta" >> "$TMP/MEASURE84_doctored4.md"
    if bash "$0" "$TMP/MEASURE84_doctored4.md" >/dev/null 2>&1; then
      rm -rf "$TMP"
      echo "SELFTEST FAIL: unitless ± band NOT caught (A-SK53-bis/KS-SK-87-2)"
      exit 1
    fi
  fi
  rm -rf "$TMP"
  echo "SELFTEST PASS: smuggled figure caught (KG-83-3) + bytes-first teeth (A-SK40) + committed-only corpus (A-SK55) + figure-scope derivata (A-SK56) + window ± (A-SK53-bis) all bite"
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
  9276
  0.8
  1.019
);
# 110    = righe-per-run ENFORCE del protocollo (design78 driver)
# 384    = bound del buffer CString di stack in std (A-BB27, costante di libreria)
# 6910767= git rev baseline WP-80 (identita', enforced dal driver MEASURE80)
# 4000/8048/5000 = soglie ex-ante design79 par.10 (P1a/P1b/P5b) — anche nel
#                  corpus via verdict81.out, tenute qui per robustezza
# 9276   = wc -c del fixture hello_pad85 COMMITTATO (MEASURE85 riga sorgente;
#          A-SK56 lo espone: era esente per scope-di-riga, ora nominato)
# 0.8    = soglia alta ex-ante VATTR (verdict89.sh r.402 COMMITTED pre-run;
#          dichiarata nei doc per sanatoria KB-91-2/A-BB64 — dal verdict90
#          in poi la soglia vive nel header pre-run del giudice)
# 1.019  = ratio anti-moda W8, ricomputo Bak (Concilio WP-91 verbale sedia 5
#          COMMITTED) — robustezza mostrata FUORI banda per la sanatoria
#          A-BB64; entra in-band dal verdict90 (A-BB62)

# ---- corpus: committed machine outputs -------------------------------------
# A-SK55 (Council WP-90, Klabnik FORGE BITTEN LIVE): the corpus is read
# from the COMMITTED tree at HEAD (git ls-tree + git show HEAD:), never
# from the working tree — an uncommitted forge file in measure-out used
# to legalize any figure (KS-SK-90-1: a figure legalized by a file not
# reachable from HEAD voids the gate's PASS). bsd_glob is kept ONLY to
# express the patterns; matching runs against the HEAD file list.
my $root = qx(git -C "$here" rev-parse --show-toplevel 2>/dev/null);
chomp $root;
die "gate-measure-cifre: not in a git repo (A-SK55 needs HEAD)\n" unless $root;
my @headtree = split /\n/, qx(git -C "$root" ls-tree -r --name-only HEAD);
my %headset = map { $_ => 1 } @headtree;
sub committed_glob {
  my ($absglob) = @_;
  my $pat = $absglob;
  1 while $pat =~ s{/[^/]+/\.\./}{/};      # normalize dir/../
  return () unless index($pat, "$root/") == 0;
  $pat = substr($pat, length("$root/"));
  my $rx = join '', map { $_ eq '*' ? '[^/]*' : $_ eq '?' ? '[^/]' : quotemeta $_ }
                    split /(\*|\?)/, $pat;
  return grep { /^$rx$/ } @headtree;
}
my @sources;
push @sources, committed_glob("$here/*.out"), committed_glob("$here/../wp82-harness/*.out");
push @sources, committed_glob("$mout/*81*.summary"), committed_glob("$mout/*81*.matrix"),
               committed_glob("$mout/*81*.idle"),    committed_glob("$mout/*81*.census"),
               committed_glob("$mout/*81*.log");
# S-82.0: the measure82 campaign raws (82*/m82*/m82r* labels)
push @sources, committed_glob("$mout/*82*.summary"), committed_glob("$mout/*82*.census"),
               committed_glob("$mout/*82*.log"),     committed_glob("$mout/m82*"),
               committed_glob("$mout/axum.82*");
# S-83.0: the measure83 campaign raws (83*/m83* labels) + verdict83
push @sources, committed_glob("$mout/*83*.summary"), committed_glob("$mout/*83*.census"),
               committed_glob("$mout/*83*.log"),     committed_glob("$mout/m83*"),
               committed_glob("$mout/axum.83*"),     committed_glob("$here/../wp83-harness/*.out");
# S-84.0: the measure84 campaign raws (84*/m84* labels) + verdict84 + the
# A-DS29 fixture-oracle ledger
push @sources, committed_glob("$mout/*84*.summary"), committed_glob("$mout/*84*.census"),
               committed_glob("$mout/*84*.log"),     committed_glob("$mout/m84*"),
               committed_glob("$mout/axum.84*"),     committed_glob("$here/../wp84-harness/*.out"),
               committed_glob("$here/../wp84-harness/evidence/*");
# S-86.0: the measure86 campaign raws (86*/m86* labels) + verdict86 + the
# wp86 harness outs
push @sources, committed_glob("$mout/*86*.summary"), committed_glob("$mout/*86*.census"),
               committed_glob("$mout/*86*.log"),     committed_glob("$mout/m86*"),
               committed_glob("$mout/axum.86*"),     committed_glob("$here/../wp86-harness/*.out");
# S-87.0: the WP-88 re-judgment machine outputs (rejudge86 & friends)
push @sources, committed_glob("$here/../wp87-harness/*.out");
# S-88.0: the measure88 campaign raws (m88* labels) + verdict88 (per-attempt,
# per-generation .out) in wp88-harness
push @sources, committed_glob("$mout/m88*"), committed_glob("$here/../wp88-harness/*.out");
# S-89.0: the measure89 campaign raws (m89* labels) + verdict89 (per-attempt,
# per-generation .out) + ds35/ds40 verify pins in wp89-harness
push @sources, committed_glob("$mout/m89*"), committed_glob("$here/../wp89-harness/*.out");
# S-85.0: the measure85 campaign raws (85*/m85* labels) + verdict85 + the
# wp85 evidence dir
push @sources, committed_glob("$mout/*85*.summary"), committed_glob("$mout/*85*.census"),
               committed_glob("$mout/*85*.log"),     committed_glob("$mout/m85*"),
               committed_glob("$mout/axum.85*"),     committed_glob("$here/../wp85-harness/*.out"),
               committed_glob("$here/../wp85-harness/evidence/*");
push @sources, committed_glob("$here/evidence/*");
die "gate-measure-cifre: EMPTY corpus (no committed sources found)\n" unless @sources;
my (%corpus, %corpus_count);
# corpus cache (--all mode): first line = HEAD rev; C<token> / N<count>.
my $cache = $ENV{GATE_CIFRE_CORPUS_CACHE} || '';
my $headrev = qx(git -C "$root" rev-parse HEAD); chomp $headrev;
my $cache_loaded = 0;
if ($cache && -s $cache) {
  open my $ch, '<', $cache or die "cannot read corpus cache\n";
  my $first = <$ch> // ''; chomp $first;
  if ($first eq $headrev) {
    while (my $r = <$ch>) {
      chomp $r;
      if    ($r =~ s/^C//) { $corpus{$r} = 1 }
      elsif ($r =~ s/^N//) { $corpus_count{$r} = 1 }
    }
    $cache_loaded = 1;
  }
  close $ch;
}
if (!$cache_loaded) {
for my $f (@sources) {
  next if $f =~ m{/\._};             # AppleDouble
  # A-SK55: content from HEAD, never the working tree (list-form pipe:
  # no shell quoting issues under "/Volumes/Extreme Pro/")
  open my $fh, '-|', 'git', '-C', $root, 'show', "HEAD:$f" or next;
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
for my $f (committed_glob("$here/evidence/*.fails")) {
  next if $f =~ m{/\._};
  open my $cfh, '-|', 'git', '-C', $root, 'show', "HEAD:$f" or next;
  my $n = 0; $n++ while <$cfh>; close $cfh;
  $corpus_count{$n} = 1;
}
if ($cache) {
  open my $ch, '>', $cache or die "cannot write corpus cache\n";
  print $ch "$headrev\n";
  print $ch "C$_\n" for keys %corpus;
  print $ch "N$_\n" for keys %corpus_count;
  close $ch;
}
}  # !$cache_loaded

# ---- scan target -----------------------------------------------------------
open my $fh, '<', $target or die "cannot open $target\n";
my ($ln, @miss) = (0);
my %derived_ok;   # A-SK56: byte tokens legalized by verified arithmetic
# A-DL26 (Council WP-85, KL-85-2) + A-SK40 (Council WP-86): from MEASURE84
# on, every MEMORY figure must print BYTES FIRST ("N B = X MiB") — and the
# companion is VERIFIED: bytes/scale must round to the displayed figure.
# Units are [KMGT]i?B case-INSENSITIVE (mb/mib no longer evade); the check
# is per-FIGURE, not per-line (a stray "x W" in prose no longer exempts the
# whole line). Per-FIGURE exceptions: a protocol pin band "N±M <unit>"
# (a cited identity band, not a measure). Project convention: MB==MiB
# (this corpus has always done binary math; K/G/T likewise binary).
my $bytes_first = $target =~ /MEASURE8[4-9]|MEASURE9\d/;
sub it_num { my $s = shift; $s =~ s/\.(?=\d{3}\b)//g; $s =~ s/,/./; return $s; }
my %SCALE = (k => 1024, m => 1024**2, g => 1024**3, t => 1024**4);
while (my $line = <$fh>) {
  $ln++;
  if ($bytes_first) {
    my $work = $line;
    # (1) verified pairs "N B = X <unit>": check numerically, then blank out
    while ($work =~ /((\d[\d.,]*)\s*B\s*=\s*(\d[\d.,]*)\s*([KMGTkmgt])(i?)[Bb]\b)/) {
      my ($whole, $braw, $fraw, $u, $ii) = ($1, $2, $3, $4, $5);
      # A-SK43 (Council WP-87, Klabnik Q2): the corpus does BINARY math —
      # "MB" under SI reading lies to the outside reader. MEASURE8[4-9]
      # docs must spell the i (MiB/KiB/GiB/TiB).
      if ($ii eq '') {
        push @miss, "line $ln: unit '${u}B' without the i — MiB-only in MEASURE8[4-9] (A-SK43): $line";
      }
      my $bytes = it_num($braw);
      my $fig   = it_num($fraw);
      my $dec   = ($fig =~ /\.(\d+)$/) ? length($1) : 0;
      my $tol   = 0.5 * 10 ** (-$dec) + 1e-9;
      my $calc  = $bytes / $SCALE{lc $u};
      if (abs($calc - $fig) > $tol) {
        push @miss, sprintf("line %d: companion MISMATCH (A-SK40): %s B / %s = %.4f, displayed %s: %s",
                            $ln, $bytes, lc($u)."iB-scale", $calc, $fig, $line);
      }
      my $blank = ' ' x length($whole);
      $work =~ s/\Q$whole\E/$blank/;
    }
    # (2) per-FIGURE band ALLOWLIST (A-SK43/KS-SK-87-2): the old rule
    # blanked ANY "N±M <unit>" — with a 1-2 digit M a doc could widen a
    # pin at will ("232±16 MiB" passed companion, band and corpus). Only
    # the NAMED legal bands survive; today that is 232±1 MiB, e basta.
    while ($work =~ /(\d[\d.,]*\s*±\s*\d[\d.,]*\s*[KMGTkmgt]i?[Bb])\b/g) {
      my $band = $1;
      (my $norm = $band) =~ s/\s+//g;
      push @miss, "line $ln: ± band '$band' not in the A-SK43 allowlist (KS-SK-87-2): $line"
        unless $norm eq '232±1MiB';
    }
    $work =~ s/232\s*±\s*1\s*MiB\b/' ' x length($&)/ge;
    # (2b) A-SK53-bis (Council WP-90, Klabnik — re-issued after being
    # DROPPED from the WP-89 synthesis; supersedes the A-SK48 row-scope
    # tooth): ± judged at WINDOW scope on the WHOLE line, allowlist-only.
    # The old tooth fired only when the line carried a [KMGT] unit — a
    # percent band on a bare-B line ("… B ±5%") escaped unjudged. Now:
    # strip every allowlisted band (normalized, whitespace-free) from the
    # line; ANY surviving '±' in a bytes-first doc is an unlisted band.
    # Allowlist (each entry NAMED):
    #   232±1MiB        = axum W1 identity pin band (A-SK43)
    #   3.605.572B±5%   = banda KL-85-2 — RITIRATA (KB-90-2): legal ONLY
    #                     as the target of a NAMED-DEVIATION citation in
    #                     MEASURE87/88 (historical record); never as a
    #                     live protocol band in future docs.
    # Per-DOC entries (named, historical prose forms in FROZEN docs —
    # never granted to future MEASURE docs):
    #   MEASURE85: '232±1' (the axum pin cited unitless in prose),
    #              '±5%'   (bare tolerance operator, no figure attached)
    {
      (my $flat = $line) =~ s/\s+//g;
      my @bands = ('232±1MiB', '3.605.572B±5%');
      push @bands, '232±1', '±5%' if $target =~ /MEASURE85/;
      for my $band (sort { length($b) <=> length($a) } @bands) {
        $flat =~ s/\Q$band\E(?!\d)//g;
      }
      if ($flat =~ /±/) {
        push @miss, "line $ln: '±' band not in the A-SK53-bis window allowlist (KS-SK-87-2): $line";
      }
    }
    # (2c) A-SK48 cosmetic tooth: "Mib" (bits sold as bytes) never legal.
    if ($line =~ /\b\d[\d.,]*\s*[KMGT]ib\b/) {
      push @miss, "line $ln: unit spelled '<X>ib' (bit, not byte) on a figure (A-SK48): $line";
    }
    # (3) any remaining unit figure lacks its bytes companion
    while ($work =~ /(\d[\d.,]*\s*[KMGTkmgt]i?[Bb])\b/g) {
      push @miss, "line $ln: memory figure '$1' without VERIFIED bytes-first companion (A-DL26/KL-85-2/A-SK40): $line";
    }
  }
  if ($line =~ /\[derivata/) {                   # A-BG26 tagged line
    # A-SK56 (Council WP-90, Klabnik FORGE BITTEN LIVE): the tag is at
    # FIGURE scope, not row scope — it exempts DERIVED figures (verified
    # companions, ratios, deltas WITHOUT unit) but NEVER a measured-form
    # byte token: every ">=3-digit N B" on a tagged line stays
    # corpus-bound (a fabricated "N B = X MiB [derivata: companion]"
    # self-verifies its companion and used to pass whole — KS-SK-90-2).
    if ($bytes_first) {
      # A derived BYTE figure is legal iff its derivation is VERIFIED at
      # machine: the tag on the line carries an "X−Y" (or X-Y) expression
      # whose operands are BOTH in the committed corpus and whose result
      # equals the token exactly. Once legalized, later citations of the
      # SAME token in this doc are legal (wrapped repeats carry the tag
      # but not the expression). Everything else: corpus-bound.
      my %eval_ok;
      if ($line =~ /\[derivata:([^\]]*)/) {
        my $expr = $1;
        # U+2212 MINUS is multibyte: a [−-] class would split it into
        # bytes under non-utf8 perl — spell the sequence out.
        while ($expr =~ /(\d(?:[\d.,]*\d)?)\s*(?:\xE2\x88\x92|-)\s*(\d(?:[\d.,]*\d)?)/g) {
          my ($xa, $xb) = (it_num($1), it_num($2));
          next unless ($corpus{$xa} || $ALLOW{$xa}) && ($corpus{$xb} || $ALLOW{$xb});
          $eval_ok{$xa - $xb} = 1;
        }
      }
      my $probe2 = $line;
      $probe2 =~ s/\b(?=[0-9a-f]*[a-f])[0-9a-f]{7,}\b//gi;
      while ($probe2 =~ /(?<![\dA-Za-z,.±])(\d{1,3}(?:\.\d{3})+|\d{3,})\s*B(?![A-Za-z0-9])/g) {
        my $raw = $1;
        (my $norm = $raw) =~ s/\.(?=\d{3}\b)//g;
        next if $ALLOW{$norm} || $corpus{$norm} || $corpus_count{$norm};
        if ($eval_ok{$norm} || $derived_ok{$norm}) { $derived_ok{$norm} = 1; next; }
        push @miss, "line $ln: byte token '$raw B' on a [derivata] line NOT in corpus and NOT machine-verified from corpus operands — the tag is figure-scope (A-SK56/KS-SK-90-2): $line";
      }
    }
    next;
  }
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
