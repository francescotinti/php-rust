#!/bin/bash
# gate-measure-cifre.sh — KG-83-3 (Council WP-83): every numeric figure in a
# MEASURE document must match a line of COMMITTED machine output (.out,
# .summary, .matrix, .idle, raw .census/.log), or be a verified bytes-first
# companion, or be legalized by a PROVENANCE-resolved [derivata: prov ...]
# (A-SK60, Council WP-91 — the free X−Y evaluator is abolished; label-only
# tags legalize nothing), or be a NAMED protocol constant in the allowlist
# below. Perimeter, bytes-first flags and ± graces come from the committed
# manifest (A-SK63/64). Machine check, not promise.
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
# A-SK65: cache ONLY via argv (+nonce); the env var is dead and IGNORED.
CACHE_ARG=""
NONCE_ARG=""
while [ $# -gt 0 ]; do
  case "$1" in
    --cache) CACHE_ARG="${2:-}"; shift 2;;
    --nonce) NONCE_ARG="${2:-}"; shift 2;;
    *) break;;
  esac
done
TARGET="${1:-$HERE/MEASURE81_RESULTS.md}"
MOUT="$HERE/../wp78-harness/measure-out"

# A-SK64 (Council WP-91, Klabnik — supersedes the A-SK40 name-glob perimeter):
# --all takes its perimeter from the COMMITTED MANIFEST, never from filename
# regexes (a forged name inherited graces; MEASURE100 would have dropped out
# of bytes-first at WP-100 — KS-SK-91-2). Bidirectional FAIL: an entry whose
# file is missing FAILS; a MEASURE*_RESULTS.md in the tree without a manifest
# entry FAILS. judge=no rows are DECLARED exclusions (named, with reason in
# the manifest comments), no longer silent glob gaps.
MANIFEST="$HERE/gate-cifre-manifest.tsv"
if [ "${1:-}" = "--all" ]; then
  rc=0
  [ -f "$MANIFEST" ] || { echo "FAIL gate-measure-cifre --all: manifest $MANIFEST missing (A-SK64)"; exit 1; }
  # A-SK65 (Council WP-91, Klabnik): the corpus cache travels via ARGV with a
  # parent-generated NONCE; the env var is IGNORED by the child (a poisoned
  # inherited GATE_CIFRE_CORPUS_CACHE used to legalize any figure —
  # KS-SK-91-3). The parent creates the file itself: no pre-existing path.
  CACHE=$(mktemp /tmp/gate-cifre-corpus.XXXXXX)
  NONCE=$(LC_ALL=C tr -dc 'a-f0-9' < /dev/urandom | head -c 16)
  while IFS=$'\t' read -r mpath msha mjudge mbf mbands; do
    case "$mpath" in ''|'#'*) continue;; esac
    f="$(git -C "$HERE" rev-parse --show-toplevel)/$mpath"
    if [ ! -f "$f" ]; then
      echo "FAIL gate-measure-cifre --all: manifest doc MISSING from tree: $mpath (A-SK64)"; rc=1; continue
    fi
    if [ "$mjudge" = "no" ]; then
      echo "SKIP $mpath (declared judge=no in manifest — A-SK64)"; continue
    fi
    bash "$0" --cache "$CACHE" --nonce "$NONCE" "$f" || rc=1
  done < "$MANIFEST"
  # reverse direction: every MEASURE*_RESULTS.md in the tree must be manifested
  ROOT="$(git -C "$HERE" rev-parse --show-toplevel)"
  for f in "$HERE"/../wp*-harness/MEASURE*_RESULTS.md; do
    [ -f "$f" ] || continue
    rel="${f#"$ROOT/"}"
    # normalize wp81-harness/../wpNN-harness → wpNN-harness
    rel=$(printf '%s\n' "$rel" | sed 's|[^/]*/\.\./||')
    grep -q "^$rel	" "$MANIFEST" || {
      echo "FAIL gate-measure-cifre --all: $rel has NO manifest entry (A-SK64 bidirectional)"; rc=1; }
  done
  rm -f "$CACHE"
  [ "$rc" = 0 ] && echo "PASS gate-measure-cifre --all (A-SK64): manifest perimeter, bidirectional"
  exit $rc
