#!/bin/bash
# battery-equivalence.sh — A-SK30/A-AH34 (Council WP-84), v2 A-SK32/A-SK33/
# A-AH36 (Council WP-85), v3 A-SK36/A-SK37/A-AH39 (Council WP-86): stamp
# ANCORATO alla riga terminale + .done solo-PASS con sha256(OUT) ricomputato,
# corner index (staged) nel check ledger, claim PROVVISORIO fino al commit
# dell'append, manifest lever-fixtures2 nomina l'oggetto in-cargo
# (crates/php-runtime/). v4 A-SK41 (Council WP-87): OUT↔.done coherence
# alone proves only SELF-consistency (a hand-written OUT with a recomputed
# sha passes it) — the stamp must ALSO exist in the COMMITTED canonical
# battery-stamps ledger, appended by the battery itself at run time
# (KS-SK-87-1: stamp not ledgered => battery VOID). Storia v2: KH82-2 REWRITTEN, poi le TRE elusioni di
# Klabnik chiuse nel CODICE (non nel commento) e il buco trascluso di
# Hejlsberg:
#   (a) OUT mai legato a BREV -> ora il summary DEVE portare `git=$BREV` e
#       il `.done` accanto a OUT deve portare `rev=$BREV` (A-SK32);
#   (b) ledger parametrico touch-creato -> ora il ledger è CANONICO, path
#       pinnato QUI, tracked in git, e la copia di lavoro deve essere
#       IDENTICA alla versione a HEAD prima del claim (append poi committato
#       nella campagna) (A-SK33, KS-SK-85-2);
#   (c) FAIL smerciato come "assente con motivo" -> un `^FAIL <name>` per un
#       gate del set rifiuta SEMPRE, il named-absent non lo copre (A-SK32,
#       KS-SK-85-1).
# An equivalence claim ("the battery at rev B certifies HEAD") is legal IFF,
# all computed HERE:
#   (i)   `git diff B..HEAD -- crates/ Cargo.toml Cargo.lock` is EMPTY;
#   (ii)  every expected gate is present as `OK <name>` BY NAME in the
#         battery output; gates ABSENT form a NAMED set with a caller-
#         supplied reason — and {parity-full, lever-fixtures,
#         lever-fixtures2, measure-cifre} may NEVER be absent; a gate that
#         FAILED in the battery refuses outright (never "absent");
#   (iii) at most ONE equivalence per chain: ledger line per battery rev —
#         a second claim on the same rev REFUSES (never transitive: B must
#         be an ancestor whose battery RAN, not itself an equivalence);
#   (iv)  never for a gate whose OBJECT changed in B..HEAD — object = the
#         gate SCRIPT plus its NAMED transcluded helpers/fixtures (A-AH36:
#         manifest per-gate below; census-twin declares run-gate.sh +
#         fixtures/, the two run-gates declare their fixture .php).
# v5 A-SK46/A-AH43/A-AH44 (Council WP-88):
#   --same-rev: the campaign fast-path (BREV == HEAD) used to bypass this
#     checker ENTIRELY (S-86.0 consumed its battery with no machine
#     recomputing sha256(OUT) nor checking the ledgered stamp — KS-SK-88-1).
#     With --same-rev the OUT/.done/ledger teeth run in full; the
#     equivalence-only teeth (delta, one-per-chain, object-change) are
#     skipped and NO ledger append happens.
#   A-SK46: the committed stamp is matched on ALL FOUR fields (rev, sha256,
#     matrix, matrix_sha256) and the matrix archive named in .done must be
#     COMMITTED at HEAD with a matching sha.
#   A-AH43: the ledger is read from ONE canonical git path resolved via
#     `git rev-parse --show-prefix` — never a two-source concatenation (a
#     tracked twin at the git root could certify a stamp only IT carries).
#   A-AH44: toolchain comparator — the matrix archive records `rustc=`;
#     it must equal the CURRENT `rustc -V` or the claim fails BY NAME
#     (KS-AH-87-2 made mechanical).
# Usage (v2 — the ledger is NO LONGER a parameter, A-SK33):
#   battery-equivalence.sh [--same-rev] <battery.out> <battery_rev> [absent-gate:reason ...]
# Exit 0 = equivalence LEGAL (or same-rev consumption LEGAL); !=0 =
# precondition FAIL (campaign VOID, KS-SK-84-1/KS-AH-84-3/KS-SK-85-1/
# KS-SK-85-2/KS-SK-88-1).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
SAME_REV=0
# A-SK57 bite-test (same-commit, Council WP-90): the row-granularity
# prefix logic must bite on (a) non-prefix rewrite, (b) last-row
# EXTENSION smuggled as append; and pass (c) a legal row append. Pure
# string check — no git state needed, so the tooth is testable anywhere.
if [ "${1:-}" = "--selftest-prefix" ]; then
  st_judge() { # <old> <new> -> echoes REWRITE | EXTEND | OK
    local old="$1" new="$2" rem
    rem="${new#"$old"}"
    if [ -n "$old" ] && [ "$rem" = "$new" ] && [ "$old" != "$new" ]; then echo REWRITE
    elif [ -n "$old" ] && [ -n "$rem" ] && [ "${rem#$'\n'}" = "$rem" ]; then echo EXTEND
    else echo OK; fi
  }
  OLDL=$'rev=aaa sha=111\nrev=bbb sha=222'
  R1=$(st_judge "$OLDL" $'rev=zzz sha=999\nrev=bbb sha=222')   # history rewritten
  R2=$(st_judge "$OLDL" "${OLDL}X")                            # last row extended
  R3=$(st_judge "$OLDL" "${OLDL}"$'\nrev=ccc sha=333')         # legal append
  if [ "$R1" = REWRITE ] && [ "$R2" = EXTEND ] && [ "$R3" = OK ]; then
    echo "SELFTEST-PREFIX PASS: rewrite/extension bite, legal append passes (A-SK57)"
    exit 0
  fi
  echo "SELFTEST-PREFIX FAIL: got R1=$R1 R2=$R2 R3=$R3 (want REWRITE/EXTEND/OK) — A-SK57 tooth does not bite"
  exit 1
