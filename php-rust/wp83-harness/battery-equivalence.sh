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
# Usage (v2 — the ledger is NO LONGER a parameter, A-SK33):
#   battery-equivalence.sh <battery.out> <battery_rev> [absent-gate:reason ...]
# Exit 0 = equivalence LEGAL; !=0 = precondition FAIL (campaign VOID,
# KS-SK-84-1/KS-AH-84-3/KS-SK-85-1/KS-SK-85-2).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="${1:?battery.out}"; BREV="${2:?battery rev}"
shift 2
FAILS=0
fail() { echo "FAIL: $*"; FAILS=$((FAILS+1)); }

HEADREV=$(git -C "$REPO" rev-parse --short HEAD)

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
parity-full:wp83-harness/gate-parity-83p1.sh"
NEVER_ABSENT="parity-full lever-fixtures lever-fixtures2 measure-cifre"

# (i) crates/lock delta EMPTY, computed here
DELTA=$(git -C "$REPO" diff --name-only "$BREV..HEAD" -- crates Cargo.toml Cargo.lock 2>/dev/null)
if [ -n "$DELTA" ]; then
  fail "(i) crates/Cargo delta $BREV..$HEADREV NOT empty:"; echo "$DELTA" | head -5
fi

# changed files for tooth (iv)
CHANGED=$(git -C "$REPO" diff --name-only "$BREV..HEAD" 2>/dev/null)

# object_changed <path-or-dirprefix> -> 0 if any changed file matches it
# (git -C <subdir> emits paths relative to the GIT root, so the php-rust/
# prefixed form is the operative one; the bare form is an inert fallback).
object_changed() {
  local obj="$1"
  case "$obj" in
    */) echo "$CHANGED" | grep -qE "^(php-rust/)?${obj}" ;;
    *)  echo "$CHANGED" | grep -qxF "php-rust/$obj" || echo "$CHANGED" | grep -qxF "$obj" ;;
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
  DSHA=$(sed -n 's/.*sha256=\([0-9a-f]*\).*/\1/p' "$DONE" | head -1)
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
    BLEDGER_REL="wp83-harness/evidence/battery-stamps.ledger"
    if ! git -C "$REPO" show "HEAD:$BLEDGER_REL" 2>/dev/null | \
         grep -q "rev=$BREV sha256=$DSHA"; then
      fail "(A-SK41) stamp rev=$BREV sha256=$DSHA not in the COMMITTED $BLEDGER_REL — battery not ledgered (KS-SK-87-1)"
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
