#!/bin/bash
# battery-83pre.sh — S-83.0: FULL battery on the consolidated pre-measure
# tree, sequential (project rule). Verdict per gate BY NAME (`OK <name>`);
# A-SK30: the battery REV is stamped in the summary header AND inside
# .done, so an equivalence claim can pin exactly which tree these gates
# certified (battery-equivalence.sh is the ONLY legal equivalence path).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
# OUTSIDE the repo (whole-tree porcelain rule).
W="/Volumes/Extreme Pro/Claude/wp83-battery-out"
mkdir -p "$W"
FAILS=0
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
echo "== battery-83pre git=$GIT_REV =="
run_gate() { # <label> <cmd...>
  local label="$1"; shift
  echo "== $label =="
  if "$@" > "$W/$label.log" 2>&1; then
    echo "OK  $label ($(tail -1 "$W/$label.log"))"
  else
    echo "FAIL $label:"; tail -5 "$W/$label.log"
    FAILS=$((FAILS+1))
  fi
}

run_gate feature-matrix   bash "$REPO/wp78-harness/gate-feature-matrix.sh"
run_gate census-twin      bash "$REPO/wp78-harness/gate-census-twin.sh"
run_gate doc-purge        bash "$REPO/wp78-harness/gate-doc-purge.sh"
run_gate capture-order    bash "$REPO/wp78-harness/gate-capture-order.sh"
run_gate concurrent       bash "$REPO/wp78-harness/gate-concurrent.sh"
run_gate stdout-tandem    bash "$REPO/wp78-harness/gate-stdout-tandem.sh"
run_gate worker-panic     bash "$REPO/wp78-harness/gate-worker-panic.sh"
run_gate run-gate-cli     bash "$REPO/wp77-harness/run_gate_g_apertura_2.sh"
run_gate run-gate-axum    bash "$REPO/wp77-harness/run_gate_g_apertura_2_axum.sh"
run_gate dr1              bash "$REPO/wp80-harness/gate-dr1-module-immut.sh"
run_gate lever-pins       bash "$REPO/wp81-harness/gate-lever-pins.sh"
run_gate lever-fixtures   bash "$REPO/wp81-harness/gate-lever-fixtures.sh"
run_gate lever-fixtures2  bash "$REPO/wp81-harness/gate-lever-fixtures2.sh"
run_gate measure-cifre    bash "$REPO/wp81-harness/gate-measure-cifre.sh"
run_gate parity-full      bash "$HERE/gate-parity-83p1.sh"

if [ "$FAILS" = 0 ]; then echo "== BATTERY-83PRE PASS (15/15) git=$GIT_REV =="; else echo "== BATTERY-83PRE FAIL($FAILS) git=$GIT_REV =="; fi
echo "rev=$GIT_REV" > "$W/.done"
exit $FAILS