fi
if [ "${1:-}" = "--same-rev" ]; then SAME_REV=1; shift; fi
OUT="${1:?battery.out}"; BREV="${2:?battery rev}"
shift 2
FAILS=0
fail() { echo "FAIL: $*"; FAILS=$((FAILS+1)); }

HEADREV=$(git -C "$REPO" rev-parse --short HEAD)
# --same-rev semantics: same CODE revision, not same commit id — the A-SK41
# stamp commit itself moves HEAD by one evidence-only commit. Teeth (i)
# (crates/Cargo delta EMPTY) and (iv) (gate objects unchanged) below are
# the judges of "same code"; BREV must still be an ancestor.
if [ "$SAME_REV" = 1 ] && ! git -C "$REPO" merge-base --is-ancestor "$BREV" HEAD 2>/dev/null; then
  fail "(--same-rev) battery rev $BREV is not an ancestor of HEAD $HEADREV"
fi
# A-AH43: ONE canonical git path for every `git show` in this checker.
GITPREFIX="$(git -C "$REPO" rev-parse --show-prefix)"

# A-SK33: the ONE canonical ledger — pinned here, never a parameter.
LEDGER="$HERE/evidence/equivalence.ledger"
LEDGER_REL="wp83-harness/evidence/equivalence.ledger"

# The named gate set (mirror of battery-83pre.sh). Format per line:
#   name:script[,transcluded-object ...]   (A-AH36 — a trailing / marks a
# directory prefix; every object is part of tooth (iv)'s change surface).
# Bump = same-commit, named.
GATES="feature-matrix:wp78-harness/gate-feature-matrix.sh
census-twin:wp78-harness/gate-census-twin.sh,wp78-harness/gate-axum/run-gate.sh,wp78-harness/gate-axum/fixtures/
doc-purge:wp78-harness/gate-doc-purge.sh,wp78-harness/gate-axum/fixtures/
capture-order:wp78-harness/gate-capture-order.sh
concurrent:wp78-harness/gate-concurrent.sh
stdout-tandem:wp78-harness/gate-stdout-tandem.sh
worker-panic:wp78-harness/gate-worker-panic.sh
run-gate-cli:wp77-harness/run_gate_g_apertura_2.sh,wp77-harness/fixtures/gate_two_reqs_same_vm.php
run-gate-axum:wp77-harness/run_gate_g_apertura_2_axum.sh,wp77-harness/fixtures/gate_two_reqs_same_vm.php
dr1:wp80-harness/gate-dr1-module-immut.sh
lever-pins:wp81-harness/gate-lever-pins.sh
lever-fixtures:wp81-harness/gate-lever-fixtures.sh
lever-fixtures2:wp81-harness/gate-lever-fixtures2.sh,crates/php-runtime/
measure-cifre:wp81-harness/gate-measure-cifre.sh
axum-tests:wp88-harness/gate-axum-tests.sh
parity-full:wp83-harness/gate-parity-83p1.sh"
NEVER_ABSENT="parity-full lever-fixtures lever-fixtures2 measure-cifre"

