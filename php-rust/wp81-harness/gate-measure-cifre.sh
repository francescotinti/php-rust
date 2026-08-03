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
# v3 (Council WP-93, Klabnik — SIX new forges 6/6 through v2, three capital
# refutations): A-SK-78 self-tether of the RUNNING judge (F6 judge-copy);
# A-SK-79 grade in the exit code (ADVISORY-PASS=64); A-SK-74 rev-parse door
# abolished (F1 decimal ancestor prefixes → committed rev= rows); A-SK-75
# ALLOW entries are authorities with HEAD-verified provenance (F2 — 2.8/
# 46.25 revoked); A-SK-76 glued digit-runs refused, never truncated (F3);
# A-SK-77 budget history out of the corpus (F4); A-SK-80 perimeter by
# complement (every committed .md under php-rust/); A-SK-81 labeled
# same-key prov operands (F5). The six forges are permanent teeth T17-T22
# (KS-SK-93-1..4).
#
# Self-test: --selftest plants fabricated figures and forged authorities in
# copies and expects FAIL at the EXACT rc (a gate that cannot bite is
# vacuous). The four Klabnik WP-92 forges and the six WP-93 forges are
# repeated as permanent teeth.
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

# A-SK-78 (Council WP-93, Klabnik forge F6 LANDED — KS-SK-93-1): SELF-TETHER
# of the RUNNING judge. A patched COPY of this script printed a verdict-grade
# PASS carrying the judge_sha of the PRISTINE HEAD blob — the sha signed a
# namesake, not the code that ran. hash-object($0) must equal the HEAD blob
# of JUDGE_REL before any verdict-grade output; in advisory mode the
# mismatch is a NOTE (an advisory result is never verdict-grade anyway).
# ($0 can be relative: hash the ABSOLUTE self path — `git -C` would resolve
# a relative $0 against ROOT, yielding an empty sha and a vacuous tether)
SELF_ABS="$HERE/$(basename "$0")"
RUN_SHA=$(git -C "$ROOT" hash-object -- "$SELF_ABS" 2>/dev/null)
JHEAD_SHA=$(git -C "$ROOT" rev-parse -q --verify "HEAD:$JUDGE_REL" 2>/dev/null)
if [ -z "$JHEAD_SHA" ]; then
  echo "FAIL gate-measure-cifre: judge $JUDGE_REL NOT committed at HEAD (A-SK-67/A-SK-78)"; exit 1
fi
if [ "$RUN_SHA" != "$JHEAD_SHA" ]; then
  if [ "$MODE" = advisory ]; then
    echo "NOTE: RUNNING judge (hash-object \$0) != HEAD blob of $JUDGE_REL — result is ADVISORY anyway (A-SK-78)"
  else
    echo "REFUSE gate-measure-cifre: RUNNING judge (hash-object \$0 = $(printf '%.16s' "$RUN_SHA")) != HEAD blob $(printf '%.16s' "$JHEAD_SHA") of $JUDGE_REL (A-SK-78/KS-SK-93-1) — the judge_sha signs the code that RUNS, never a namesake"
    exit 1
  fi
fi