fi

if [ "${1:-}" = "--selftest" ]; then
  TMP=$(mktemp -d)
  M89DOC="$HERE/../wp89-harness/MEASURE89_RESULTS.md"
  M85DOC="$HERE/../wp85-harness/MEASURE85_RESULTS.md"
  # T0 — baseline: an INTACT copy of MEASURE89 must PASS from scratch
  # space (bytes-first fail-closed, zero graces): otherwise every FAIL
  # tooth below is vacuous (a gate that fails everything bites nothing).
  cp "$M89DOC" "$TMP/baseline.md"
  if ! bash "$0" "$TMP/baseline.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: intact MEASURE89 copy does NOT pass — teeth below would be vacuous"
    rm -rf "$TMP"; exit 1
  fi
  # T1 — KG-83-3 smuggle
  cp "$TMP/baseline.md" "$TMP/t1.md"
  echo "smuggled figure: a_calls was 123457 on a good day" >> "$TMP/t1.md"
  if bash "$0" "$TMP/t1.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: smuggled 123457 was NOT caught (KG-83-3)"; rm -rf "$TMP"; exit 1
  fi
  # T2 — A-SK40: naked lowercase unit figure without companion
  cp "$TMP/baseline.md" "$TMP/t2.md"
  echo "note: the cache costs 5,00 mib steady, honest." >> "$TMP/t2.md"
  if bash "$0" "$TMP/t2.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: naked lowercase 'mib' figure NOT caught (A-SK40)"; rm -rf "$TMP"; exit 1
  fi
  # T3 — A-SK40: companion that does not verify
  cp "$TMP/baseline.md" "$TMP/t3.md"
  echo "note: 1.048.576 B = 2,00 MiB [derivata: selftest]" >> "$TMP/t3.md"
  if bash "$0" "$TMP/t3.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: mismatching bytes companion NOT caught (A-SK40)"; rm -rf "$TMP"; exit 1
  fi
  # T4 — A-SK55: uncommitted forge in measure-out must not legalize
  FORGE="$MOUT/m88.zzforge-selftest.tmp"
  echo "committed=987654321" > "$FORGE"
  cp "$TMP/baseline.md" "$TMP/t4.md"
  echo "smuggled: campaign committed was 987654321 flat" >> "$TMP/t4.md"
  if bash "$0" "$TMP/t4.md" >/dev/null 2>&1; then
    rm -f "$FORGE"; rm -rf "$TMP"
    echo "SELFTEST FAIL: working-tree forge in measure-out legalized a figure (A-SK55/KS-SK-90-1)"; exit 1
  fi
  rm -f "$FORGE"
  # T5 — A-SK60: the EXACT Klabnik live forge (WP-91 Q1b) — fabricated
  # b_base with a free X−Y over corpus-present vmmap fragments. The
  # evaluator is abolished: this must FAIL.
  cp "$TMP/baseline.md" "$TMP/t5.md"
  echo "b_base rivisto = 19.600.000 B = 18,69 MiB per worker [derivata: 23.000.000 − 3.400.000]" >> "$TMP/t5.md"
  if bash "$0" "$TMP/t5.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: free X−Y [derivata] forge NOT caught (A-SK60/KS-SK-91-1)"; rm -rf "$TMP"; exit 1
  fi
  # T6 — A-SK62: fabricated 2σ lower bound inside brackets on a
  # [derivata] line (the second Klabnik bite: only "N B"-shaped tokens
  # were judged before).
  cp "$TMP/baseline.md" "$TMP/t6.md"
  echo "nota: 2σ = [11.111.111, 20.745.049] B [derivata: companion /1048576]" >> "$TMP/t6.md"
  if bash "$0" "$TMP/t6.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: fabricated bracket token on [derivata] line NOT caught (A-SK62/KS-SK-91-1)"; rm -rf "$TMP"; exit 1
  fi
  # T6b — A-SK56 historic tooth kept: fabricated "N B = X MiB" pair
  cp "$TMP/baseline.md" "$TMP/t6b.md"
  echo "W=9 999.948.288 B = 953,56 MiB [derivata: companion /1048576]" >> "$TMP/t6b.md"
  if bash "$0" "$TMP/t6b.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: fabricated byte token on [derivata] row NOT caught (A-SK56/KS-SK-90-2)"; rm -rf "$TMP"; exit 1
  fi
  # T7 — A-SK53-bis: unitless ± band
  cp "$TMP/baseline.md" "$TMP/t7.md"
  echo "tolleranza dichiarata ±7% sul floor, onesta" >> "$TMP/t7.md"
  if bash "$0" "$TMP/t7.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: unitless ± band NOT caught (A-SK53-bis/KS-SK-87-2)"; rm -rf "$TMP"; exit 1
  fi
  # T8 — A-SK63: a forged NAME must inherit NOTHING — a copy of
  # MEASURE85 (whose committed row grants '±5%' and '232±1') under a
  # forged name gets fail-closed defaults and must FAIL on its bands.
  cp "$M85DOC" "$TMP/MEASURE85_zzforge_RESULTS.md"
  if bash "$0" "$TMP/MEASURE85_zzforge_RESULTS.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: forged-name copy inherited the M85 graces (A-SK63/KS-SK-91-2)"; rm -rf "$TMP"; exit 1
  fi
  # T9 — A-SK65: a POISONED cache inherited via env must be IGNORED.
  HEADREV=$(git -C "$HERE" rev-parse HEAD)
  POISON="$TMP/poison.cache"
  { echo "$HEADREV"; echo "C777444111"; } > "$POISON"
  cp "$TMP/baseline.md" "$TMP/t9.md"
  echo "smuggled: threshold hit 777444111 flat" >> "$TMP/t9.md"
  if GATE_CIFRE_CORPUS_CACHE="$POISON" bash "$0" "$TMP/t9.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: poisoned env cache legalized a figure (A-SK65/KS-SK-91-3)"; rm -rf "$TMP"; exit 1
  fi
  # T10 — A-SK60 POSITIVE control + bite: a provenance-resolved derivata
  # must legalize its exact value (positive: the resolver is not dead
  # code) and must NOT legalize a different value (negative).
  G3REL="php-rust/wp89-harness/verdict89.a1.g3.out"
  cp "$TMP/baseline.md" "$TMP/t10.md"
  echo "delta se: 734.670 B [derivata: prov 1.319.393@$G3REL:115 − 584.723@$G3REL:63]" >> "$TMP/t10.md"
  if ! bash "$0" "$TMP/t10.md" 2>&1 | grep -q "provenance-verified"; then
    echo "SELFTEST FAIL: provenance resolver never fired on a legal derivata (A-SK60 positive control)"; rm -rf "$TMP"; exit 1
  fi
  if ! bash "$0" "$TMP/t10.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: legal provenance derivata did NOT pass (A-SK60 positive control)"; rm -rf "$TMP"; exit 1
  fi
  cp "$TMP/baseline.md" "$TMP/t10b.md"
  echo "delta se: 734.671 B [derivata: prov 1.319.393@$G3REL:115 − 584.723@$G3REL:63]" >> "$TMP/t10b.md"
  if bash "$0" "$TMP/t10b.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: provenance derivata legalized a WRONG value (A-SK60)"; rm -rf "$TMP"; exit 1
  fi
  # T12 — A-SK66: citing a NON-max generation without naming the
  # supersession on the line must FAIL; the same citation WITH the
  # supersession named must not add a miss (checked via max-gen cite).
  cp "$TMP/baseline.md" "$TMP/t12.md"
  echo "verdetto: verdict89.a1.g1.out dice la verita, fidatevi" >> "$TMP/t12.md"
  if bash "$0" "$TMP/t12.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: NON-max generation citation NOT caught (A-SK66/KS-SK-91-4)"; rm -rf "$TMP"; exit 1
  fi
  cp "$TMP/baseline.md" "$TMP/t12b.md"
  echo "storia: verdict89.a1.g1.out superseded da g3, ledgerato" >> "$TMP/t12b.md"
  if ! bash "$0" "$TMP/t12b.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: max-gen-declared supersession citation wrongly refused (A-SK66)"; rm -rf "$TMP"; exit 1
  fi
  # T11 — A-SK61: the corpus cardinality budget must bite (tamper the
  # budget below the real cardinality, expect FAIL, restore).
  BUDGET_FILE="$HERE/gate-cifre-corpus.budget"
  BUDGET_BKP=$(cat "$BUDGET_FILE")
  echo "max_tokens=10" > "$BUDGET_FILE"
  if bash "$0" "$TMP/baseline.md" >/dev/null 2>&1; then
    echo "$BUDGET_BKP" > "$BUDGET_FILE"
    echo "SELFTEST FAIL: corpus cardinality over budget NOT caught (A-SK61)"; rm -rf "$TMP"; exit 1
  fi
  echo "$BUDGET_BKP" > "$BUDGET_FILE"
  rm -rf "$TMP"
  echo "SELFTEST PASS: KG-83-3 smuggle + A-SK40 companions + A-SK55 committed-only + A-SK60 provenance (positive+bite) + A-SK62 every-token + A-SK63 manifest graces + A-SK65 env-cache ignored + A-SK61 budget + A-SK53-bis window all bite"
  exit 0