# (i) crates/lock delta EMPTY, computed here (same-rev: trivially empty,
# still computed — a wrong BREV would show here too).
# v6 A-AH47 (Council WP-89): rust-toolchain.toml JOINS the pathspec — a
# toolchain-pin change between BREV and HEAD compiles ANOTHER compiler
# while every source file is byte-identical (KS-AH-89-1).
DELTA=$(git -C "$REPO" diff --name-only "$BREV..HEAD" -- crates Cargo.toml Cargo.lock rust-toolchain.toml 2>/dev/null)
if [ -n "$DELTA" ]; then
  fail "(i) crates/Cargo/toolchain delta $BREV..$HEADREV NOT empty:"; echo "$DELTA" | head -5
fi

# changed files for tooth (iv)
CHANGED=$(git -C "$REPO" diff --name-only "$BREV..HEAD" 2>/dev/null)

# object_changed <path-or-dirprefix> -> 0 if any changed file matches it.
# v6 A-AH46 (Council WP-89, Hejlsberg): ONE source for the prefix — the
# ${GITPREFIX} already resolved above. The old hardcoded `php-rust/` plus
# bare fallback was the two-source pathology A-AH43 banned: on a checkout
# with a different directory name tooth (iv) went vacuous in silence.
object_changed() {
  local obj="$1"
  case "$obj" in
    */) echo "$CHANGED" | grep -qE "^${GITPREFIX}${obj}" ;;
    *)  echo "$CHANGED" | grep -qxF "${GITPREFIX}${obj}" ;;
  esac
}

# (ii)+(iv) per-NAME presence; absent = named set with reason; FAIL refuses
[ -f "$OUT" ] || { fail "(ii) battery output missing: $OUT"; echo "== EQUIVALENCE REFUSED =="; exit 1; }

# A-SK36 (v3, Council WP-86 — supersedes the A-SK32 substring stamp): OUT is
# legal evidence ONLY with the ANCHORED terminal pass line
#   == BATTERY-<N>PRE PASS (k/k) git=$BREV ==
# unique in the file AND its last line (an appended `git=<rev>` on a stale
# OUT, or a prefix-match like git=${BREV}xx, no longer passes). The .done
# next to OUT must carry rev=$BREV AND sha256=<sha256(OUT)>, which this
# checker RECOMPUTES — a copied OUT with a hand-forged .done is refused.
PASSRE="^== BATTERY-[0-9]+PRE PASS \([0-9]+/[0-9]+\) git=$BREV ==\$"
NPASS=$(grep -cE "$PASSRE" "$OUT" || true)
if [ "$NPASS" != 1 ] || ! tail -1 "$OUT" | grep -qE "$PASSRE"; then
  fail "(A-SK36) anchored terminal PASS line for rev $BREV absent/duplicated/not-final in OUT (count=$NPASS) — KS-SK-86-1"
fi
DONE="$(dirname "$OUT")/.done"
if [ ! -f "$DONE" ]; then
  fail "(A-SK36) .done next to OUT missing — the battery of rev $BREV never COMPLETED there (a PASS-only stamp)"
