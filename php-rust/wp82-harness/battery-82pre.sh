#!/bin/bash
# battery-82pre.sh — S-82.0: FULL battery on the consolidated pre-measure
# tree (after p1-p6), sequential (project rule). Every gate must PASS
# before p7 measures; verdict per gate BY NAME.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
# OUTSIDE the repo: the feature-matrix gate refuses ANY dirty/untracked path
# (git status --porcelain over the WHOLE tree, A-AH14) — the battery's own
# logs must not trip it. Summary + key logs are copied in and committed at
# session close (evidence discipline unchanged).
W="/Volumes/Extreme Pro/Claude/wp82-battery-out"
mkdir -p "$W"
FAILS=0
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
run_gate parity-full      bash "$HERE/gate-parity-82p2.sh"

if [ "$FAILS" = 0 ]; then echo "== BATTERY-82PRE PASS (15/15) =="; else echo "== BATTERY-82PRE FAIL($FAILS) =="; fi
touch "$W/.done"
exit $FAILS