fi

perl - "$TARGET" "$HERE" "$MOUT" "$CACHE_ARG" "$NONCE_ARG" <<'PERL'
use strict; use warnings;
use File::Glob qw(bsd_glob);   # plain glob() splits patterns on spaces — the
                               # repo lives under "/Volumes/Extreme Pro/"
use Cwd qw(abs_path);
my ($target, $here, $mout, $cache_arg, $nonce_arg) = @ARGV;

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
# S-90.0: the measure90 campaign raws (m90* labels) + verdict90 (per-attempt,
# per-generation .out) in wp90-harness — the corpus budget MUST be raised in
# the same commit that lands these (A-SK61 deliberate-growth discipline)
push @sources, committed_glob("$mout/m90*"), committed_glob("$here/../wp90-harness/*.out");
# S-91.0: the repair90-estimators machine output (Concilio WP-92 team-misura
# P2: A-BB64..68 + A-BG57/60/61 recomputed on the committed m90 raws, zero
# new runs) + future wp91 judges — budget raised in the SAME commit (A-SK61)
push @sources, committed_glob("$here/../wp91-harness/*.out");
# S-85.0: the measure85 campaign raws (85*/m85* labels) + verdict85 + the
# wp85 evidence dir
push @sources, committed_glob("$mout/*85*.summary"), committed_glob("$mout/*85*.census"),
               committed_glob("$mout/*85*.log"),     committed_glob("$mout/m85*"),
               committed_glob("$mout/axum.85*"),     committed_glob("$here/../wp85-harness/*.out"),
               committed_glob("$here/../wp85-harness/evidence/*");