else
  grep -q "^rev=$BREV " "$DONE" || grep -qx "rev=$BREV" "$DONE" || \
    fail "(A-SK36) .done does not stamp rev=$BREV"
  # v4 .done format (A-AH40) carries TWO sha fields (sha256= of OUT and
  # matrix_sha256=): the old greedy `.*sha256=` matched the LAST one and
  # compared the MATRIX sha against sha256(OUT) — anchored parse.
  DSHA=$(sed -n 's/^rev=[0-9a-f]* sha256=\([0-9a-f]*\).*/\1/p' "$DONE" | head -1)
  OSHA=$(shasum -a 256 "$OUT" | cut -d' ' -f1)
  if [ -z "$DSHA" ]; then
    fail "(A-SK36) .done carries no sha256= — pre-v3 or forged stamp (KS-SK-86-1)"
  elif [ "$DSHA" != "$OSHA" ]; then
    fail "(A-SK36) sha256(OUT)=$OSHA != .done sha256=$DSHA — OUT is not the file the battery completed"
  else
    # A-SK41 (v4): the stamp must be in the COMMITTED battery-stamps
    # ledger — batteries from 86pre on append it at run time. Batteries
    # older than the ledger (rev not reachable from a ledgered line) are
    # pre-v4 evidence: refuse, re-run the battery.
    # v5 A-AH43: ONE source of proof — the canonical path resolved with
    # the git prefix; never a concatenation of two `git show` outputs (a
    # tracked twin at the git root could otherwise certify alone).
    # v5 A-SK46: the committed line must match ALL FOUR fields, and the
    # matrix archive it names must be COMMITTED with a matching sha.
    BLEDGER_REL="wp83-harness/evidence/battery-stamps.ledger"
    DMTX=$(sed -n 's/^rev=[0-9a-f]* sha256=[0-9a-f]* matrix=\([^ ]*\) matrix_sha256=\([0-9a-f]*\).*/\1 \2/p' "$DONE" | head -1)
    DMTX_NAME="${DMTX%% *}"; DMTX_SHA="${DMTX#* }"
    if [ -z "$DMTX_NAME" ] || [ -z "$DMTX_SHA" ] || [ "$DMTX_NAME" = "$DMTX_SHA" ]; then
      fail "(A-SK46) .done carries no matrix=/matrix_sha256= pair (A-AH40 stamp incomplete)"
    else
      if ! git -C "$REPO" show "HEAD:${GITPREFIX}${BLEDGER_REL}" 2>/dev/null | \
           grep -q "rev=$BREV sha256=$DSHA matrix=$DMTX_NAME matrix_sha256=$DMTX_SHA"; then
        fail "(A-SK41/A-SK46) 4-field stamp rev=$BREV sha256=$DSHA matrix=$DMTX_NAME matrix_sha256=$DMTX_SHA not in the COMMITTED ${GITPREFIX}${BLEDGER_REL} (canonical path, A-AH43) — battery not ledgered (KS-SK-87-1/KS-SK-88-1)"
      fi
      MTX_REL="wp78-harness/matrix-archive/$DMTX_NAME"
      MTX_COMMITTED_SHA=$(git -C "$REPO" show "HEAD:${GITPREFIX}${MTX_REL}" 2>/dev/null | shasum -a 256 | cut -d' ' -f1)
      if [ -z "$MTX_COMMITTED_SHA" ] || ! git -C "$REPO" cat-file -e "HEAD:${GITPREFIX}${MTX_REL}" 2>/dev/null; then
        fail "(A-SK46) matrix archive $DMTX_NAME not COMMITTED at HEAD — the stamp's matrix is unanchored"
      elif [ "$MTX_COMMITTED_SHA" != "$DMTX_SHA" ]; then
        fail "(A-SK46) committed matrix sha $MTX_COMMITTED_SHA != .done matrix_sha256=$DMTX_SHA"
      else
        # A-AH44: toolchain comparator — the archive records the compiling
        # rustc; the consuming campaign must run the SAME one, or the
        # binary identity argument breaks BY NAME (KS-AH-87-2 mechanical).
        # v6 A-AH47/KS-AH-89-2: EXACTLY one rustc= row (head -1 on a
        # multi-row archive judged the first in silence); comparator
        # samples IN-REPO with -Vv (host triple included) to match the
        # recorder. Pre-v6 single-line `-V` archives fail the equality
        # against the richer sample BY NAME — re-run the battery.
        NRUSTC=$(git -C "$REPO" show "HEAD:${GITPREFIX}${MTX_REL}" | tr -d '\0' | grep -c '^rustc=' || true)
        MTX_RUSTC=$(git -C "$REPO" show "HEAD:${GITPREFIX}${MTX_REL}" | tr -d '\0' | sed -n 's/^rustc=//p' | head -1)
        CUR_RUSTC=$( (cd "$REPO" && rustc -Vv 2>/dev/null | tr '\n' ';') )
        if [ "$NRUSTC" != 1 ]; then
          fail "(A-AH47) matrix archive carries $NRUSTC rustc= rows, expected exactly 1 (KS-AH-89-2)"
        elif [ -z "$MTX_RUSTC" ]; then
          # A-AH53 (Council WP-90): correct diagnosis — the header EXISTS
          # (NRUSTC==1 above), it is the VALUE that is empty (masked
          # recorder failure, now refused at record time too).
          fail "(A-AH44/A-AH53) matrix rustc= header present but VALUE EMPTY — recorder sampled a broken toolchain; re-run the battery"
        elif [ "$MTX_RUSTC" != "$CUR_RUSTC" ]; then
          fail "(A-AH44/A-AH47) toolchain drift: matrix rustc='$MTX_RUSTC' != current in-repo rustc -Vv='$CUR_RUSTC' — battery certifies ANOTHER compiler (KS-AH-87-2/KS-AH-89-1)"
        fi
        # A-AH55 (Council WP-91): cargo= declared at least ADVISORY —
        # value empty/unknown makes the matrix NULL for cargo-citing
        # identities (KS-AH-91-3); a missing row is a declared pre-v7
        # archive (ADVISORY, not silent); drift is ADVISORY (rustc= stays
        # the primary judge).
        NCARGO=$(git -C "$REPO" show "HEAD:${GITPREFIX}${MTX_REL}" | tr -d '\0' | grep -c '^cargo=' || true)
        MTX_CARGO=$(git -C "$REPO" show "HEAD:${GITPREFIX}${MTX_REL}" | tr -d '\0' | sed -n 's/^cargo=//p' | head -1)
        if [ "$NCARGO" = 0 ]; then
          echo "ADVISORY (A-AH55): matrix archive carries no cargo= row (pre-v7 recorder) — cargo identity unjudged"
        elif [ -z "$MTX_CARGO" ] || [ "$MTX_CARGO" = "unknown" ]; then
          fail "(A-AH55/KS-AH-91-3) matrix cargo= is empty/unknown — matrix NULL for cargo-citing identities; re-run the battery"
        else
          CUR_CARGO=$( (cd "$REPO" && cargo -V 2>/dev/null) || true)
          [ "$MTX_CARGO" = "$CUR_CARGO" ] || echo "ADVISORY (A-AH55): cargo drift matrix='$MTX_CARGO' vs current='$CUR_CARGO' (rustc= is the primary judge)"
        fi
      fi
    fi
  fi
