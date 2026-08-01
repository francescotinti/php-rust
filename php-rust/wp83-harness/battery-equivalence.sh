#!/bin/bash
# battery-equivalence.sh — A-SK30/A-AH34 (Council WP-84): KH82-2 REWRITTEN.
# The old form (`grep -cE '^OK'` >= 14 + matrix-at-HEAD) was refuted twice:
# a count admits one always-FAIL gate never named (Klabnik Q3, KS-SK-84-1),
# and the matrix at HEAD certifies the BUILD, not the equivalence
# (Hejlsberg Q4). An equivalence claim ("the battery at rev B certifies
# HEAD") is legal IFF, all computed HERE:
#   (i)   `git diff B..HEAD -- crates/ Cargo.toml Cargo.lock` is EMPTY;
#   (ii)  every expected gate is present as `OK <name>` BY NAME in the
#         battery output; gates ABSENT form a NAMED set with a caller-
#         supplied reason — and {parity-full, lever-fixtures,
#         lever-fixtures2, measure-cifre} may NEVER be absent;
#   (iii) at most ONE equivalence per chain: ledger line per battery rev —
#         a second claim on the same rev REFUSES (never transitive: B must
#         be an ancestor whose battery RAN, not itself an equivalence);
#   (iv)  never for a gate whose SCRIPT is among the changed files B..HEAD
#         (its object changed: re-run it, don't equivalence it).
# Usage:
#   battery-equivalence.sh <battery.out> <battery_rev> <ledger-file> \
#       [absent-gate:reason ...]
# Exit 0 = equivalence LEGAL; !=0 = precondition FAIL (campaign VOID,
# KS-SK-84-1/KS-AH-84-3).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="${1:?battery.out}"; BREV="${2:?battery rev}"; LEDGER="${3:?ledger file}"
shift 3
FAILS=0
fail() { echo "FAIL: $*"; FAILS=$((FAILS+1)); }

HEADREV=$(git -C "$REPO" rev-parse --short HEAD)

# The named gate set (mirror of battery-83pre.sh) with each gate's SCRIPT
# for tooth (iv). Bump = same-commit, named.
GATES="feature-matrix:wp78-harness/gate-feature-matrix.sh
census-twin:wp78-harness/gate-census-twin.sh
doc-purge:wp78-harness/gate-doc-purge.sh
capture-order:wp78-harness/gate-capture-order.sh
concurrent:wp78-harness/gate-concurrent.sh
stdout-tandem:wp78-harness/gate-stdout-tandem.sh
worker-panic:wp78-harness/gate-worker-panic.sh
run-gate-cli:wp77-harness/run_gate_g_apertura_2.sh
run-gate-axum:wp77-harness/run_gate_g_apertura_2_axum.sh
dr1:wp80-harness/gate-dr1-module-immut.sh
lever-pins:wp81-harness/gate-lever-pins.sh
lever-fixtures:wp81-harness/gate-lever-fixtures.sh
lever-fixtures2:wp81-harness/gate-lever-fixtures2.sh
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

# (ii) per-NAME presence; absent = named set with reason
[ -f "$OUT" ] || { fail "(ii) battery output missing: $OUT"; echo "== EQUIVALENCE REFUSED =="; exit 1; }
for entry in $GATES; do
  name="${entry%%:*}"; script="${entry#*:}"
  if grep -qE "^OK[[:space:]]+$name( |\$)" "$OUT"; then
    # (iv) present-and-OK but its SCRIPT changed since B: the recorded OK
    # certifies the OLD object — refuse the equivalence for this gate.
    if echo "$CHANGED" | grep -qxF "php-rust/$script" || echo "$CHANGED" | grep -qxF "$script"; then
      fail "(iv) gate '$name' OK at $BREV but its script changed since ($script) — re-run, don't equivalence"
    fi
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

# (iii) one equivalence per chain, ledger-enforced; never transitive
mkdir -p "$(dirname "$LEDGER")"
touch "$LEDGER"
if grep -q "^battery_rev=$BREV " "$LEDGER"; then
  fail "(iii) an equivalence on battery rev $BREV is ALREADY ledgered — one per chain (KS-AH-84-3)"
fi
if grep -q " head=$BREV\$" "$LEDGER"; then
  fail "(iii) rev $BREV was itself certified BY equivalence — transitive chain refused (KS-AH-84-3)"
fi

if [ "$FAILS" = 0 ]; then
  echo "battery_rev=$BREV head=$HEADREV" >> "$LEDGER"
  echo "== EQUIVALENCE LEGAL ($BREV certifies $HEADREV; ledgered) =="
  exit 0
else
  echo "== EQUIVALENCE REFUSED ($FAILS) — battery must re-run (KS-SK-84-1/KS-AH-84-3) =="
  exit 1
fi