push @sources, committed_glob("$here/evidence/*");
die "gate-measure-cifre: EMPTY corpus (no committed sources found)\n" unless @sources;
# A-SK66 (Council WP-91, Klabnik — the gG hole made LATENT-proof): for
# every per-generation verdict family verdict<NN>.a<A>.g<G>.out only the
# MAX generation stays in the corpus — figures of FAILed/superseded
# judges must not legalize doc tokens.
my %gen_max;   # "dir|NN|A" -> max G (needed again for the citation tooth)
for my $s (@sources) {
  if ($s =~ m{^(.*/)verdict(\d+)\.a(\d+)\.g(\d+)\.out$}) {
    my ($k, $g) = ("$1|$2|$3", $4);
    $gen_max{$k} = $g if !exists $gen_max{$k} || $g > $gen_max{$k};
  }
}
@sources = grep {
  !(m{^(.*/)verdict(\d+)\.a(\d+)\.g(\d+)\.out$} && $4 < $gen_max{"$1|$2|$3"})
} @sources;
# citation map for the target scan (keyed WITHOUT dir: docs cite by name)
my %cite_max;
for my $k (keys %gen_max) {
  my (undef, $nn, $a) = split /\|/, $k;
  my $ck = "$nn|$a";
  $cite_max{$ck} = $gen_max{$k} if !exists $cite_max{$ck} || $gen_max{$k} > $cite_max{$ck};
}
my (%corpus, %corpus_count);
# A-SK65 (Council WP-91, Klabnik): corpus cache ONLY via argv + parent
# nonce — the env var GATE_CIFRE_CORPUS_CACHE is DEAD and IGNORED (it was
# an unauthenticated input: a poisoned inherited cache legalized any
# figure, KS-SK-91-3). Cache header = "<headrev> <nonce>": a pre-existing
# or foreign file cannot match the parent's fresh nonce.
my $cache = (defined $cache_arg && $cache_arg ne '' && defined $nonce_arg && $nonce_arg ne '')
            ? $cache_arg : '';