fi

for entry in $GATES; do
  name="${entry%%:*}"; objects="${entry#*:}"
  # A-SK32/KS-SK-85-1: a FAILED gate can never be sold as "absent with
  # reason" — refuse outright, battery must re-run.
  if grep -qE "^FAIL[[:space:]]+$name(:|[[:space:]]|\$)" "$OUT"; then
    fail "(ii) gate '$name' FAILED in the battery — a FAIL is never 'absent' (KS-SK-85-1)"
    continue
  fi
  if grep -qE "^OK[[:space:]]+$name( |\$)" "$OUT"; then
    # (iv) present-and-OK but its OBJECT (script or a named transcluded
    # helper/fixture, A-AH36) changed since B: the recorded OK certifies
    # the OLD object — refuse the equivalence for this gate.
    IFS=',' read -ra OBJS <<< "$objects"
    for obj in "${OBJS[@]}"; do
      if object_changed "$obj"; then
        fail "(iv) gate '$name' OK at $BREV but its object changed since ($obj) — re-run, don't equivalence (KS-AH-85-2)"
      fi
    done
    continue
  fi
  # not OK in the battery output: must be in the caller's named-absent set
  reason=""
  for a in "$@"; do
    case "$a" in "$name":*) reason="${a#*:}";; esac
  done
  if [ -z "$reason" ]; then
    fail "(ii) gate '$name' neither OK in battery nor in the NAMED absent set (KS-SK-84-1)"
  else
    case " $NEVER_ABSENT " in
      *" $name "*) fail "(ii) gate '$name' may NEVER be absent (A-SK30)";;
      *) echo "note: gate '$name' ABSENT by name, reason: $reason";;
    esac
  fi
done

