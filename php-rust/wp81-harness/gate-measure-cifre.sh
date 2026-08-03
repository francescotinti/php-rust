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
# A-SK-67 (Council WP-92, Klabnik FORGE LANDED — KS-SK-92-1): the AUTHORITY
# INPUTS (manifest, budget, and this judge itself) are read from HEAD, never
# from the working tree — an UNCOMMITTED manifest row used to grant legacy
# semantics to a 10-second-old doc. In verdict/all mode a working-tree
# authority file that differs from its HEAD blob is an outright FAIL; the
# PASS line prints judge_sha= manifest_sha= budget_sha= so a patched judge
# can never produce an anonymous PASS.
#
# T2 resolution (team-cifre WP-92) — TWO EXPLICIT MODES:
#   advisory (default, single target): the TARGET is read from the working
#     tree (a doc in scrittura non è a HEAD per definizione), authorities
#     STILL from HEAD; the result is ADVISORY-PASS/FAIL and NEVER
#     verdict-grade.
#   --verdict <target> / --all: target content read from HEAD too; PASS
#     prints the authority shas. --all runs in ONE perl process (corpus
#     built once).
#
# A-SK-70 (Council WP-92, Klabnik FORGE LANDED — KS-SK-92-4): the corpus
# cache is ABOLISHED. --cache/--nonce moved the poison from env to argv —
# equally unauthenticated (the caller chooses the nonce). Any invocation
# carrying them is REFUSED. The --all cost is paid by the single-process
# design, not by a cache.
#
# SCOPE (declared): tokens of >=3 digits (after normalizing the Italian
# formatting: thousands '.' stripped, decimal ',' -> '.'). 1-2 digit tokens
# are protocol/threshold small constants judged by eye — binding them would
# drown the gate in noise (every R=3, W=1, §N). Hex identities (git revs,
# sha fingerprints containing [a-f]) are excluded here: their truth is
# enforced by the feature-matrix/driver identity gates, not by this one.
#
# Self-test: --selftest plants fabricated figures and forged authorities in
# copies and expects FAIL (a gate that cannot bite is vacuous). The four
# Klabnik WP-92 forges are repeated as permanent teeth.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(git -C "$HERE" rev-parse --show-toplevel)" || { echo "FAIL gate-measure-cifre: not in a git repo (A-SK55/A-SK-67 need HEAD)"; exit 1; }
MOUT="$HERE/../wp78-harness/measure-out"
MANIFEST_REL="php-rust/wp81-harness/gate-cifre-manifest.tsv"
BUDGET_REL="php-rust/wp81-harness/gate-cifre-corpus.budget"
JUDGE_REL="php-rust/wp81-harness/gate-measure-cifre.sh"

# A-SK-70/KS-SK-92-4: cache/nonce are DEAD — refuse, never parse.
for a in "$@"; do
  case "$a" in
    --cache|--nonce)
      echo "FAIL gate-measure-cifre: --cache/--nonce are ABOLISHED — an invoker-supplied cache/nonce is an unauthenticated authority (A-SK-70/KS-SK-92-4)"
      exit 1;;
  esac
done