if [ "${1:-}" = "--selftest" ]; then
  TMP=$(mktemp -d)
  M89DOC="$HERE/../wp89-harness/MEASURE89_RESULTS.md"
  M85DOC="$HERE/../wp85-harness/MEASURE85_RESULTS.md"
  # A-SK-79 (Council WP-93, Klabnik Q4): the GRADE lives in the EXIT CODE —
  # ADVISORY-PASS exits 64, FAIL exits 1, verdict PASS exits 0. Every tooth
  # asserts the EXACT rc: a plain `if bash` cannot tell a FAIL (1) from an
  # advisory pass (64), and a tooth "satisfied" by rc 64 is vacuous.
  rc_of() { bash "$0" "$@" >/dev/null 2>&1; echo $?; }
  # T0 — baseline: an INTACT copy of MEASURE89 must ADVISORY-PASS (rc 64)
  # from scratch space (bytes-first fail-closed, zero graces): otherwise
  # every FAIL tooth below is vacuous (a gate that fails everything bites
  # nothing).
  cp "$M89DOC" "$TMP/baseline.md"
  if [ "$(rc_of "$TMP/baseline.md")" != 64 ]; then
    echo "SELFTEST FAIL: intact MEASURE89 copy rc != 64 (ADVISORY-PASS must exit 64 — A-SK-79)"
    rm -rf "$TMP"; exit 1
  fi
  # T1 — KG-83-3 smuggle
  cp "$TMP/baseline.md" "$TMP/t1.md"
  echo "smuggled figure: a_calls was 123457 on a good day" >> "$TMP/t1.md"
  if [ "$(rc_of "$TMP/t1.md")" != 1 ]; then
    echo "SELFTEST FAIL: smuggled 123457 was NOT caught (KG-83-3)"; rm -rf "$TMP"; exit 1
  fi
  # T2 — A-SK40: naked lowercase unit figure without companion
  cp "$TMP/baseline.md" "$TMP/t2.md"
  echo "note: the cache costs 5,00 mib steady, honest." >> "$TMP/t2.md"
  if [ "$(rc_of "$TMP/t2.md")" != 1 ]; then
    echo "SELFTEST FAIL: naked lowercase 'mib' figure NOT caught (A-SK40)"; rm -rf "$TMP"; exit 1
  fi
  # T3 — A-SK40: companion that does not verify
  cp "$TMP/baseline.md" "$TMP/t3.md"
  echo "note: 1.048.576 B = 2,00 MiB [derivata: selftest]" >> "$TMP/t3.md"
  if [ "$(rc_of "$TMP/t3.md")" != 1 ]; then
    echo "SELFTEST FAIL: mismatching bytes companion NOT caught (A-SK40)"; rm -rf "$TMP"; exit 1
  fi
  # T4 — A-SK55: uncommitted forge in measure-out must not legalize
  FORGE="$MOUT/m88.zzforge-selftest.tmp"
  echo "committed=987654321" > "$FORGE"
  cp "$TMP/baseline.md" "$TMP/t4.md"
  echo "smuggled: campaign committed was 987654321 flat" >> "$TMP/t4.md"
  if [ "$(rc_of "$TMP/t4.md")" != 1 ]; then
    rm -f "$FORGE"; rm -rf "$TMP"
    echo "SELFTEST FAIL: working-tree forge in measure-out legalized a figure (A-SK55/KS-SK-90-1)"; exit 1
  fi
  rm -f "$FORGE"
  # T5 — A-SK60: the EXACT Klabnik live forge (WP-91 Q1b) — fabricated
  # b_base with a free X−Y over corpus-present vmmap fragments. The
  # evaluator is abolished: this must FAIL.
  cp "$TMP/baseline.md" "$TMP/t5.md"
  echo "b_base rivisto = 19.600.000 B = 18,69 MiB per worker [derivata: 23.000.000 − 3.400.000]" >> "$TMP/t5.md"
  if [ "$(rc_of "$TMP/t5.md")" != 1 ]; then
    echo "SELFTEST FAIL: free X−Y [derivata] forge NOT caught (A-SK60/KS-SK-91-1)"; rm -rf "$TMP"; exit 1
  fi
  # T6 — A-SK62: fabricated 2σ lower bound inside brackets on a
  # [derivata] line (the second Klabnik bite: only "N B"-shaped tokens
  # were judged before).
  cp "$TMP/baseline.md" "$TMP/t6.md"
  echo "nota: 2σ = [11.111.111, 20.745.049] B [derivata: companion /1048576]" >> "$TMP/t6.md"
  if [ "$(rc_of "$TMP/t6.md")" != 1 ]; then
    echo "SELFTEST FAIL: fabricated bracket token on [derivata] line NOT caught (A-SK62/KS-SK-91-1)"; rm -rf "$TMP"; exit 1
  fi
  # T6b — A-SK56 historic tooth kept: fabricated "N B = X MiB" pair
  cp "$TMP/baseline.md" "$TMP/t6b.md"
  echo "W=9 999.948.288 B = 953,56 MiB [derivata: companion /1048576]" >> "$TMP/t6b.md"
  if [ "$(rc_of "$TMP/t6b.md")" != 1 ]; then
    echo "SELFTEST FAIL: fabricated byte token on [derivata] row NOT caught (A-SK56/KS-SK-90-2)"; rm -rf "$TMP"; exit 1
  fi
  # T7 — A-SK53-bis: unitless ± band
  cp "$TMP/baseline.md" "$TMP/t7.md"
  echo "tolleranza dichiarata ±7% sul floor, onesta" >> "$TMP/t7.md"
  if [ "$(rc_of "$TMP/t7.md")" != 1 ]; then
    echo "SELFTEST FAIL: unitless ± band NOT caught (A-SK53-bis/KS-SK-87-2)"; rm -rf "$TMP"; exit 1
  fi
  # T8 — A-SK63: a forged NAME must inherit NOTHING — a copy of
  # MEASURE85 (whose committed row grants '±5%' and '232±1') under a
  # forged name gets fail-closed defaults and must FAIL on its bands.
  cp "$M85DOC" "$TMP/MEASURE85_zzforge_RESULTS.md"
  if [ "$(rc_of "$TMP/MEASURE85_zzforge_RESULTS.md")" != 1 ]; then
    echo "SELFTEST FAIL: forged-name copy inherited the M85 graces (A-SK63/KS-SK-91-2)"; rm -rf "$TMP"; exit 1
  fi
  # T9 — A-SK65 kept: a POISONED cache inherited via env must be IGNORED
  # (the env var is dead; A-SK-70 killed the argv path too — see T15).
  HEADREV=$(git -C "$HERE" rev-parse HEAD)
  POISON="$TMP/poison.cache"
  { echo "$HEADREV"; echo "C777444111"; } > "$POISON"
  cp "$TMP/baseline.md" "$TMP/t9.md"
  echo "smuggled: threshold hit 777444111 flat" >> "$TMP/t9.md"
  if [ "$(GATE_CIFRE_CORPUS_CACHE="$POISON" bash "$0" "$TMP/t9.md" >/dev/null 2>&1; echo $?)" != 1 ]; then
    echo "SELFTEST FAIL: poisoned env cache legalized a figure (A-SK65/KS-SK-91-3)"; rm -rf "$TMP"; exit 1
  fi
  # T10 — A-SK60 POSITIVE control + bite: a provenance-resolved derivata
  # must legalize its exact value (positive: the resolver is not dead
  # code) and must NOT legalize a different value (negative).
  G3REL="php-rust/wp89-harness/verdict89.a1.g3.out"
  cp "$TMP/baseline.md" "$TMP/t10.md"
  echo "delta se: 734.670 B [derivata: prov 1.319.393@$G3REL:115 − 584.723@$G3REL:63]" >> "$TMP/t10.md"
  if ! bash "$0" "$TMP/t10.md" 2>&1 | grep -q "(A-SK60 provenance-verified"; then
    echo "SELFTEST FAIL: provenance resolver never fired on a legal derivata (A-SK60 positive control)"; rm -rf "$TMP"; exit 1
  fi
  if [ "$(rc_of "$TMP/t10.md")" != 64 ]; then
    echo "SELFTEST FAIL: legal provenance derivata rc != 64 (A-SK60 positive control / A-SK-79)"; rm -rf "$TMP"; exit 1
  fi
  cp "$TMP/baseline.md" "$TMP/t10b.md"
  echo "delta se: 734.671 B [derivata: prov 1.319.393@$G3REL:115 − 584.723@$G3REL:63]" >> "$TMP/t10b.md"
  if [ "$(rc_of "$TMP/t10b.md")" != 1 ]; then
    echo "SELFTEST FAIL: provenance derivata legalized a WRONG value (A-SK60)"; rm -rf "$TMP"; exit 1
  fi
  # T12 v2 — A-SK66/A-SK-72: the supersession proof lives in the COMMITTED
  # campaign ledger, never in the wording of the citing line (the keyword
  # test was satisfied by «NON e superseded»). Citing a generation with NO
  # ledger proof (g0 never existed) must FAIL regardless of wording; the
  # citation may omit `.out` and is still seen. Citing a superseded
  # generation WITH proof (m89 g1 has an esito=FAIL ledger row) is legal
  # as history even with neutral wording.
  cp "$TMP/baseline.md" "$TMP/t12.md"
  echo "verdetto: verdict89.a1.g0 dice la verita, fidatevi" >> "$TMP/t12.md"
  if [ "$(rc_of "$TMP/t12.md")" != 1 ]; then
    echo "SELFTEST FAIL: citation of an unproved generation (no .out suffix) NOT caught (A-SK66/A-SK-72)"; rm -rf "$TMP"; exit 1
  fi
  cp "$TMP/baseline.md" "$TMP/t12b.md"
  echo "storia: verdict89.a1.g1.out — generazione citata, esito nel ledger" >> "$TMP/t12b.md"
  if [ "$(rc_of "$TMP/t12b.md")" != 64 ]; then
    echo "SELFTEST FAIL: ledger-proved superseded citation wrongly refused (A-SK-72/A-SK-79)"; rm -rf "$TMP"; exit 1
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
  if [ "$(rc_of --verdict "$HERE/../wp89-harness/MEASURE89_RESULTS.md")" != 1 ]; then
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
  if [ "$(rc_of --verdict "$T13DOC")" != 1 ]; then
    printf '%s\n' "$MAN_BKP" > "$MANIFEST_FILE"; rm -f "$T13DOC"; rm -rf "$TMP"
    echo "SELFTEST FAIL: verdict mode accepted a TAMPERED working-tree manifest (WP-92 forge a / A-SK-67)"; exit 1
  fi
  if [ "$(rc_of "$T13DOC")" != 1 ]; then
    printf '%s\n' "$MAN_BKP" > "$MANIFEST_FILE"; rm -f "$T13DOC"; rm -rf "$TMP"
    echo "SELFTEST FAIL: an UNCOMMITTED manifest row was honored (WP-92 forge a / A-SK-67)"; exit 1
  fi
  printf '%s\n' "$MAN_BKP" > "$MANIFEST_FILE"
  rm -f "$T13DOC"
  # T14 — WP-92 forge (b) REPEATED (Klabnik Q2, KS-SK-92-2): the EXACT
  # cross-campaign prov forge — operands from TWO DIFFERENT vmmap files
  # (axum.83c vs axum.82c) — must FAIL on commensurability.
  V83="php-rust/wp78-harness/measure-out/axum.83c.lever.n1000.r1.vmmap.V1"
  V82="php-rust/wp78-harness/measure-out/axum.82c.lever.n1000.r1.vmmap.V1"
  cp "$TMP/baseline.md" "$TMP/t14.md"
  echo "b_base rivisto: 19.600.000 B [derivata: prov 23.000.000@$V83:1963 − 3.400.000@$V82:1964]" >> "$TMP/t14.md"
  if [ "$(rc_of "$TMP/t14.md")" != 1 ]; then
    echo "SELFTEST FAIL: cross-file prov operands NOT refused (WP-92 forge b / A-SK-69/KS-SK-92-2)"; rm -rf "$TMP"; exit 1
  fi
  # T14b — A-SK-69 operator: 'diviso' between operands used to pass
  # because a '-' inside 'wp89-harness' satisfied the /−|-/ test. The
  # masked-expression parser must refuse it and NEVER print
  # provenance-verified.
  cp "$TMP/baseline.md" "$TMP/t14b.md"
  echo "x: 734.670 B [derivata: prov 1.319.393@$G3REL:115 diviso 584.723@$G3REL:63]" >> "$TMP/t14b.md"
  if bash "$0" "$TMP/t14b.md" 2>&1 | grep -q "(A-SK60 provenance-verified"; then
    echo "SELFTEST FAIL: non-minus operator printed provenance-verified (A-SK-69)"; rm -rf "$TMP"; exit 1
  fi
  if [ "$(rc_of "$TMP/t14b.md")" != 1 ]; then
    echo "SELFTEST FAIL: non-minus prov operator NOT refused (A-SK-69)"; rm -rf "$TMP"; exit 1
  fi
  # T14c — A-SK-69: prov citing ADDRESS-RANGE lines (same file, in-pool)
  # must be refused — the vmmap fragments A-SK61 strips from the corpus
  # re-entered through the prov door.
  cp "$TMP/baseline.md" "$TMP/t14c.md"
  echo "y: 19.600.000 B [derivata: prov 23.000.000@$V83:1963 − 3.400.000@$V83:1964]" >> "$TMP/t14c.md"
  if [ "$(rc_of "$TMP/t14c.md")" != 1 ]; then
    echo "SELFTEST FAIL: address-range prov line NOT refused (A-SK-69)"; rm -rf "$TMP"; exit 1
  fi
  # T14d — A-SK-73: an operand path committed at HEAD but OUTSIDE the
  # corpus source set (the budget file itself) must be refused — the
  # prov pool is the budgeted corpus, not the whole tree.
  BREL="php-rust/wp81-harness/gate-cifre-corpus.budget"
  cp "$TMP/baseline.md" "$TMP/t14d.md"
  echo "z: 000 B [derivata: prov 24221@$BREL:1 − 24221@$BREL:1]" >> "$TMP/t14d.md"
  if bash "$0" "$TMP/t14d.md" 2>&1 | grep -q "(A-SK60 provenance-verified"; then
    echo "SELFTEST FAIL: out-of-pool prov path resolved (A-SK-73)"; rm -rf "$TMP"; exit 1
  fi
  # T15 — WP-92 forge (c) REPEATED (Klabnik Q3, KS-SK-92-4): an
  # invoker-supplied cache/nonce must be REFUSED, never parsed.
  if [ "$(rc_of --cache "$TMP/x" --nonce deadbeef "$TMP/baseline.md")" != 1 ]; then
    echo "SELFTEST FAIL: invoker-supplied --cache/--nonce NOT refused (WP-92 forge c / A-SK-70/KS-SK-92-4)"; rm -rf "$TMP"; exit 1
  fi
  # T16 — WP-92 forge (d) REPEATED (Klabnik Q4, KS-SK-92-3/A-SK-71): a
  # figure-bearing doc born in a perimeter class (sessions/) WITHOUT a
  # manifest row must FAIL --all — the cifre that reach the human live in
  # the rotation docs, not only in MEASURE.
  T16DOC="$ROOT/php-rust/sessions/zzforge-t16.md"
  echo "cifra fuori perimetro: il picco era 123457 B, fidatevi" > "$T16DOC"
  if [ "$(rc_of --all)" != 1 ]; then
    rm -f "$T16DOC"; rm -rf "$TMP"
    echo "SELFTEST FAIL: perimeter-class doc without manifest row NOT caught by --all (WP-92 forge d / A-SK-71/KS-SK-92-3)"; exit 1
  fi
  rm -f "$T16DOC"
  # T18 — WP-93 forge F1 REPEATED (Klabnik, A-SK-74): a figure colliding
  # with the DECIMAL prefix of an ancestor commit must FAIL — the
  # rev-parse door is abolished; only committed rev= rows name identities.
  # (loop over candidates: a prefix that happens to live in the corpus is
  # legitimately citable and proves nothing — try the next one)
  T18OK=0
  for T18TOK in $(git -C "$ROOT" rev-list HEAD | cut -c1-7 | grep -E '^[0-9]{7}$' | grep -v '^6910767$' | head -5); do
    cp "$TMP/baseline.md" "$TMP/t18.md"
    T18IT=$(printf '%s' "$T18TOK" | sed -E 's/^([0-9])([0-9]{3})([0-9]{3})$/\1.\2.\3/')
    echo "b_peak rivisto = $T18IT B per worker, fidatevi" >> "$TMP/t18.md"
    if [ "$(rc_of "$TMP/t18.md")" = 1 ]; then T18OK=1; break; fi
  done
  if [ "$T18OK" != 1 ]; then
    echo "SELFTEST FAIL: decimal ancestor-prefix figure NOT caught on any candidate (WP-93 forge F1 / A-SK-74)"; rm -rf "$TMP"; exit 1
  fi
  # negative control: the rev NAMED by a committed rev= row stays citable.
  cp "$TMP/baseline.md" "$TMP/t18b.md"
  echo "baseline WP-80 a git 6910767 (identita' nominata dal revs)" >> "$TMP/t18b.md"
  if [ "$(rc_of "$TMP/t18b.md")" != 64 ]; then
    echo "SELFTEST FAIL: NAMED rev= identity 6910767 wrongly refused (A-SK-74 negative control)"; rm -rf "$TMP"; exit 1
  fi
  # T19 — WP-93 forge F2 REPEATED (Klabnik, A-SK-75/KS-SK-93-3): the ALLOW
  # laundering — 2,8 and 46,25 were council-verbale figures promoted to
  # legal constants everywhere. REVOKED: both must FAIL now.
  cp "$TMP/baseline.md" "$TMP/t19.md"
  echo "regressione CPU full: 2,8 volte la media, tolleranza 46,25 percento" >> "$TMP/t19.md"
  if [ "$(rc_of "$TMP/t19.md")" != 1 ]; then
    echo "SELFTEST FAIL: revoked ALLOW constants 2,8/46,25 still legalize figures (WP-93 forge F2 / A-SK-75/KS-SK-93-3)"; rm -rf "$TMP"; exit 1
  fi
  # T20 — WP-93 forge F3 REPEATED (Klabnik, A-SK-76): glued unit — the
  # tokenizer must REFUSE `1.024.999B`, never judge its truncation `1.024`;
  # the FAIL must name the glue, not the value.
  cp "$TMP/baseline.md" "$TMP/t20.md"
  echo "b_work rivisto = 1.024.999B per worker" >> "$TMP/t20.md"
  if [ "$(rc_of "$TMP/t20.md")" != 1 ]; then
    echo "SELFTEST FAIL: glued-unit figure NOT refused (WP-93 forge F3 / A-SK-76)"; rm -rf "$TMP"; exit 1
  fi
  if ! bash "$0" "$TMP/t20.md" 2>&1 | grep -q 'A-SK-76'; then
    echo "SELFTEST FAIL: glued-unit figure died for the WRONG reason (want the A-SK-76 refusal, not a truncation verdict)"; rm -rf "$TMP"; exit 1
  fi
  # T21 — WP-93 forge F4 REPEATED (Klabnik, A-SK-77): historic budget
  # values are authorities' history, not machine truth — 23.999 must FAIL.
  cp "$TMP/baseline.md" "$TMP/t21.md"
  echo "outstanding al checkpoint: 23.999 allocazioni" >> "$TMP/t21.md"
  if [ "$(rc_of "$TMP/t21.md")" != 1 ]; then
    echo "SELFTEST FAIL: historic budget value 23.999 still legalized (WP-93 forge F4 / A-SK-77)"; rm -rf "$TMP"; exit 1
  fi
  # negative control: the CURRENT committed budget value stays citable as a
  # named authority.
  BUDNOW=$(git -C "$ROOT" show "HEAD:php-rust/wp81-harness/gate-cifre-corpus.budget" | head -1 | sed 's/max_tokens=//')
  cp "$TMP/baseline.md" "$TMP/t21b.md"
  echo "budget corpus in vigore: $BUDNOW token" >> "$TMP/t21b.md"
  if [ "$(rc_of "$TMP/t21b.md")" != 64 ]; then
    echo "SELFTEST FAIL: CURRENT budget value $BUDNOW wrongly refused (A-SK-77 negative control)"; rm -rf "$TMP"; exit 1
  fi
  # T22 — WP-93 forge F5 REPEATED (Klabnik, A-SK-81): prov on a round-MiB
  # value built from two NAKED digit-runs of the same reqns log line —
  # operands must be labeled key=value and share the key.
  V83LOG="php-rust/wp78-harness/measure-out/axum.83cr.lever.n2000.r1.log"
  cp "$TMP/baseline.md" "$TMP/t22.md"
  echo "massa: 18.000.000 B [derivata: prov 18287875@$V83LOG:3 − 287875@$V83LOG:3]" >> "$TMP/t22.md"
  if [ "$(rc_of "$TMP/t22.md")" != 1 ]; then
    echo "SELFTEST FAIL: naked-run prov operands NOT refused (WP-93 forge F5 / A-SK-81)"; rm -rf "$TMP"; exit 1
  fi
  if ! bash "$0" "$TMP/t22.md" 2>&1 | grep -q 'A-SK-81'; then
    echo "SELFTEST FAIL: F5 forge died for the WRONG reason (want the A-SK-81 label refusal)"; rm -rf "$TMP"; exit 1
  fi
  # T17 — WP-93 forge F6 REPEATED (Klabnik, A-SK-78/KS-SK-93-1): a PATCHED
  # copy of the judge (perimeter tooth disabled — exactly the
  # zzforge-judge93 forge) run --all with a forge doc present must NEVER
  # print a verdict-grade PASS: the self-tether refuses any runner whose
  # hash-object differs from the HEAD blob of JUDGE_REL.
  # (the copy lives INSIDE the repo like the original forge: from a tmpdir
  # it would die on "not in a git repo" before the tether ever fired)
  T17JUDGE="$HERE/zzforge-judge-t17.sh"
  perl -pe 's/\$all_rc = 1;//g if /UNCOMMITTED/' "$0" > "$T17JUDGE"
  T17DOC="$ROOT/php-rust/sessions/zzforge-t17.md"
  echo "cifra fuori perimetro: il picco era 123457 B, fidatevi" > "$T17DOC"
  T17OUT=$(bash "$T17JUDGE" --all 2>&1); T17RC=$?
  rm -f "$T17DOC" "$T17JUDGE"
  if [ "$T17RC" = 0 ] || printf '%s\n' "$T17OUT" | grep -q '^PASS gate-measure-cifre --all'; then
    echo "SELFTEST FAIL: a PATCHED judge copy produced a PASS (WP-93 forge F6 / A-SK-78/KS-SK-93-1)"; rm -rf "$TMP"; exit 1
  fi
  # the refusal must come from the TETHER, not from an incidental FAIL of
  # the forge doc: the A-SK-78 line names the reason on the record.
  if ! printf '%s\n' "$T17OUT" | grep -q 'A-SK-78'; then
    echo "SELFTEST FAIL: patched judge copy did not die on the SELF-TETHER (A-SK-78)"; rm -rf "$TMP"; exit 1
  fi
  rm -rf "$TMP"
  echo "SELFTEST PASS: KG-83-3 smuggle + A-SK40 companions + A-SK55 committed-only + A-SK60 provenance (positive+bite) + A-SK62 every-token + A-SK63 manifest graces + A-SK65 env-cache ignored + A-SK53-bis window + A-SK-67 HEAD-authorities (budget tamper, forge-a manifest row) + A-SK-69 strict prov (forge-b: cross-file, non-minus operator, address-range, positive same-file) + A-SK-70 cache abolished (forge-c) + A-SK-71 perimeter (forge-d) + A-SK-72 ledger-proved supersession (.out optional) + A-SK-73 pool=corpus + A-SK-74 named-rev identities (forge F1, T18+control) + A-SK-75 ALLOW-as-authority, 2.8/46.25 revoked (forge F2, T19) + A-SK-76 glued-run refused (forge F3, T20) + A-SK-77 budget-history out of corpus (forge F4, T21+control) + A-SK-78 self-tether (forge F6, T17) + A-SK-79 exit-code grades (every tooth rc-exact) + A-SK-80 complement perimeter + A-SK-81 labeled prov operands (forge F5, T22) all bite"
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
my $REVS_REL     = "php-rust/wp81-harness/gate-cifre-revs.txt";

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
for my $a ([$JUDGE_REL, 'judge'], [$MANIFEST_REL, 'manifest'], [$BUDGET_REL, 'budget'], [$REVS_REL, 'revs']) {
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

# A-SK-75 (Council WP-93, Klabnik forge F2 LANDED — KS-SK-93-3): a NAMED
# protocol constant is an AUTHORITY, not a measure — every entry carries
# the path@line of a COMMITTED blob that STATES it («non è una misura»),
# and the resolution is VERIFIED at HEAD before any judgment: an entry
# whose provenance line does not carry the token is a FAIL of the judge
# itself, never a silent grace.
# REVOKED (KS-SK-93-3): 2.8 and 46.25 — council-verbale figures (judge=no
# sources) promoted to legal constants everywhere; a verbale can carry a
# MANDATE, never mint a citable figure. 110 dropped (no committed
# provenance line names it). 6910767 moved to the revs authority
# (A-SK-74: it is an identity, not a constant).
my %ALLOW_AUTH = (
  '384'   => ['php-rust/wp83-harness/COUNCIL_WP83_REVIEWS.md', 271,
              'bound buffer CString stack std (A-BB27)'],
  '4000'  => ['php-rust/wp81-harness/verdict81.out', 32, 'soglia ex-ante P1a design79 par.10'],
  '8048'  => ['php-rust/wp81-harness/verdict81.out', 33, 'soglia ex-ante P1b design79 par.10'],
  '5000'  => ['php-rust/wp81-harness/verdict81.out', 38, 'soglia ex-ante P5b design79 par.10'],
  '9276'  => ['php-rust/wp85-harness/MEASURE85_RESULTS.md', 39, 'wc -c fixture hello_pad85 (A-SK56)'],
  '0.8'   => ['php-rust/wp89-harness/verdict89.sh', 402, 'soglia alta ex-ante VATTR (KL-90-4)'],
  '1.019' => ['php-rust/wp91-harness/COUNCIL_WP91_REVIEWS.md', 355,
              'ratio anti-moda W8, ricomputo Bak (A-BB62)'],
);
my %ALLOW;
for my $tok (sort keys %ALLOW_AUTH) {
  my ($ap, $al) = @{$ALLOW_AUTH{$tok}};
  my @blob = head_content($ap);
  my $l = $blob[$al-1] // '';
  my $found = 0;
  while ($l =~ /(\d[\d.,]*\d|\d)/g) {
    my $c = $1; (my $n = $c) =~ s/\.(?=\d{3}\b)//g; $n =~ s/,/./;
    if ($n eq $tok || $c eq $tok) { $found = 1; last }
  }
  if (!$found) {
    print "FAIL gate-measure-cifre: ALLOW constant '$tok' — provenance $ap:$al does NOT state it at HEAD (A-SK-75/KS-SK-93-3)\n";
    exit 1;
  }
  $ALLOW{$tok} = 1;
}

# ---- corpus: committed machine outputs -------------------------------------
# A-SK55 (Council WP-90, Klabnik FORGE BITTEN LIVE): the corpus is read
# from the COMMITTED tree at HEAD (git ls-tree + git show HEAD:), never
# from the working tree — an uncommitted forge file in measure-out used
# to legalize any figure (KS-SK-90-1).
my @headtree = split /\n/, qx(git -C "$root" ls-tree -r --name-only HEAD);
my %headset = map { $_ => 1 } @headtree;

# ---- A-AH63 (Council WP-93, Hejlsberg): campaign grammar v2 PRE-BIRTH
# tooth. Any committed m9[1-9]+ campaign ledger is validated against the
# v2 checker BEFORE anything of that campaign is consumed (corpus tokens,
# supersession proofs): ONE non-conformant row voids the whole campaign
# (KS-AH-93-2). Both the LEDGER and the CHECKER run from their HEAD blobs
# — a patched working-tree checker judges nothing (same authority
# discipline as A-SK-67/A-SK-78).
{
  my $CHECKER_REL = "php-rust/wp91-harness/check-campaign-v2.sh";
  my @v2ledgers = grep { m{^php-rust/wp78-harness/measure-out/m9[1-9]\d*\.campaign\.ledger$} } @headtree;
  if (@v2ledgers) {
    if (!$headset{$CHECKER_REL}) {
      print "FAIL gate-measure-cifre: m91+ campaign ledger at HEAD but checker $CHECKER_REL NOT committed (A-AH63)\n";
      exit 1;
    }
    my $tdir = qx(mktemp -d); chomp $tdir;
    qx(git -C "$root" show "HEAD:$CHECKER_REL" > "$tdir/chk.sh");
    for my $lg (@v2ledgers) {
      qx(git -C "$root" show "HEAD:$lg" > "$tdir/l.ledger");
      my $rc = system('bash', "$tdir/chk.sh", "$tdir/l.ledger");
      if ($rc != 0) {
        qx(rm -rf "$tdir");
        print "FAIL gate-measure-cifre: campaign ledger $lg violates grammar v2 (A-AH63/KS-AH-93-2) — campagna VOID alla consumazione\n";
        exit 1;
      }
    }
    qx(rm -rf "$tdir");
  }
}
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
# S-92.0: the wp92 machine outputs (ds35-verify4 length-prefixed pins on 51
# fixtures, dl59-join ADVISORY analysis) — budget raised in the SAME commit
# (A-SK61 deliberate-growth discipline)
push @sources, committed_glob("$here/../wp92-harness/*.out");
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
# A-AH64 (Council WP-93, Hejlsberg): the corpus admits the MAX generation
# ONLY with a committed esito=PASS row in its campaign ledger — the old
# filter kept "max by number G" with no outcome check: a campaign ending
# in an unsuperseded FAIL would have legalized its figures as truth
# (max-FAIL = history, never truth — KS-AH-93-3). Families with NO
# campaign ledger at HEAD predate the ledger era and stand as judged
# (declared; today m88/m89/m90 all carry their PASS rows, verified).
my %max_unproven;
for my $k (keys %gen_max) {
  my ($dir, $nn, $a) = split /\|/, $k;
  my $lrel = "php-rust/wp78-harness/measure-out/m$nn.campaign.ledger";
  next unless $headset{$lrel};
  my $g = $gen_max{$k};
  my $ok = 0;
  for my $l (head_content($lrel)) {
    $l =~ s/\0//g;
    next unless $l =~ /esito=PASS\b/;
    if ($l =~ /generation=g$g\b/ || $l =~ /verdict\Q$nn\E\.a\Q$a\E\.g\Q$g\E\.out/) { $ok = 1; last }
  }
  if (!$ok) {
    $max_unproven{$k} = 1;
    print "NOTE: verdict$nn.a$a.g$g is MAX by number but has NO committed esito=PASS ledger row — its figures are OUT of the corpus (A-AH64/KS-AH-93-3: max-FAIL is history, never truth)\n";
  }
}
@sources = grep {
  !(m{^(.*/)verdict(\d+)\.a(\d+)\.g(\d+)\.out$} && ($4 < $gen_max{"$1|$2|$3"} || $max_unproven{"$1|$2|$3"}))
} @sources;
# A-SK-73 (Council WP-92): the prov operand pool IS the corpus source set —
# governed by the same committed A-SK61 budget. Before this, any figure on
# any line of any HEAD blob was an operand (36.573 addressable vs 24.042
# budgeted): the budget measured the wrong surface.
my %src_set = map { $_ => 1 } @sources;
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

# A-SK-77 (Council WP-93, Klabnik forge F4 LANDED): the budget HISTORY is
# OUT of the corpus. It is an AUTHORITY, not machine output — F4 minted
# «23.999 allocazioni» from a historic budget blob — and it was already
# excluded from the prov pool (A-SK-73): keeping it as a token source was
# an internal contradiction («autorità E sorgente di corpus»). Only the
# CURRENT committed budget value is citable, as a named authority (added
# to %ALLOW below, after the HEAD parse; the gate prints it every run).

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
# A-SK-77: the CURRENT committed budget value is a named authority — its
# provenance is the HEAD budget file itself, parsed and printed right here.
$ALLOW{$budget} = 1;
print "corpus cardinality=$card budget=$budget (A-SK61, budget from HEAD; A-SK-77: history OUT of corpus, current value = named authority)\n";
if ($card > $budget) {
  print "FAIL gate-measure-cifre: corpus cardinality $card EXCEEDS committed budget $budget (A-SK61) — raise the budget deliberately with the new sources\n";
  exit 1;
}

# ---- manifest from HEAD (A-SK-67) ------------------------------------------
my @man_rows;
for my $row (head_content($MANIFEST_REL)) {
  chomp $row;
  next if $row =~ /^\s*(#|$)/;
  my ($mpath, $msha, $mjudge, $mbf, $mbands, $mverd) = split /\t/, $row;
  push @man_rows, [$mpath, $msha, $mjudge, $mbf, $mbands, $mverd];
}

# ---- identity/citation helpers ---------------------------------------------
# A-SK-74 (Council WP-93, Klabnik forge F1 LANDED): the rev-parse door is
# ABOLISHED — 53 ancestor commits have 7-digit all-decimal prefixes and 32
# have 8, exactly the byte window (1-99 M): the "rare residual" declared by
# the old exemption was a known population, and Italian normalization
# (23.330.397 -> 23330397) carried a fabricated figure straight through the
# door. A decimal token of >=7 digits is an identity ONLY if it is a prefix
# of a sha40 NAMED by a committed rev= row in the revs authority (read from
# HEAD like every authority — A-SK-67).
my @REV_IDS;
for my $l (head_content($REVS_REL)) {
  push @REV_IDS, $1 if $l =~ /^rev=([0-9a-f]{40})\b/;
}
sub is_named_rev_identity {
  my ($tok) = @_;
  return 0 unless $tok =~ /^\d{7,40}$/;
  for my $r (@REV_IDS) { return 1 if index($r, $tok) == 0 }
  return 0;
}
# A-SK-72 (Council WP-92): the supersession of a verdict generation is
# PROVED by a committed campaign-ledger row (esito=FAIL for that
# generation, or supersede_of=g<G>) — never by wording on the citing line
# («NON e superseded» passed the old keyword test).
my %sup_cache;
sub ledger_supersession_proved {
  my ($nn, $g) = @_;
  my $k = "$nn|$g";
  return $sup_cache{$k} if exists $sup_cache{$k};
  my $lrel = "php-rust/wp78-harness/measure-out/m$nn.campaign.ledger";
  my $ok = 0;
  if ($headset{$lrel}) {
    for my $l (head_content($lrel)) {
      $l =~ s/\0//g;
      if ($l =~ /supersede_of=g$g\b/ || ($l =~ /generation=g$g\b/ && $l =~ /esito=FAIL\b/)) { $ok = 1; last }
    }
  }
  return $sup_cache{$k} = $ok;
}

# ---- resolve target list ---------------------------------------------------
my @targets;   # [abs-or-rel display name, repo-rel path or '', source: head|work]
my $all_rc = 0;
if ($target_arg eq '--all') {
  for my $r (@man_rows) {
    my ($mpath, $msha, $mjudge, $mbf, $mbands, $mverd) = @$r;
    if (!$headset{$mpath}) {
      print "FAIL gate-measure-cifre --all: manifest doc MISSING from HEAD: $mpath (A-SK64/A-SK-67)\n"; $all_rc = 1; next;
    }
    if ($mjudge eq 'no') {
      # A-SK-68 (Council WP-92): judge=no is NOT "cifra libera" (Gregg's
      # condition) — a FROZEN row keeps its doc sha-pinned (drift at HEAD
      # = FAIL) and anchors its truth to the EPOCH VERDICT blob
      # (verdict=<repo-path>@<sha16>, resolvable at HEAD). The blob
      # comparisons ride the same HEAD-authority code path exercised by
      # T11/T13 (a committed drift cannot be simulated in a selftest).
      my $ok = 1;
      if (defined $msha && $msha ne '-') {
        my $h = head_blob_sha($mpath);
        if ($h ne $msha) {
          print "FAIL gate-measure-cifre --all: frozen judge=no doc $mpath DRIFTED from its manifest pin (A-SK-68): $h vs $msha\n";
          $all_rc = 1; $ok = 0;
        }
      }
      my $anch = '';
      if (defined $mverd && $mverd =~ /^verdict=([^\@\s]+)\@([0-9a-f]{16})$/) {
        my ($vp, $vs) = ($1, $2);
        my $vh = head_blob_sha($vp);
        if (substr($vh, 0, 16) ne $vs) {
          print "FAIL gate-measure-cifre --all: epoch verdict anchor for $mpath does NOT resolve at HEAD ($vp \@ $vs) (A-SK-68)\n";
          $all_rc = 1; $ok = 0;
        } else { $anch = ", epoch verdict anchored: $vp (A-SK-68)" }
      }
      print "SKIP $mpath (declared judge=no in manifest — A-SK64$anch)\n" if $ok;
      next;
    }
    push @targets, [$mpath, $mpath, 'head'];
  }
  # reverse direction — A-SK-71 (Council WP-92, KS-SK-92-3): the perimeter
  # is every committed .md that PUBLISHES session figures, not just the
  # MEASURE docs (43 cifre in NEXT_SESSION + WP_SESSION_90 arrivavano
  # all'umano NON giudicate contro 28 giudicate). Classes (declared):
  # MEASURE docs, NEXT_SESSION_WORDPRESS.md, sessions/, design docs,
  # council verbali, gap reports. A class member without a manifest row is
  # a FAIL — at HEAD and in the working tree alike (a doc can be born
  # uncommitted: WP-92 forge (d)).
  my %manset = map { $_->[0] => 1 } @man_rows;
  # A-SK-80 (Council WP-93, Klabnik): perimeter by COMPLEMENT, not by
  # allowlist of classes — every committed .md under php-rust/
  # (subdirectories INCLUDED) requires a manifest row. The old class list
  # left 153 figures in the four most-read GitHub docs (FOOTPRINT_CPU_
  # ROADMAP/COVERAGE/TODO/README) and the council verbali dirs outside
  # the perimeter. A committed or untracked .md with no row is a FAIL.
  my $class_rx = qr{^php-rust/.*\.md$};
  for my $f (grep { /$class_rx/ } @headtree) {
    next if $manset{$f};
    print "FAIL gate-measure-cifre --all: $f has NO manifest entry (A-SK64/A-SK-71 bidirectional)\n"; $all_rc = 1;
  }
  for my $f (split /\n/, qx(git -C "$root" ls-files --others --exclude-standard -- php-rust)) {
    next unless $f =~ /$class_rx/;
    next if $manset{$f};
    print "FAIL gate-measure-cifre --all: UNCOMMITTED $f in a perimeter class with NO manifest entry (A-SK-71/KS-SK-92-3): a figure-bearing doc born outside the perimeter is never verdict-grade\n"; $all_rc = 1;
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

  # manifest row lookup (from HEAD rows). A-SK-68 (Council WP-92): the
  # legacy_frozen branch is ABOLISHED — "doc STORICO CONGELATO" was a
  # comment, not a predicate (no committed-ness test, no membership test;
  # Klabnik's forge (a) rode it through an uncommitted manifest row). The
  # 5 historical docs are now judge=no rows anchored to their EPOCH
  # VERDICT blobs (see the --all branch above); no new row may ever grant
  # the pre-WP-91 evaluator semantics — the code is gone.
  my $bytes_first = 1;
  my @doc_bands;
  if ($rel) {
    for my $r (@man_rows) {
      my ($mpath, $msha, $mjudge, $mbf, $mbands, $mverd) = @$r;
      next unless defined $mpath && $mpath eq $rel;
      $bytes_first = (defined $mbf && $mbf eq 'no') ? 0 : 1;
      my $sha_match = 0;
      if (defined $msha && $msha ne '-') {
        # the pin is judged against the CONTENT BEING JUDGED: HEAD blob in
        # verdict/all mode, working blob in advisory mode
        my $tsha = ($src eq 'head') ? head_blob_sha($rel) : work_blob_sha("$root/$rel");
        $sha_match = ($tsha eq $msha) ? 1 : 0;
        print "line 0: NOTE — $rel edited since its manifest pin: ± graces REVOKED (A-SK63)\n"
          unless $sha_match;
      }
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
    # A-SK66 (Council WP-91) + A-SK-72 (Council WP-92) citation tooth:
    # naming a NON-max generation is legal ONLY if the supersession is
    # PROVED by a committed campaign-ledger row — the old keyword test
    # («supersed|judge-unrecoverable») was satisfied by «NON e superseded»
    # (word, not proof). The `.out` suffix is OPTIONAL in the citation
    # regex: a citation without it used to be invisible.
    while ($line =~ /verdict(\d+)\.a(\d+)\.g(\d+)(?:\.out)?\b/g) {
      my ($nn, $a, $g) = ($1, $2, $3);
      my $mx = $cite_max{"$nn|$a"};
      if (!defined $mx) {
        push @miss, "line $ln: cites verdict$nn.a$a.g$g but NO committed generation exists for that family (A-SK66/KS-SK-91-4): $line";
      } elsif ($g > $mx) {
        push @miss, "line $ln: cites generation g$g NOT committed for that family (max g$mx) (A-SK66): $line";
      } elsif ($g < $mx && !ledger_supersession_proved($nn, $g)) {
        push @miss, "line $ln: cites NON-max generation g$g (max committed g$mx) with NO committed ledger row proving the supersession (A-SK66/A-SK-72): $line";
      }
    }
    # (1) verified pairs "N B = X <unit>": ALWAYS attempted — a pair that
    # verifies arithmetically exempts its companion half in ANY doc class
    # (the rotation docs are bytes_first=no but still write verified pairs).
    # ENFORCEMENT (mismatch = miss, A-SK43 unit-i) only in bytes-first
    # docs: for bytes_first=no the pair check is purely an EXEMPTION —
    # fail-open to the corpus scan, no new failure mode on legacy docs.
    while ($work =~ /((\d[\d.,]*)\s*B\s*=\s*(\d[\d.,]*)\s*([KMGTkmgt])(i?)[Bb]\b)/) {
      my ($whole, $braw, $fraw, $u, $ii) = ($1, $2, $3, $4, $5);
      my $bytes = it_num($braw);
      my $fig   = it_num($fraw);
      my $dec   = ($fig =~ /\.(\d+)$/) ? length($1) : 0;
      my $tol   = 0.5 * 10 ** (-$dec) + 1e-9;
      my $calc  = $bytes / $SCALE{lc $u};
      my $pair_ok = (abs($calc - $fig) <= $tol) ? 1 : 0;
      if ($bytes_first) {
        # A-SK43 (Council WP-87, Klabnik Q2): the corpus does BINARY math —
        # "MB" under SI reading lies to the outside reader.
        if ($ii eq '') {
          push @miss, "line $ln: unit '${u}B' without the i — MiB-only in MEASURE8[4-9] (A-SK43): $line";
        }
        if (!$pair_ok) {
          push @miss, sprintf("line %d: companion MISMATCH (A-SK40): %s B / %s = %.4f, displayed %s: %s",
                              $ln, $bytes, lc($u)."iB-scale", $calc, $fig, $line);
        }
      }
      if (!$bytes_first && !$pair_ok) {
        # unverified pair in a no-BF doc: blank to advance, NO exemption —
        # its halves stay bound to the corpus scan
        my $blank = ' ' x length($whole);
        $work =~ s/\Q$whole\E/$blank/;
        next;
      }
      $companion_ok{$fig} = 1;       # the verified MiB half only (A-SK62)
      my $blank = ' ' x length($whole);
      $work =~ s/\Q$whole\E/$blank/;
    }
    if ($bytes_first) {
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
      # A-SK60 (Council WP-91) + A-SK-69/A-SK-73 (Council WP-92, Klabnik
      # forge (b) LANDED — the free evaluator was not abolished, it was
      # ANNOTATED: 36.573 addressable operands vs 24.042 corpus tokens,
      # 46,25% closure of the plausible window, cross-campaign operands,
      # operator satisfied by a '-' inside 'wp89-harness'). A derived
      # value legalizes a token ONLY by STRICT provenance:
      #   [derivata: prov <N>@<repo/path>:<line> − <M>@<repo/path>:<line>]
      # with ALL of (fail-closed):
      #   - exactly two operands and a MINUS operator verified on the
      #     MASKED expression (paths blanked first — A-SK-69);
      #   - both operands from the SAME file (KS-SK-92-2 commensurability);
      #   - the file is a member of the BUDGETED corpus source set — the
      #     prov pool is the corpus, governed by the same A-SK61 budget
      #     (A-SK-73);
      #   - the cited line REFUSED if it carries an address range (hex or
      #     long pure-digit — vmmap fragments re-entered through this
      #     door), and A-SK61-stripped before the operand search.
      my %eval_ok;
      my $tagtext = ($line =~ /\[derivata:([^\]]*)/) ? $1 : '';
      if ($tagtext =~ /^\s*prov\s/) {
        my @ops_info;
        while ($tagtext =~ /(\d(?:[\d.,]*\d)?)\@([^\s:\]]+):(\d+)/g) {
          push @ops_info, [$1, $2, $3];
        }
        (my $opmask = $tagtext) =~ s/\d(?:[\d.,]*\d)?\@[^\s:\]]+:\d+/OPERAND/g;
        my $op_ok = ($opmask =~ /^\s*prov\s+OPERAND\s*(?:\xE2\x88\x92|-)\s*OPERAND\s*$/) ? 1 : 0;
        if (@ops_info != 2 || !$op_ok) {
          push @miss, "line $ln: [derivata: prov] tag malformed or operator not a verified MINUS on the masked expression (A-SK-69): $line";
        } elsif ($ops_info[0][1] ne $ops_info[1][1]) {
          push @miss, "line $ln: prov operands from DIFFERENT files ($ops_info[0][1] vs $ops_info[1][1]) — not commensurable (A-SK-69/KS-SK-92-2): $line";
        } elsif (!$src_set{$ops_info[0][1]}) {
          push @miss, "line $ln: prov operand path $ops_info[0][1] OUTSIDE the budgeted corpus source set (A-SK-73): $line";
        } else {
          my (@ops, @opkeys, $bad);
          for my $oi (@ops_info) {
            my ($tok, $rp, $rl) = @$oi;
            my $ntok = it_num($tok);
            my @blob = head_content($rp);
            my $srcline = $blob[$rl-1] // ''; $srcline =~ s/\0//g;
            if ($srcline =~ /\b(?:(?=[0-9a-f]*[a-f])[0-9a-f]{6,}-[0-9a-f]{6,}|[0-9a-f]{6,}-(?=[0-9a-f]*[a-f])[0-9a-f]{6,}|\d{6,}-\d{6,})\b/) {
              push @miss, "line $ln: prov operand cites an ADDRESS-RANGE line ($rp:$rl) — refused (A-SK-69): $line";
              $bad = 1; last;
            }
            $srcline =~ s/\b(?:(?=[0-9a-f]*[a-f])[0-9a-f]{6,}-[0-9a-f]{6,}|[0-9a-f]{6,}-(?=[0-9a-f]*[a-f])[0-9a-f]{6,})\b/ /g;
            # A-SK-81 (Council WP-93, Klabnik forge F5 LANDED): an operand is
            # a LABELED key=value on the cited line — any naked digit-run of
            # the line is NOT an operand (F5 picked two naked runs from a
            # reqns list and closed 100% of the publishable MiB-round space).
            my $okey = '';
            while ($srcline =~ /([A-Za-z_][A-Za-z0-9_-]*)=(\d[\d.,]*\d|\d)\b/g) {
              my ($k, $v) = ($1, $2);
              (my $nv = $v) =~ s/\.(?=\d{3}\b)//g; $nv =~ s/,/./;
              if ($nv eq $ntok || $v eq $ntok) { $okey = $k; last; }
            }
            if (!$okey) {
              push @miss, "line $ln: prov operand $ntok is NOT a labeled key=value on $rp:$rl (A-SK-81): $line";
              $bad = 1; last;
            }
            print "line $ln: derivata operand $ntok <= $rp:$rl resolved at HEAD as $okey= (A-SK60/A-SK-69/A-SK-81)\n";
            push @ops, $ntok; push @opkeys, $okey;
          }
          # A-SK-81: both operands must share the key — same magnitude.
          if (!$bad && @ops == 2 && $opkeys[0] ne $opkeys[1]) {
            push @miss, "line $ln: prov operands carry DIFFERENT keys ($opkeys[0]= vs $opkeys[1]=) — not the same magnitude (A-SK-81): $line";
            $bad = 1;
          }
          if (!$bad && @ops == 2) {
            $eval_ok{$ops[0] - $ops[1]} = 1;
            printf "line %d: derivata value %s = %s − %s (A-SK60 provenance-verified, same-file+in-pool — A-SK-69/A-SK-73)\n",
                   $ln, $ops[0] - $ops[1], $ops[0], $ops[1];
          }
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
      # A-SK-76: the glued-run refusal bites on [derivata] lines too.
      while ($probe2 =~ /(?<![\dA-Za-z,.±])(\d[\d.,]*\d|\d)(?=[A-Za-z])/g) {
        my $glue = $1;
        (my $gd = $glue) =~ s/[.,]//g;
        next if length($gd) < 3;
        push @miss, "line $ln: digit-run '$glue' GLUED to a letter on a [derivata] line — refused, never truncated (A-SK-76): $line";
      }
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
        next if is_named_rev_identity($norm);
        push @miss, "line $ln: token '$raw' on a [derivata] line NOT in corpus and NOT provenance-verified (A-SK56/A-SK60/A-SK62): $line";
      }
      next;
    }
    my $probe = $line;
    # remove hex identities and alphanumeric IDs so their digits don't tokenize
    $probe =~ s/\b(?=[0-9a-f]*[a-f])[0-9a-f]{7,}\b//gi;   # git/sha fragments (7+ hex with a letter)
    $probe =~ s/[A-Za-z_][A-Za-z0-9_-]*[0-9][A-Za-z0-9_-]*//g; # KS-AH-83-1, F13, wp81...
    # A-SK-76 (Council WP-93, Klabnik forge F3 LANDED): a digit-run GLUED to
    # a letter is REFUSED, never truncated — the token alternation used to
    # backtrack `1.024.999B` down to a legal `1.024` and judge THAT. Scope:
    # >=3 digits, same as the gate.
    while ($probe =~ /(?<![\dA-Za-z,.±])(\d[\d.,]*\d|\d)(?=[A-Za-z])/g) {
      my $glue = $1;
      (my $gd = $glue) =~ s/[.,]//g;
      next if length($gd) < 3;
      push @miss, "line $ln: digit-run '$glue' GLUED to a letter — refused, never truncated (A-SK-76): $line";
    }
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
      next if is_named_rev_identity($norm);
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
    print "ADVISORY-PASS gate-measure-cifre (KG-83-3, NEVER verdict-grade — T2/A-SK-67; exit=64 A-SK-79): every bound figure in $disp matches committed machine output (or carries [derivata]/named-constant)\n";
  } else {
    print "PASS gate-measure-cifre (KG-83-3): every bound figure in $disp matches committed machine output (or carries [derivata]/named-constant) [judge_sha=$ASHA{judge} manifest_sha=$ASHA{manifest} budget_sha=$ASHA{budget} revs_sha=$ASHA{revs} head=".substr($headrev,0,12)."]\n";
  }
}
if ($target_arg eq '--all' && $grand_rc == 0) {
  print "PASS gate-measure-cifre --all (A-SK64/A-SK-67): manifest perimeter, bidirectional, authorities from HEAD [judge_sha=$ASHA{judge} manifest_sha=$ASHA{manifest} budget_sha=$ASHA{budget} revs_sha=$ASHA{revs} head=".substr($headrev,0,12)."]\n";
}
# A-SK-79 (Council WP-93, Klabnik Q4 — KS-SK-93-4): the GRADE lives in the
# exit code. An advisory CLEAN result exits 64 (ADVISORY-PASS), never 0:
# rc=0 means verdict-grade PASS and nothing else — run_gate used to treat
# an advisory 0 as a closed battery row.
exit 64 if $mode eq 'advisory' && $grand_rc == 0;
exit $grand_rc;
PERL