# (iii) one equivalence per chain, CANONICAL ledger only (A-SK33):
# tracked in git, working copy identical to HEAD's version before the claim
# (a fresh/untracked/regenerated ledger erases the anti-transitive history —
# KS-SK-85-2: every equivalence of the chain VOID).
# v5 A-SK46: same-rev consumption is NOT an equivalence — tooth (iii) and
# the append do not apply; the verdict line names the mode.
# A-AH61 (Council WP-93, Hejlsberg — KS-AH-93-1): the attempts discipline
# (A-AH50/54 PASS-row + triangle, A-AH58/59 grammar v2) used to live ONLY
# inside the --same-rev branch: a consumption in EQUIVALENCE mode of a 9x
# battery skipped writer=, anchors and the attempts triangle — «la
# delibera dice alla consumazione, il codice diceva alla consumazione
# same-rev». Hoisted HERE so BOTH consumption paths bite.
# A-AH50≡A-BG49 (Council WP-90, KS-AH-90-1/KG-90-1): the consumed PASS
# must have its own row in the COMMITTED battery-attempts ledger — an
# attempt that left no in-band row makes this PASS non-consumable.
# Scoped to 89pre+ batteries (the ledger was born at WP-89; historical
# 88pre verdicts stand as judged, they are never re-consumed).
ATTL_REL="wp83-harness/evidence/battery-attempts.ledger"
BATTERY_NAME=$(basename "$OUT" .out)
case "$BATTERY_NAME" in
  battery-8[0-8]*) : ;;  # pre-ledger batteries, declared exempt
  *)
    NATT=$(git -C "$REPO" show "HEAD:${GITPREFIX}${ATTL_REL}" 2>/dev/null | grep -c "battery=${BATTERY_NAME#battery-} rev=$BREV .*esito=PASS" || true)
    if [ "$NATT" -lt 1 ]; then
      fail "(A-AH50/KS-AH-90-1) no committed esito=PASS row for $BATTERY_NAME rev=$BREV in battery-attempts.ledger — attempt not in-band, PASS non-consumable"
    else
      # A-AH54 triangle (Council WP-91, Hejlsberg — KS-AH-91-1): the
      # consumed PASS row must carry sha256 == DSHA (the stamp's
      # sha256(OUT), itself recomputed against OUT above): the
      # attempts↔stamp↔OUT triangle closes — a fabricated PASS row
      # with a foreign or absent sha no longer feeds the tooth.
      NATT_SHA=$(git -C "$REPO" show "HEAD:${GITPREFIX}${ATTL_REL}" 2>/dev/null | grep -c "battery=${BATTERY_NAME#battery-} rev=$BREV .*esito=PASS.*sha256=${DSHA:-__nodsha__}" || true)
      if [ "$NATT_SHA" -lt 1 ]; then
        fail "(A-AH54/KS-AH-91-1) committed PASS row for $BATTERY_NAME rev=$BREV carries NO sha256==DSHA($DSHA) — attempts↔stamp↔OUT triangle open, consumption VOID"
      fi
    fi
    ;;
esac
# A-AH58/A-AH59 (Council WP-92, DELIBERA UNICA di formato ledger —
# wp91-harness/design91-ledger.md): grammar v2 enforced on 91pre+
# batteries, ALL rows of the battery family (KS-AH-92-1/2). 89pre/90pre
# rows stand as written (grammar v1, declared — history is never
# re-graded). A-AH61: the family pattern covers 3-digit batteries too
# (battery-9[1-9]* left 100pre+ out of the discipline by name).
case "$BATTERY_NAME" in
  battery-9[1-9]*|battery-[1-9][0-9][0-9]*)
    BNAME="${BATTERY_NAME#battery-}"
    V2ROWS=$(git -C "$REPO" show "HEAD:${GITPREFIX}${ATTL_REL}" 2>/dev/null | grep "battery=$BNAME " || true)
    if [ -n "$V2ROWS" ]; then
      BADW=$(printf '%s\n' "$V2ROWS" | grep -cvE "writer=(script:[0-9a-f]{16}|operator)( |$)" || true)
      [ "$BADW" -gt 0 ] && fail "(A-AH58/KS-AH-92-1) $BADW attempts row(s) for battery=$BNAME without a valid writer= — consumption VOID"
      BADA=$(printf '%s\n' "$V2ROWS" | grep "esito=ABORT" | grep -cv "writer=operator" || true)
      [ "$BADA" -gt 0 ] && fail "(A-AH58/KS-AH-92-1) $BADA esito=ABORT row(s) without writer=operator — an ABORT is an operator act"
      BADE=$(printf '%s\n' "$V2ROWS" | grep -cvE "esito=(PASS|FAIL|REFUSE|ABORT)( |$)" || true)
      [ "$BADE" -gt 0 ] && fail "(A-AH58) $BADE row(s) with esito outside {PASS,FAIL,REFUSE,ABORT}"
      # A-AH61: sha256 ANCHORED — the unanchored {64} accepted 65 hex.
      BADS=$(printf '%s\n' "$V2ROWS" | grep -E "esito=(FAIL|REFUSE|ABORT)" | grep -cvE "sha256=[0-9a-f]{64}( |\$)" || true)
      [ "$BADS" -gt 0 ] && fail "(A-AH59/KS-AH-92-2) $BADS FAIL/REFUSE/ABORT row(s) without a sha256 OUT anchor — battery VOID (never 'assente con motivo')"
      # A-PP-68 (Council WP-93, KS-PP-93-2): the trap's temporal semantics
      # is a FIELD — an ABORT row without gate_in_flight=/deferred= is VOID.
      BADG=$(printf '%s\n' "$V2ROWS" | grep "esito=ABORT" | grep -cvE "gate_in_flight=[^ ]+ deferred=[01]( |\$)" || true)
      [ "$BADG" -gt 0 ] && fail "(A-PP-68/KS-PP-93-2) $BADG esito=ABORT row(s) without gate_in_flight=/deferred= fields"
    fi
    ;;
