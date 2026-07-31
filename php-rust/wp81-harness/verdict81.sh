#!/bin/bash
# verdict81.sh v2 — S-82.0 p1 (Council WP-83): fail-closed verdict tooling.
# Reads the 81.* campaign raws (wp78-harness/measure-out) and judges the
# design79 §10 predictions BY MACHINE. v2 changes (binding emendamenti):
#   A-SK19  : absent/empty JUDGED field = FAIL d'ufficio (KS-SK-83-1);
#             steady_n pinned ==100 for every run, presence-pinned per field.
#   A-SK20  : P* judged on the MEAN of R=3, never r1-only; per-field spread
#             (b/resid/retain included) emitted IN the verdict (A-BG28).
#   KG-83-1 : md5 byte-identity r1/r2/r3 computed BY THIS SCRIPT; a P* on a
#             single run is only legal behind that gate (here: always mean).
#   KG-83-2 : judged field with spread beyond its declared band => that P*
#             FAILs (never silently VOID). Bands: counters 0.5%, b/resid 5%.
#   A-BG26  : every DERIVED figure (percentages, cross-campaign deltas) is
#             emitted HERE with formula+sources (D* lines) — MEASURE81 cites
#             these lines; hand transcription of derivates is banned.
#   A-BG27  : a2 NAMED in the table (the 2 residual HIT calls live in a2),
#             bytes printed next to calls for every judged channel.
#   A-BB30  : the req=1 MISS row is emitted as pin (put+park live there,
#             OUTSIDE the steady window) — "b invariant" is only readable
#             next to it.
# Predictions P1-P8 (design79 §10) unchanged in the bounds, judged on means:
#   P1  hello a_calls(HIT) < 4000 AND < 8048 (KS-AH-80-4 v2 VOID-gate)
#   P2  a1_calls == 0 steady   P3  a3_calls == 0 steady
#   P4  bare a_calls(HIT) <= 200 (measured ex-post floor, KB-82-3)
#   P5  resid(hello) 44.1+-15 calls / 43463+-5000 B (MEAN)
#   P6  b invariant +-5% (hello 730/95627; include_heavy 27982)
#   P7  retain_len 1/1/3/6 (bare/hello/include_gate/include_heavy)
#   P8  depth_max<=1, inflight_max<=1, a3_trip==0 on ALL runs (not just r1)
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
MOUT="${VERDICT81_MOUT:-$HERE/../wp78-harness/measure-out}"
AN="$HERE/../wp80-harness/analyze80.pl"