MODE=advisory
if [ "${1:-}" = "--verdict" ]; then MODE=verdict; shift; fi
if [ "${1:-}" = "--all" ]; then MODE=all; fi

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
  # T9 — A-SK65 kept: a POISONED cache inherited via env must be IGNORED
  # (the env var is dead; A-SK-70 killed the argv path too — see T15).
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
  # T11 v2 — A-SK-67/KS-SK-92-1: a TAMPERED working-tree budget must FAIL
  # in verdict mode (the authority is HEAD; working!=HEAD is itself the
  # crime — before WP-92 the tamper changed the judgment silently).
  # NOTE (declared): the cardinality>budget branch below is no longer
  # unit-testable by file tamper (budget comes from HEAD); its live bite
  # is on record — battery-90pre attempt a2 FAILED on it (83f0cfc).
  BUDGET_FILE="$HERE/gate-cifre-corpus.budget"
  BUDGET_BKP=$(cat "$BUDGET_FILE")
  echo "max_tokens=10" > "$BUDGET_FILE"
  if bash "$0" --verdict "$HERE/../wp89-harness/MEASURE89_RESULTS.md" >/dev/null 2>&1; then
    echo "$BUDGET_BKP" > "$BUDGET_FILE"
    echo "SELFTEST FAIL: tampered working-tree budget NOT refused in verdict mode (A-SK-67/KS-SK-92-1)"; rm -rf "$TMP"; exit 1
  fi
  echo "$BUDGET_BKP" > "$BUDGET_FILE"
  # T13 — WP-92 forge (a) REPEATED (Klabnik Q1, KS-SK-92-1): a fresh doc
  # plus an UNCOMMITTED manifest row (sha-pinned to the doc, graces
  # granted) must fail BOTH ways: (i) verdict mode refuses the tampered
  # manifest outright; (ii) advisory mode never sees the row (manifest
  # from HEAD) so the doc gets fail-closed defaults and its derivata
  # forge dies.
  T13DOC="$HERE/zzforge-t13.md"
  {
    echo "# forged doc, 10 seconds old"
    echo "b_base rivisto = 19.600.000 B = 18,69 MiB per worker [derivata: 23.000.000 − 3.400.000]"
  } > "$T13DOC"
  T13SHA=$(git -C "$ROOT" hash-object -- "$T13DOC")
  MANIFEST_FILE="$HERE/gate-cifre-manifest.tsv"
  MAN_BKP=$(cat "$MANIFEST_FILE")
  printf 'php-rust/wp81-harness/zzforge-t13.md\t%s\tyes\tyes\t-\n' "$T13SHA" >> "$MANIFEST_FILE"
  if bash "$0" --verdict "$T13DOC" >/dev/null 2>&1; then
    printf '%s\n' "$MAN_BKP" > "$MANIFEST_FILE"; rm -f "$T13DOC"; rm -rf "$TMP"
    echo "SELFTEST FAIL: verdict mode accepted a TAMPERED working-tree manifest (WP-92 forge a / A-SK-67)"; exit 1
  fi
  if bash "$0" "$T13DOC" >/dev/null 2>&1; then
    printf '%s\n' "$MAN_BKP" > "$MANIFEST_FILE"; rm -f "$T13DOC"; rm -rf "$TMP"
    echo "SELFTEST FAIL: an UNCOMMITTED manifest row was honored (WP-92 forge a / A-SK-67)"; exit 1
  fi
  printf '%s\n' "$MAN_BKP" > "$MANIFEST_FILE"
  rm -f "$T13DOC"
  # T15 — WP-92 forge (c) REPEATED (Klabnik Q3, KS-SK-92-4): an
  # invoker-supplied cache/nonce must be REFUSED, never parsed.
  if bash "$0" --cache "$TMP/x" --nonce deadbeef "$TMP/baseline.md" >/dev/null 2>&1; then
    echo "SELFTEST FAIL: invoker-supplied --cache/--nonce NOT refused (WP-92 forge c / A-SK-70/KS-SK-92-4)"; rm -rf "$TMP"; exit 1
  fi
  rm -rf "$TMP"
  echo "SELFTEST PASS: KG-83-3 smuggle + A-SK40 companions + A-SK55 committed-only + A-SK60 provenance (positive+bite) + A-SK62 every-token + A-SK63 manifest graces + A-SK65 env-cache ignored + A-SK53-bis window + A-SK-67 HEAD-authorities (budget tamper, forge-a manifest row) + A-SK-70 cache abolished (forge-c) all bite"
  exit 0
fi

# ---- target list ------------------------------------------------------------
# --all: judged targets come from the HEAD manifest (single perl process,
# A-SK-70); everything else (SKIP prints, bidirectional check) also lives in
# the perl below so the corpus is built exactly once.
if [ "$MODE" = all ]; then
  TARGETS="--all"
else
  TARGETS="${1:-$HERE/MEASURE81_RESULTS.md}"
fi

perl - "$ROOT" "$HERE" "$MOUT" "$MODE" "$TARGETS" <<'PERL'
use strict; use warnings;
use Cwd qw(abs_path);
my ($root, $here, $mout, $mode, $target_arg) = @ARGV;

my $MANIFEST_REL = "php-rust/wp81-harness/gate-cifre-manifest.tsv";
my $BUDGET_REL   = "php-rust/wp81-harness/gate-cifre-corpus.budget";
my $JUDGE_REL    = "php-rust/wp81-harness/gate-measure-cifre.sh";

my $headrev = qx(git -C "$root" rev-parse HEAD); chomp $headrev;