esac

if [ "$SAME_REV" = 1 ]; then
  # v6 A-SK50 (Council WP-89, Klabnik — tooth (i-bis)): the evidence-only
  # window BREV..HEAD was BLIND on three surfaces: the checker itself, the
  # ledger+matrix (forgeable in one commit on paths no tooth watched), and
  # the gate-measure-cifre .out corpus. Now the WHOLE delta must be a
  # SUBSET of a named allowlist (KS-SK-89-1):
  #   - the battery-stamps ledger (append-only, prefix-checked below);
  #   - matrix-archive/ but ONLY the archive named by .done;
  #   - measure-out/ raws (VOIDed attempts of the same campaign).
  # Anything else — checker/verdict/campaign scripts, harness .out files,
  # session docs — refuses the consumption: those belong AFTER the verdict,
  # not inside the evidence window.
  BLEDGER_REL="wp83-harness/evidence/battery-stamps.ledger"
  # A-AH50 (Council WP-90): the battery-attempts ledger writes INSIDE the
  # evidence window BY CONSTRUCTION (every attempt rows there before the
  # stamp commit) — it belongs to the window allowlist like the stamps
  # ledger. Bitten in S-89.0: the first stamp commit carrying the PASS
  # row would have been refused as a non-allowlisted delta.
  ATTLEDGER_REL="wp83-harness/evidence/battery-attempts.ledger"
  FULLDELTA=$(git -C "$REPO" diff --name-only "$BREV..HEAD" 2>/dev/null)
  if [ -n "$FULLDELTA" ]; then
    BAD_DELTA=$(echo "$FULLDELTA" | grep -vE "^${GITPREFIX}(${BLEDGER_REL}|${ATTLEDGER_REL}|wp78-harness/matrix-archive/|wp78-harness/measure-out/)" || true)
    if [ -n "$BAD_DELTA" ]; then
      echo "$BAD_DELTA" | head -5
      fail "(A-SK50) same-rev window $BREV..$HEADREV touches NON-allowlisted paths — evidence-only window violated (KS-SK-89-1)"
    fi
    # matrix-archive delta may contain ONLY the archive the .done names
    BAD_MTX=$(echo "$FULLDELTA" | grep -E "^${GITPREFIX}wp78-harness/matrix-archive/" | grep -vxF "${GITPREFIX}wp78-harness/matrix-archive/${DMTX_NAME:-__none__}" || true)
    if [ -n "$BAD_MTX" ]; then
      echo "$BAD_MTX" | head -5
      fail "(A-SK50) same-rev window commits a matrix archive NOT named by .done (KS-SK-89-1)"
    fi
  fi
  # ledger@BREV must be a PREFIX of ledger@HEAD (append-only in-window:
  # a rewrite that keeps the stamp but edits history escapes the 4-field
  # grep — the prefix check kills it).
  # A-SK57 (Council WP-90, Klabnik): prefix at ROW granularity — a bare
  # prefix test lets an EXTENSION of the last committed line pass as an
  # append ("rev=abc" -> "rev=abcX" is a rewrite). The remainder must be
  # empty or START with a newline (command substitution strips the old
  # trailing newline, so a legal append remainder is "\n<rows>").
  BL_OLD=$(git -C "$REPO" show "$BREV:${GITPREFIX}${BLEDGER_REL}" 2>/dev/null)
  BL_NEW=$(git -C "$REPO" show "HEAD:${GITPREFIX}${BLEDGER_REL}" 2>/dev/null)
  BL_REM="${BL_NEW#"$BL_OLD"}"
  if [ -n "$BL_OLD" ] && [ "$BL_REM" = "$BL_NEW" ] && [ "$BL_OLD" != "$BL_NEW" ]; then
    fail "(A-SK50) battery-stamps ledger at $BREV is NOT a prefix of HEAD's — in-window rewrite (KS-SK-89-1)"
  elif [ -n "$BL_OLD" ] && [ -n "$BL_REM" ] && [ "${BL_REM#$'\n'}" = "$BL_REM" ]; then
    fail "(A-SK57) ledger delta EXTENDS the last committed row — row-granularity append violated (KS-SK-89-1)"
  fi
  # A-AH54 (Council WP-91, Hejlsberg — KS-AH-91-1): the battery-attempts
  # ledger, once allowlisted in-window (A-SK50 emendation), was
  # REWRITABLE in-window: delete a FAIL/REFUSE, fabricate the PASS row
  # the A-AH50 tooth demands, and no tooth bit. Same A-SK57 discipline,
  # row granularity: prefix at BREV + remainder empty-or-newline-first.
  AL_OLD=$(git -C "$REPO" show "$BREV:${GITPREFIX}${ATTLEDGER_REL}" 2>/dev/null)
  AL_NEW=$(git -C "$REPO" show "HEAD:${GITPREFIX}${ATTLEDGER_REL}" 2>/dev/null)
  AL_REM="${AL_NEW#"$AL_OLD"}"
  if [ -n "$AL_OLD" ] && [ "$AL_REM" = "$AL_NEW" ] && [ "$AL_OLD" != "$AL_NEW" ]; then
    fail "(A-AH54) battery-attempts ledger at $BREV is NOT a prefix of HEAD's — in-window rewrite (KS-AH-91-1)"
  elif [ -n "$AL_OLD" ] && [ -n "$AL_REM" ] && [ "${AL_REM#$'\n'}" = "$AL_REM" ]; then
    fail "(A-AH54) attempts-ledger delta EXTENDS the last committed row — row-granularity append violated (KS-AH-91-1)"
  fi
  # (A-AH61: the attempts discipline — A-AH50/54 + grammar v2 — is HOISTED
  # above the branch: it bites on BOTH consumption paths now.)
  if [ "$FAILS" = 0 ]; then
    echo "== SAME-REV CONSUMPTION LEGAL (battery at $BREV == HEAD, v6 teeth verified: anchored PASS, sha256(OUT), 4-field committed stamp, committed matrix, toolchain -Vv in-repo, window allowlist + ledger-prefix A-SK50) =="
    exit 0
  else
    echo "== SAME-REV CONSUMPTION REFUSED ($FAILS) — battery must re-run (KS-SK-88-1/KS-SK-89-1) =="
    exit 1
  fi