if [ "${1:-}" = "--selftest" ]; then
  # Negative control (the A-SK19 teeth must BITE — a gate that never fails
  # on a broken input is vacuous, WP-72 lesson).
  SRC="$HERE/../wp78-harness/measure-out"
  TMP=$(mktemp -d)
  # tooth 1: a judged field stripped from one raw => verdict must FAIL
  cp "$SRC"/*.81.*.census "$TMP"/
  sed -i '' 's/ retain_len=[0-9]*//g' "$TMP/census.81.hello.r1.census"
  if VERDICT81_MOUT="$TMP" bash "$0" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: stripped retain_len did NOT fail the verdict (KS-SK-83-1)"
    rm -rf "$TMP"; exit 1
  fi
  # tooth 2: truncated raw (steady_n < 100) => steady_n pin must FAIL
  cp "$SRC"/*.81.*.census "$TMP"/
  head -50 "$SRC/census.81.hello.r2.census" > "$TMP/census.81.hello.r2.census"
  if VERDICT81_MOUT="$TMP" bash "$0" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: truncated raw (steady_n<100) did NOT fail (A-SK19)"
    rm -rf "$TMP"; exit 1
  fi
  rm -rf "$TMP"
  echo "SELFTEST PASS: field-strip and steady_n teeth both bite (A-SK19/KS-SK-83-1)"
  exit 0
fi

perl - "$MOUT" "$AN" <<'PERL'
use strict; use warnings;
my ($mout, $an) = @ARGV;
my $fails = 0;
sub fail { print "FAIL: @_\n"; $fails++ }
sub fmt  { my $v = shift; my $s = sprintf('%.1f', $v); $s =~ s/\.0$//; $s }

my @ARMS = qw(census censuscli);
my @FX   = qw(hello include_gate include_heavy bare);
# judged fields per arm — presence-pinned (A-SK19); cli arm limited by the
# DECLARED superglobals asymmetry (A-DS10/KS-DS-81-3): total/a1/a3 only.
my %JUDGED = (
  census    => [qw(a_calls a_bytes a1_calls a1_bytes a2_calls a2_bytes
                   a3_calls a3_bytes b_calls b_bytes resid_calls resid_bytes
                   retain_len)],
  censuscli => [qw(total_calls total_bytes a1_calls a3_calls)],
);
sub band { my $f = shift; return ($f =~ /^(b_|resid_)/) ? 5.0 : 0.5 }

my %M;
for my $arm (@ARMS) { for my $fx (@FX) {
  my (%vals, @md5s, %trip);
  for my $r (1..3) {
    my $raw = "$mout/$arm.81.$fx.r$r.census";
    unless (-f $raw) { fail "missing raw $raw"; next }
    my $md5 = `md5 -q "$raw"`; chomp $md5; push @md5s, $md5;
    my $line = `perl "$an" "$raw" 2>/dev/null`; chomp $line;
    unless ($line) { fail "analyze produced NOTHING for $raw (A-SK19)"; next }
    my %f; while ($line =~ /([a-z0-9_]+)=([0-9.]+|absent)/g) { $f{$1} = $2 }
    if (!defined $f{steady_n} || $f{steady_n} eq '' ) {
      fail "$arm/$fx r$r: steady_n ABSENT (A-SK19)";
    } elsif ($f{steady_n} != 100) {
      fail "$arm/$fx r$r: steady_n=$f{steady_n} != 100 (A-SK19 pin)";
    }
    for my $k (@{$JUDGED{$arm}}) {
      if (!defined $f{$k} || $f{$k} eq '' || $f{$k} eq 'absent') {
        fail "$arm/$fx r$r: judged field $k ABSENT/EMPTY (KS-SK-83-1)";
      } else { push @{$vals{$k}}, $f{$k} }
    }
    if ($arm eq 'census') {
      for my $t (qw(depth_max_max inflight_max_max a3_trip_max)) {
        if (!defined $f{$t} || $f{$t} eq 'absent') {
          fail "$arm/$fx r$r: tripwire field $t ABSENT (KS-SK-83-1)";
        } else {
          $trip{$t} = $f{$t} if !defined $trip{$t} || $f{$t} > $trip{$t};
        }
      }
    }
  }
  my $ident = (@md5s == 3 && $md5s[0] eq $md5s[1] && $md5s[1] eq $md5s[2]) ? 1 : 0;
  $M{$arm}{$fx}{ident} = $ident;
  $M{$arm}{$fx}{md5}   = substr($md5s[0] // 'NA', 0, 8);
  $M{$arm}{$fx}{trip}  = \%trip;
  for my $k (keys %vals) {
    my @v = @{$vals{$k}};
    if (@v != 3) { fail "$arm/$fx field $k present on ".scalar(@v)."/3 runs"; next }
    my ($mn, $mx) = ((sort { $a <=> $b } @v)[0], (sort { $a <=> $b } @v)[-1]);
    my $mean   = ($v[0] + $v[1] + $v[2]) / 3;
    my $spread = $mn == 0 ? ($mx == 0 ? 0 : -1) : ($mx - $mn) * 100 / $mn;
    if ($spread < 0) { fail "$arm/$fx $k: min==0 with max=$mx — spread undefined (KG-83-2)"; $spread = 9e9 }
    $M{$arm}{$fx}{mean}{$k}   = $mean;
    $M{$arm}{$fx}{spread}{$k} = $spread;
    $M{$arm}{$fx}{runs}{$k}   = \@v;
    if (!$ident && $spread > band($k)) {
      fail sprintf("%s/%s %s spread %.2f%% > band %.1f%% (KG-83-2) — P* on this field FAIL",
                   $arm, $fx, $k, $spread, band($k));
    }
  }
  # req=1 MISS row pin (A-BB30): first census row of r1, raw counters
  my $raw1 = "$mout/$arm.81.$fx.r1.census";
  if (-f $raw1 && $arm eq 'census') {
    open my $fh, '<', $raw1 or fail "cannot open $raw1";
    my %r1;
    while (<$fh>) {
      next unless /^census(?:-cli)?: /;
      while (/([a-z0-9_]+)=(\d+)/g) { $r1{$1} = $2 }
      last;
    }
    close $fh;
    if (defined $r1{a_calls}) {
      $M{$arm}{$fx}{req1} = { a_calls => $r1{a_calls}, a_bytes => $r1{a_bytes} // 'NA',
                              retain_len => $r1{retain_len} // 'NA' };
    } else { fail "$arm/$fx: no req=1 census row for A-BB30 pin" }
  }
}}

print "# verdict81 v2 — lever-arm steady MEANS of R=3 (gross churn, >= upper bound)\n";
print "# generated by wp81-harness/verdict81.sh (KG-82-1/A-BG26) — do not hand-edit\n";
print "# P* judged on means (A-SK20); spread per-field in-table (A-BG28/KG-83-2)\n\n";

print "## md5 byte-identity gate (KG-83-1)\n";
for my $arm (@ARMS) { for my $fx (@FX) {
  my $i = $M{$arm}{$fx}{ident} // 0;
  printf "IDENT %s/%s: %s (md5 %s)\n", $arm, $fx,
    ($i ? "r1==r2==r3 BYTE-IDENTICAL" : "runs DIFFER — mean+spread judge"),
    $M{$arm}{$fx}{md5} // 'NA';
}}
print "\n";

print "| arm | fixture | a_calls m(r1/r2/r3) | spr% | a_bytes | a1 c/B | a2 c/B | a3 c/B | b c/B | resid c/B | retain |\n";
print "|---|---|---|---|---|---|---|---|---|---|---|\n";
for my $arm (@ARMS) { for my $fx (@FX) {
  my $m = $M{$arm}{$fx}{mean} or next;
  my $r = $M{$arm}{$fx}{runs};
  my $s = $M{$arm}{$fx}{spread};
  my $key = $arm eq 'census' ? 'a_calls' : 'total_calls';
  next unless defined $m->{$key};
  my $runs = join('/', @{$r->{$key}});
  if ($arm eq 'census') {
    printf "| %s | %s | %s (%s) | %.2f | %s | %s/%s | %s/%s | %s/%s | %s/%s | %s/%s | %s |\n",
      $arm, $fx, fmt($m->{a_calls}), $runs, $s->{a_calls},
      fmt($m->{a_bytes}),
      fmt($m->{a1_calls}), fmt($m->{a1_bytes}),
      fmt($m->{a2_calls}), fmt($m->{a2_bytes}),
      fmt($m->{a3_calls}), fmt($m->{a3_bytes}),
      fmt($m->{b_calls}),  fmt($m->{b_bytes}),
      fmt($m->{resid_calls}), fmt($m->{resid_bytes}),
      fmt($m->{retain_len});
  } else {
    printf "| %s | %s | %s (%s) | %.2f | %s | %s/— | — | %s/— | — | — | — |\n",
      $arm, $fx, fmt($m->{total_calls}), $runs, $s->{total_calls},
      fmt($m->{total_bytes}), fmt($m->{a1_calls}), fmt($m->{a3_calls});
  }
}}
print "\n";

print "## req=1 MISS pin (A-BB30: put+park live here, OUTSIDE the steady window)\n";
for my $fx (@FX) {
  my $q = $M{census}{$fx}{req1} or next;
  printf "REQ1 census/%s: a_calls=%s a_bytes=%s retain_len=%s\n",
    $fx, $q->{a_calls}, $q->{a_bytes}, $q->{retain_len};
}
print "\n";

# ---- predictions (census arm judges; cli = declared-asymmetry twin) --------
my $H = $M{census}{hello}{mean} || {};
my $B = $M{census}{bare}{mean} || {};
my $IH = $M{census}{include_heavy}{mean} || {};
sub need { my ($h,$k,$who) = @_;
  return $h->{$k} if defined $h->{$k};
  fail "$who: field $k missing at judgment time (KS-SK-83-1)"; return undef }

my $AC = need($H,'a_calls','P1');
if (defined $AC) {
  $AC < 4000 ? print "P1a PASS: hello a_calls mean=".fmt($AC)." < 4000\n"
             : fail "P1a hello a_calls=".fmt($AC)." >= 4000";
  $AC < 8048 ? print "P1b PASS: >=90% drop (a_calls=".fmt($AC)." < 8048, KS-AH-80-4 v2)\n"
             : fail "P1b VOID-gate: a_calls=".fmt($AC)." >= 8048";
}
my $A1 = need($H,'a1_calls','P2');
if (defined $A1) { $A1 == 0 ? print "P2  PASS: a1_calls==0 steady\n" : fail "P2 a1_calls=".fmt($A1)." != 0" }
my $A3 = need($H,'a3_calls','P3');
if (defined $A3) { $A3 == 0 ? print "P3  PASS: a3_calls==0 steady\n" : fail "P3 a3_calls=".fmt($A3)." != 0" }
my $BA = need($B,'a_calls','P4');
if (defined $BA) {
  $BA <= 200 ? print "P4  PASS: bare a_calls(HIT) mean=".fmt($BA)." <= 200 (measured ex-post floor)\n"
             : fail "P4 KB-82-3: bare a_calls=".fmt($BA)." > 200 — floor bucato, rideriva PER NOME";
}
my $RC = need($H,'resid_calls','P5a'); my $RB = need($H,'resid_bytes','P5b');
if (defined $RC) {
  ($RC >= 29.1 && $RC <= 59.1) ? print "P5a PASS: resid_calls mean=".fmt($RC)." in 44.1+-15\n"
                               : fail "P5a resid_calls=".fmt($RC)." outside 44.1+-15";
}
if (defined $RB) {
  ($RB >= 38463 && $RB <= 48463) ? print "P5b PASS: resid_bytes mean=".fmt($RB)." in 43463+-5000\n"
                                 : fail "P5b resid_bytes=".fmt($RB)." outside 43463+-5000";
}
my $BC = need($H,'b_calls','P6a');
if (defined $BC) {
  ($BC >= 693.5 && $BC <= 766.5) ? print "P6a PASS: hello b_calls mean=".fmt($BC)." within +-5% of 730\n"
                                 : fail "P6a b_calls=".fmt($BC)." outside 730+-5%";
}
my $BH = need($IH,'b_calls','P6b');
if (defined $BH) {
  ($BH >= 26582.9 && $BH <= 29381.1) ? print "P6b PASS: include_heavy b_calls mean=".fmt($BH)." within +-5% of 27982\n"
                                     : fail "P6b include_heavy b_calls=".fmt($BH)." outside 27982+-5%";
}
my %RETW = (bare => 1, hello => 1, include_gate => 3, include_heavy => 6);
for my $fx (@FX) {
  my $rv = need($M{census}{$fx}{mean} || {}, 'retain_len', "P7 $fx");
  next unless defined $rv;
  $rv == $RETW{$fx} ? print "P7  PASS: $fx retain_len==$RETW{$fx}\n"
                    : fail "P7 $fx retain_len=".fmt($rv)." != $RETW{$fx}";
}
for my $fx (@FX) {
  my $t = $M{census}{$fx}{trip} || {};
  my ($dm, $im, $tp) = ($t->{depth_max_max}, $t->{inflight_max_max}, $t->{a3_trip_max});
  if (!defined $dm || !defined $im || !defined $tp) {
    fail "P8 $fx: tripwire fields missing (KS-SK-83-1)"; next;
  }
  ($dm <= 1 && $im <= 1 && $tp == 0)
    ? print "P8  PASS: $fx depth<=1 inflight<=1 trip==0 (max over R=3)\n"
    : fail "P8 $fx tripwires: depth=$dm inflight=$im trip=$tp";
}

# ---- derived figures (A-BG26): emitted HERE, cited by MEASURE81 ------------
print "\n## derived figures (A-BG26 — formula+sources; MEASURE cites these lines)\n";
# Baselines: wp80-harness/MEASURE80_RESULTS.md (git 6910767, census 5c9c6eec).
my %BASE = (hello_a => 80476, hello_b => 730, cli_hello => 81613,
            cli_heavy => 109415, heavy_b => 27982);
if (defined $AC && $BASE{hello_a}) {
  printf "D1 [derivata] hello a_calls HIT drop = (1 - %s/%d)*100 = %.4f%%  [src: census.81.hello mean; baseline MEASURE80 hello a_calls=%d]\n",
    fmt($AC), $BASE{hello_a}, (1 - $AC/$BASE{hello_a})*100, $BASE{hello_a};
}
if (defined $BC && $BASE{hello_b}) {
  printf "D2 [derivata] hello b_calls delta = (%s/%d - 1)*100 = %+.2f%%  [src: census.81.hello mean; baseline MEASURE80 b=%d]\n",
    fmt($BC), $BASE{hello_b}, ($BC/$BASE{hello_b} - 1)*100, $BASE{hello_b};
}
my $CT = ($M{censuscli}{hello}{mean} || {})->{total_calls};
if (defined $CT && $BASE{cli_hello}) {
  printf "D3 [derivata] cli hello total drop = (1 - %s/%d)*100 = %.2f%%  [src: censuscli.81.hello mean; baseline MEASURE80 cli total=%d]\n",
    fmt($CT), $BASE{cli_hello}, (1 - $CT/$BASE{cli_hello})*100, $BASE{cli_hello};
}
my $CH = ($M{censuscli}{include_heavy}{mean} || {})->{total_calls};
if (defined $CH && $BASE{cli_heavy}) {
  printf "D4 [derivata] cli include_heavy total drop = (1 - %s/%d)*100 = %.2f%%  [src: censuscli.81.include_heavy mean; baseline MEASURE80 cli total=%d]\n",
    fmt($CH), $BASE{cli_heavy}, (1 - $CH/$BASE{cli_heavy})*100, $BASE{cli_heavy};
}
if (defined $IH->{b_calls} && $BASE{heavy_b}) {
  printf "D5 [derivata] include_heavy b_calls delta = (%s/%d - 1)*100 = %+.2f%%  [src: census.81.include_heavy mean; baseline MEASURE80 b=%d]\n",
    fmt($IH->{b_calls}), $BASE{heavy_b}, ($IH->{b_calls}/$BASE{heavy_b} - 1)*100, $BASE{heavy_b};
}
# a2 channel NAMED (A-BG27): the residual HIT calls live in a2, not "absorbed"
if (defined $H->{a2_calls}) {
  printf "D6 a2 NAMED (A-BG27): hello steady a2_calls=%s a2_bytes=%s — the 2 residual HIT calls LIVE IN a2 (probe: PathBuf canonicalize + Vec UnitKey), not \"absorbed\"\n",
    fmt($H->{a2_calls}), fmt($H->{a2_bytes} // -1);
}

if ($fails == 0) { print "\n== VERDICT81 PASS (all par.10 predictions, judged on R=3 means) ==\n"; exit 0 }
else             { print "\n== VERDICT81 FAIL($fails) ==\n"; exit 1 }
PERL