my $headrev = qx(git -C "$root" rev-parse HEAD); chomp $headrev;
my $cache_loaded = 0;
if ($cache && -s $cache) {
  open my $ch, '<', $cache or die "cannot read corpus cache\n";
  my $first = <$ch> // ''; chomp $first;
  if ($first eq "$headrev $nonce_arg") {
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
    # A-SK61 (Council WP-91, Klabnik — the A-SK56 forge was built from
    # here): digit-runs inside vmmap ADDRESS RANGES are not measures —
    # 5.262/22.399 tokens (23,5%) of the corpus existed ONLY as address
    # fragments and fed the X−Y closure. Strip every hex-range substring
    # (>=6 hex chars per side, at least one side carrying a letter) BEFORE
    # tokenizing. Declared residual: a pure-digit range on both sides is
    # indistinguishable from a numeric interval and survives.
    my $src = $l;
    $src =~ s/\b(?:(?=[0-9a-f]*[a-f])[0-9a-f]{6,}-[0-9a-f]{6,}|[0-9a-f]{6,}-(?=[0-9a-f]*[a-f])[0-9a-f]{6,})\b/ /g;
    while ($src =~ /(\d[\d.]*\d|\d)/g) {
      my $t = $1;
      $corpus{$t} = 1;
      (my $noint = $t) =~ s/\.0$//;  # 43463.0 -> 43463
      $corpus{$noint} = 1;
      # A-SK61: the nodot flatten is legal ONLY for thousands-grouped
      # tokens (1.234.567 -> 1234567); flattening a DECIMAL (3.14 -> 314)
      # fabricated tokens that exist in no machine output.
      if ($t =~ /^\d{1,3}(?:\.\d{3})+$/) {
        (my $nodot = $t) =~ s/\.//g;
        $corpus{$nodot} = 1;
      }
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
  print $ch "$headrev $nonce_arg\n";
  print $ch "C$_\n" for keys %corpus;
  print $ch "N$_\n" for keys %corpus_count;
  close $ch;
}
}  # !$cache_loaded

# A-SK61 (Council WP-91): corpus cardinality PRINTED and PINNED against a
# committed budget — a silent corpus explosion (a new glob swallowing a
# huge file) multiplies the forge surface; growth must be a conscious,
# committed act (raise the budget in the same commit that adds sources).
my $card = scalar(keys %corpus);
my $budget_file = "$here/gate-cifre-corpus.budget";
open my $bf, '<', $budget_file
  or die "gate-measure-cifre: corpus budget file MISSING ($budget_file) — A-SK61 fail-closed\n";
my $budget = <$bf> // ''; close $bf;
$budget =~ s/^\s+|\s+$//g; $budget =~ s/^max_tokens=//;
die "gate-measure-cifre: malformed budget '$budget' (A-SK61)\n" unless $budget =~ /^\d+$/;
print "corpus cardinality=$card budget=$budget (A-SK61)\n";
if ($card > $budget) {
  print "FAIL gate-measure-cifre: corpus cardinality $card EXCEEDS committed budget $budget (A-SK61) — raise the budget deliberately with the new sources\n";
  exit 1;
}

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
# A-SK63/A-SK64 (Council WP-91, Klabnik): the bytes-first flag and the ±
# graces come from the COMMITTED MANIFEST, keyed by repo-relative path —
# never from a filename regex (a copy named MEASURE85_zzforge inherited
# the M85 graces; MEASURE100 would have dropped out of bytes-first at
# WP-100, KS-SK-91-2). A grace-carrying row pins the doc's blob sha:
# editing the FROZEN doc revokes its graces until the manifest row is
# consciously updated. A target with NO manifest row (scratch copy,
# forged name, out-of-tree file) gets FAIL-CLOSED defaults: bytes-first
# ON, ZERO graces.
my $bytes_first = 1;
my @doc_bands;
my $legacy_frozen = 0;   # sha-pinned UNMODIFIED historical doc: keeps the
                         # pre-WP-91 derivata semantics (see below)
{
  my $rt = abs_path($target) // $target;
  my $rel = (index($rt, "$root/") == 0) ? substr($rt, length("$root/")) : '';
  if ($rel && open my $mfh, '<', "$here/gate-cifre-manifest.tsv") {
    while (my $row = <$mfh>) {
      chomp $row;
      next if $row =~ /^\s*(#|$)/;
      my ($mpath, $msha, $mjudge, $mbf, $mbands) = split /\t/, $row;
      next unless defined $mpath && $mpath eq $rel;
      $bytes_first = (defined $mbf && $mbf eq 'no') ? 0 : 1;
      my $sha_match = 0;
      if (defined $msha && $msha ne '-') {
        my $wsha = qx(git -C "$root" hash-object -- "$rt"); chomp $wsha;
        $sha_match = ($wsha eq $msha) ? 1 : 0;
        print "line 0: NOTE — $rel edited since its manifest pin: ± graces and legacy derivata REVOKED (A-SK63)\n"
          unless $sha_match;
      }
      # A-SK60 attuazione (Concilio WP-91): i doc STORICI CONGELATI il
      # cui blob coincide con lo sha pinnato conservano la semantica
      # derivata pre-WP-91 (evaluator X−Y su operandi in corpus, scan
      # limitato ai token "N B") — storia già giudicata, blob immutabile.
      # QUALSIASI edit rompe lo sha e fa cadere il doc sulle regole
      # nuove: il forge per-edit resta morso.
      $legacy_frozen = $sha_match;
      if (defined $mbands && $mbands ne '-' && $mbands ne '') {
        @doc_bands = split /,/, $mbands
          if !defined($msha) || $msha eq '-' || $sha_match;
      }
      last;
    }
    close $mfh;
  }
}
sub it_num { my $s = shift; $s =~ s/\.(?=\d{3}\b)//g; $s =~ s/,/./; return $s; }
my %SCALE = (k => 1024, m => 1024**2, g => 1024**3, t => 1024**4);
while (my $line = <$fh>) {
  $ln++;
  my $work = $line;   # verified pairs and granted bands get blanked here
  my %companion_ok;   # per-line: VERIFIED companion figures (the "X MiB"
                      # halves) — machine-recomputable, exempt in A-SK62;
                      # the BYTE halves stay corpus-bound (A-SK56).
  # A-SK66 citation tooth (Council WP-91): naming a NON-max generation
  # verdict file without declaring the supersession on the same line, or
  # naming an uncommitted generation, is never verdict-grade
  # (KS-SK-91-4/KS-AH-91-2).
  while ($line =~ /verdict(\d+)\.a(\d+)\.g(\d+)\.out/g) {
    my ($nn, $a, $g) = ($1, $2, $3);
    my $mx = $cite_max{"$nn|$a"};
    if (!defined $mx) {
      push @miss, "line $ln: cites verdict$nn.a$a.g$g.out but NO committed generation exists for that family (A-SK66/KS-SK-91-4): $line";
    } elsif ($g < $mx && $line !~ /supersed|judge-unrecoverable/i) {
      push @miss, "line $ln: cites NON-max generation g$g (max committed g$mx) without naming the supersession on the line (A-SK66/KS-SK-91-4): $line";
    } elsif ($g > $mx) {
      push @miss, "line $ln: cites generation g$g NOT committed for that family (max g$mx) (A-SK66): $line";
    }
  }
  if ($bytes_first) {
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
      $companion_ok{$fig} = 1;       # the verified MiB half only (A-SK62)
      my $blank = ' ' x length($whole);
      $work =~ s/\Q$whole\E/$blank/;
    }
    # (2) per-FIGURE band check (A-SK43/KS-SK-87-2, allowlist now from the
    # MANIFEST — A-SK63): only the bands GRANTED to this doc survive; the
    # global name-keyed list is gone ("3.605.572B±5%" lives only in the
    # M86/87/88 rows as a NAMED-DEVIATION historical target, KB-90-2).
    while ($work =~ /(\d[\d.,]*\s*±\s*\d[\d.,]*\s*[KMGTkmgt]i?[Bb])\b/g) {
      my $band = $1;
      (my $norm = $band) =~ s/\s+//g;
      push @miss, "line $ln: ± band '$band' not granted by the manifest (A-SK63/KS-SK-87-2): $line"
        unless grep { $_ eq $norm } @doc_bands;
    }
    for my $b (@doc_bands) {
      my $pat = join '\s*', map { quotemeta } split //, $b;
      $work =~ s/$pat/' ' x length($&)/ge;
    }
    # (2b) A-SK53-bis window tooth (unchanged force, manifest-driven
    # grants): strip every GRANTED band (normalized, whitespace-free)
    # from the line; ANY surviving '±' in a bytes-first doc is unlisted.
    {
      (my $flat = $line) =~ s/\s+//g;
      for my $band (sort { length($b) <=> length($a) } @doc_bands) {
        $flat =~ s/\Q$band\E(?!\d)//g;
      }
      if ($flat =~ /±/) {
        push @miss, "line $ln: '±' band not granted by the manifest (A-SK63/A-SK53-bis/KS-SK-87-2): $line";
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
    if ($legacy_frozen) {
      # FROZEN sha-pinned historical doc (manifest): pre-WP-91 semantics
      # verbatim — X−Y evaluator over corpus-present operands, scan bound
      # to "N B" tokens only. Granted ONLY while the blob matches its
      # pin; any edit falls through to the A-SK60/A-SK62 rules below.
      if ($bytes_first) {
        my %eval_ok_l;
        if ($line =~ /\[derivata:([^\]]*)/) {
          my $expr = $1;
          while ($expr =~ /(\d(?:[\d.,]*\d)?)\s*(?:\xE2\x88\x92|-)\s*(\d(?:[\d.,]*\d)?)/g) {
            my ($xa, $xb) = (it_num($1), it_num($2));
            next unless ($corpus{$xa} || $ALLOW{$xa}) && ($corpus{$xb} || $ALLOW{$xb});
            $eval_ok_l{$xa - $xb} = 1;
          }
        }
        my $probe2 = $line;
        $probe2 =~ s/\b(?=[0-9a-f]*[a-f])[0-9a-f]{7,}\b//gi;
        while ($probe2 =~ /(?<![\dA-Za-z,.±])(\d{1,3}(?:\.\d{3})+|\d{3,})\s*B(?![A-Za-z0-9])/g) {
          my $raw = $1;
          (my $norm = $raw) =~ s/\.(?=\d{3}\b)//g;
          next if $ALLOW{$norm} || $corpus{$norm} || $corpus_count{$norm};
          if ($eval_ok_l{$norm} || $derived_ok{$norm}) { $derived_ok{$norm} = 1; next; }
          push @miss, "line $ln: byte token '$raw B' on a [derivata] line NOT in corpus and NOT machine-verified from corpus operands — the tag is figure-scope (A-SK56/KS-SK-90-2): $line";
        }
      }
      next;
    }
    # A-SK60 (Council WP-91, Klabnik — the free X−Y evaluator was REFUTED
    # LIVE): with existential operands over a 22k-token corpus (23,5% of
    # which were vmmap address fragments) the closure of differences
    # reached ~11,5% of any plausible neighborhood — a fabricated b_base
    # PASSED. The evaluator is ABOLISHED. A derived value legalizes a
    # token ONLY by PROVENANCE:
    #   [derivata: prov <N>@<repo/path>:<line> − <M>@<repo/path>:<line>]
    # Each operand is re-read from the COMMITTED blob at HEAD at exactly
    # that line, and the gate PRINTS the resolution. Label-only tags
    # ([derivata: companion /1048576], [derivata: Δ/4], …) legalize
    # NOTHING by themselves — companions are verified in step (1).
    my %eval_ok;
    my $tagtext = ($line =~ /\[derivata:([^\]]*)/) ? $1 : '';
    if ($tagtext =~ /^\s*prov\s/) {
      my (@ops, $bad);
      while ($tagtext =~ /(\d(?:[\d.,]*\d)?)\@([^\s:\]]+):(\d+)/g) {
        my ($tok, $rp, $rl) = ($1, $2, $3);
        my $ntok = it_num($tok);
        if (!$headset{$rp}) {
          push @miss, "line $ln: derivata operand $tok \@ $rp:$rl — path NOT committed at HEAD (A-SK60): $line";
          $bad = 1; last;
        }
        my @blob = qx(git -C "$root" show "HEAD:$rp");
        my $srcline = $blob[$rl-1] // ''; $srcline =~ s/\0//g;
        my $found = 0;
        while ($srcline =~ /(\d[\d.]*\d|\d)/g) {
          my $c = $1; (my $nd = $c) =~ s/\.(?=\d{3}\b)//g;
          if ($c eq $ntok || $nd eq $ntok) { $found = 1; last; }
        }
        if (!$found) {
          push @miss, "line $ln: derivata operand $ntok NOT found at $rp:$rl (A-SK60): $line";
          $bad = 1; last;
        }
        print "line $ln: derivata operand $ntok <= $rp:$rl resolved at HEAD (A-SK60)\n";
        push @ops, $ntok;
      }
      if (!$bad && @ops == 2 && $tagtext =~ /\xE2\x88\x92|-/) {
        $eval_ok{$ops[0] - $ops[1]} = 1;
        printf "line %d: derivata value %s = %s − %s (A-SK60 provenance-verified)\n",
               $ln, $ops[0] - $ops[1], $ops[0], $ops[1];
      }
    }
    # A-SK62 (Council WP-91): on a [derivata] line EVERY token >=3 digits
    # is judged — 2σ brackets, se, counts included (the old tooth bound
    # only "N B"-shaped tokens: a fabricated 2σ lower bound inside
    # brackets PASSED). The scan runs on the RAW line — a verified
    # companion pair does NOT shelter its byte half (A-SK56: the pair
    # self-verifies on any fabricated N). Exempt ONLY the
    # machine-recomputable: the verified companion FIGURES
    # (%companion_ok), provenance-resolved values, and their wrapped
    # repeats. The tag bracket content itself is judged by A-SK60 above,
    # not harvested here.
    my $probe2 = $line;
    $probe2 =~ s/\[derivata:[^\]]*\]?/ /g;
    $probe2 =~ s/\b(?=[0-9a-f]*[a-f])[0-9a-f]{7,}\b//gi;
    $probe2 =~ s/[A-Za-z_][A-Za-z0-9_-]*[0-9][A-Za-z0-9_-]*//g;
    while ($probe2 =~ /(?<![\dA-Za-z,.±])(\d{1,3}(?:\.\d{3})+(?:,\d+)?|\d+,\d+|\d{3,})(?![\dA-Za-z])/g) {
      my $raw = $1;
      my $norm = $raw;
      $norm =~ s/\.(?=\d{3}\b)//g;
      $norm =~ s/,/./;
      next if $companion_ok{$norm};
      next if $ALLOW{$norm} || $corpus{$norm} || $corpus_count{$norm};
      (my $noint = $norm) =~ s/\.0$//;
      next if $corpus{$noint};
      if ($eval_ok{$norm} || $derived_ok{$norm}) { $derived_ok{$norm} = 1; next; }
      push @miss, "line $ln: token '$raw' on a [derivata] line NOT in corpus and NOT provenance-verified (A-SK56/A-SK60/A-SK62): $line";
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