fi
if [ ! -f "$LEDGER" ]; then
  fail "(iii) canonical ledger missing at $LEDGER_REL (KS-SK-85-2 — never touch-create it)"
elif ! git -C "$REPO" ls-files --error-unmatch "$LEDGER_REL" >/dev/null 2>&1; then
  fail "(iii) canonical ledger is UNTRACKED (KS-SK-85-2)"
elif ! git -C "$REPO" diff --quiet HEAD -- "$LEDGER_REL" 2>/dev/null; then
  fail "(iii) working ledger DIVERGES from HEAD's version — history not canonical (KS-SK-85-2)"
elif ! git -C "$REPO" diff --quiet --cached HEAD -- "$LEDGER_REL" 2>/dev/null; then
  # A-SK37 (Council WP-86): the worktree==HEAD!=index corner — a staged
  # ledger edit restored in the worktree would evade the HEAD diff above.
  fail "(iii) INDEX ledger diverges from HEAD (staged edit) — history not canonical (A-SK37/KS-SK-85-2)"
fi
if [ -f "$LEDGER" ]; then
  if grep -q "^battery_rev=$BREV " "$LEDGER"; then
    fail "(iii) an equivalence on battery rev $BREV is ALREADY ledgered — one per chain (KS-AH-84-3)"
  fi
  if grep -q " head=$BREV\$" "$LEDGER"; then
    fail "(iii) rev $BREV was itself certified BY equivalence — transitive chain refused (KS-AH-84-3)"
  fi
fi

if [ "$FAILS" = 0 ]; then
  echo "battery_rev=$BREV head=$HEADREV" >> "$LEDGER"
  # A-SK37 (Council WP-86): the claim is PROVISIONAL until the append is
  # COMMITTED — an uncommitted append vanishes with a worktree restore and
  # re-opens one-per-chain. The campaign must commit it and CITE the
  # commit-id next to any equivalence claim (KS-SK-86-2: append not at
  # HEAD at campaign close => every equivalence of the campaign VOID).
  echo "== EQUIVALENCE LEGAL ($BREV certifies $HEADREV) — PROVISIONAL: commit the ledger append NOW and cite the commit-id in the claim (A-SK37/KS-SK-86-2) =="
  exit 0
else
  echo "== EQUIVALENCE REFUSED ($FAILS) — battery must re-run (KS-SK-84-1/KS-AH-84-3/KS-SK-85-1/KS-SK-85-2) =="
  exit 1
fi