# ---- A-SK-67: authority inputs from HEAD, working tree verified ------------
sub head_blob_sha { # repo-relative path -> blob sha at HEAD ('' if absent)
  my ($rel) = @_;
  my $s = qx(git -C "$root" rev-parse -q --verify "HEAD:$rel" 2>/dev/null);
  chomp $s; return $s;
}
sub work_blob_sha { # absolute path -> git hash-object of working file
  my ($abs) = @_;
  return '' unless -f $abs;
  my $s = qx(git -C "$root" hash-object -- "$abs"); chomp $s; return $s;
}
sub head_content { # repo-relative path -> list of lines from HEAD
  my ($rel) = @_;
  open my $fh, '-|', 'git', '-C', $root, 'show', "HEAD:$rel" or return ();
  my @l = <$fh>; close $fh; return @l;
}
my $authority_dirty = 0;
my %ASHA;
for my $a ([$JUDGE_REL, 'judge'], [$MANIFEST_REL, 'manifest'], [$BUDGET_REL, 'budget']) {
  my ($rel, $name) = @$a;
  my $h = head_blob_sha($rel);
  my $w = work_blob_sha("$root/$rel");
  if (!$h) { print "FAIL gate-measure-cifre: authority $name ($rel) NOT committed at HEAD (A-SK-67)\n"; exit 1 }
  $ASHA{$name} = substr($h, 0, 16);
  if ($w ne $h) {
    $authority_dirty = 1;
    if ($mode eq 'advisory') {
      print "NOTE: authority $name working tree != HEAD ($w vs $h) — result is ADVISORY anyway (A-SK-67)\n";
    } else {
      print "FAIL gate-measure-cifre: authority $name working tree != HEAD blob (A-SK-67/KS-SK-92-1) — commit the authority before a verdict-grade run\n";
      exit 1;
    }
  }
}

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
# to legalize any figure (KS-SK-90-1).
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
# A-SK-70: NO cache of any provenance — the corpus is built exactly once
# per process, and --all shares this one process (T3 team-cifre: the
# "one perl for --all" branch, not the "parent-created cache" branch).
my %seen_src;
for my $f (@sources) {
  next if $f =~ m{/\._};             # AppleDouble
  next if $seen_src{$f}++;
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

# A-SK61 (Council WP-91): corpus cardinality PRINTED and PINNED against the
# COMMITTED budget (A-SK-67: read from HEAD, never the working tree) — a
# silent corpus explosion multiplies the forge surface; growth must be a
# conscious, committed act (raise the budget in the same commit that adds
# sources). The over-budget branch bit LIVE at battery-90pre attempt a2.
my $card = scalar(keys %corpus);
my ($budget_line) = head_content($BUDGET_REL);
my $budget = defined $budget_line ? $budget_line : '';
$budget =~ s/^\s+|\s+$//g; $budget =~ s/^max_tokens=//;
die "gate-measure-cifre: malformed HEAD budget '$budget' (A-SK61/A-SK-67)\n" unless $budget =~ /^\d+$/;
print "corpus cardinality=$card budget=$budget (A-SK61, budget from HEAD)\n";
if ($card > $budget) {
  print "FAIL gate-measure-cifre: corpus cardinality $card EXCEEDS committed budget $budget (A-SK61) — raise the budget deliberately with the new sources\n";
  exit 1;
}

# ---- manifest from HEAD (A-SK-67) ------------------------------------------
my @man_rows;
for my $row (head_content($MANIFEST_REL)) {
  chomp $row;
  next if $row =~ /^\s*(#|$)/;
  my ($mpath, $msha, $mjudge, $mbf, $mbands) = split /\t/, $row;
  push @man_rows, [$mpath, $msha, $mjudge, $mbf, $mbands];
}

# ---- resolve target list ---------------------------------------------------
my @targets;   # [abs-or-rel display name, repo-rel path or '', source: head|work]
my $all_rc = 0;
if ($target_arg eq '--all') {
  for my $r (@man_rows) {
    my ($mpath, $msha, $mjudge, $mbf, $mbands) = @$r;
    if (!$headset{$mpath}) {
      print "FAIL gate-measure-cifre --all: manifest doc MISSING from HEAD: $mpath (A-SK64/A-SK-67)\n"; $all_rc = 1; next;
    }
    if ($mjudge eq 'no') {
      print "SKIP $mpath (declared judge=no in manifest — A-SK64)\n"; next;
    }
    push @targets, [$mpath, $mpath, 'head'];
  }
  # reverse direction: every MEASURE*_RESULTS.md at HEAD and in the working
  # tree must be manifested (a doc can be born in the working tree)
  my %manset = map { $_->[0] => 1 } @man_rows;
  for my $f (grep { m{^php-rust/wp\d+-harness/MEASURE\d+_RESULTS\.md$} } @headtree) {
    next if $manset{$f};
    print "FAIL gate-measure-cifre --all: $f has NO manifest entry (A-SK64 bidirectional)\n"; $all_rc = 1;
  }
  for my $f (glob("$here/../wp*-harness/MEASURE*_RESULTS.md")) {
    next unless -f $f;
    my $rel = abs_path($f); $rel = substr($rel, length("$root/")) if index($rel, "$root/") == 0;
    next if $manset{$rel};
    print "FAIL gate-measure-cifre --all: $rel has NO manifest entry (A-SK64 bidirectional, working tree)\n"; $all_rc = 1;
  }
} else {
  my $rt = abs_path($target_arg) // $target_arg;
  my $rel = (index($rt, "$root/") == 0) ? substr($rt, length("$root/")) : '';
  if ($mode eq 'verdict') {
    if (!$rel || !$headset{$rel}) {
      print "FAIL gate-measure-cifre: verdict mode requires a target committed at HEAD (A-SK-67): $target_arg\n";
      exit 1;
    }
    push @targets, [$target_arg, $rel, 'head'];
  } else {
    push @targets, [$target_arg, $rel, 'work'];
  }
}

# ---- per-target scan -------------------------------------------------------
sub it_num { my $s = shift; $s =~ s/\.(?=\d{3}\b)//g; $s =~ s/,/./; return $s; }
my %SCALE = (k => 1024, m => 1024**2, g => 1024**3, t => 1024**4);

my $grand_rc = $all_rc;
for my $t (@targets) {
  my ($disp, $rel, $src) = @$t;
  my @lines;
  if ($src eq 'head') {
    @lines = head_content($rel);
    if ($rel && -f "$root/$rel") {
      my $w = work_blob_sha("$root/$rel");
      my $h = head_blob_sha($rel);
      print "NOTE: target $rel working tree differs from HEAD — this verdict judges the HEAD blob (A-SK-67)\n"
        if $w ne $h;
    }
  } else {
    open my $fh, '<', $disp or do { print "FAIL: cannot open $disp\n"; $grand_rc = 1; next };
    @lines = <$fh>; close $fh;
  }

  # manifest row lookup (from HEAD rows)
  my $bytes_first = 1;
  my @doc_bands;
  my $legacy_frozen = 0;   # sha-pinned UNMODIFIED historical doc: keeps the
                           # pre-WP-91 derivata semantics (see below)
  if ($rel) {
    for my $r (@man_rows) {
      my ($mpath, $msha, $mjudge, $mbf, $mbands) = @$r;
      next unless defined $mpath && $mpath eq $rel;
      $bytes_first = (defined $mbf && $mbf eq 'no') ? 0 : 1;
      my $sha_match = 0;
      if (defined $msha && $msha ne '-') {
        # the pin is judged against the CONTENT BEING JUDGED: HEAD blob in
        # verdict/all mode, working blob in advisory mode
        my $tsha = ($src eq 'head') ? head_blob_sha($rel) : work_blob_sha("$root/$rel");
        $sha_match = ($tsha eq $msha) ? 1 : 0;
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
  }

  my ($ln, @miss) = (0);
  my %derived_ok;   # A-SK56: byte tokens legalized by verified arithmetic
  # A-DL26 (Council WP-85, KL-85-2) + A-SK40 (Council WP-86): from MEASURE84
  # on, every MEMORY figure must print BYTES FIRST ("N B = X MiB") — and the
  # companion is VERIFIED: bytes/scale must round to the displayed figure.
  # Units are [KMGT]i?B case-INSENSITIVE; the check is per-FIGURE. Manifest
  # rows carry the bytes-first flag and the ± graces (A-SK63/64).
  for my $line (@lines) {
    $ln++;
    my $work = $line;   # verified pairs and granted bands get blanked here
    my %companion_ok;
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
        # "MB" under SI reading lies to the outside reader.
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
      # (2) per-FIGURE band check (A-SK43/KS-SK-87-2, allowlist from the
      # MANIFEST — A-SK63): only the bands GRANTED to this doc survive.
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
      # grants): ANY surviving '±' in a bytes-first doc is unlisted.
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
      # LIVE): a derived value legalizes a token ONLY by PROVENANCE:
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
          my @blob = head_content($rp);
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
      # is judged — 2σ brackets, se, counts included. The scan runs on the
      # RAW line; exempt ONLY the machine-recomputable (verified companion
      # figures, provenance-resolved values, wrapped repeats).
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

  if (@miss) {
    print "FAIL gate-measure-cifre (KG-83-3): ".scalar(@miss)." unmatched figure(s) in $disp:\n";
    print "  $_" for @miss;
    $grand_rc = 1;
    next;
  }
  if ($mode eq 'advisory') {
    print "ADVISORY-PASS gate-measure-cifre (KG-83-3, NEVER verdict-grade — T2/A-SK-67): every bound figure in $disp matches committed machine output (or carries [derivata]/named-constant)\n";
  } else {
    print "PASS gate-measure-cifre (KG-83-3): every bound figure in $disp matches committed machine output (or carries [derivata]/named-constant) [judge_sha=$ASHA{judge} manifest_sha=$ASHA{manifest} budget_sha=$ASHA{budget} head=".substr($headrev,0,12)."]\n";
  }
}
if ($target_arg eq '--all' && $grand_rc == 0) {
  print "PASS gate-measure-cifre --all (A-SK64/A-SK-67): manifest perimeter, bidirectional, authorities from HEAD [judge_sha=$ASHA{judge} manifest_sha=$ASHA{manifest} budget_sha=$ASHA{budget} head=".substr($headrev,0,12)."]\n";
}
exit $grand_rc;
PERL
