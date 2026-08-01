#!/bin/bash
# battery-85pre.sh — S-85.0: FULL battery on the WP-85 tree, sequential
# (project rule). Verdict per gate BY NAME (`OK <name>`). v3 changes
# (Council WP-86):
#   A-SK40: the cifre gate runs in --all mode — EVERY MEASURE8[4-9] doc is
#     in the 15/15 perimeter (the WP-84 battery ran the bare default, which
#     targets MEASURE81 only: MEASURE84 was never covered — the GRAVE hole).
#   A-SK36: the battery WRITES ITS OWN OUT file (tee) and stamps `.done`
#     ONLY on PASS, with rev=<rev> sha256=<sha256(OUT)> — the equivalence
#     checker (battery-equivalence.sh v3) recomputes the sha and requires
#     the anchored terminal PASS line.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
# OUTSIDE the repo (whole-tree porcelain rule).
W="/Volumes/Extreme Pro/Claude/wp85-battery-out"
mkdir -p "$W"
OUTF="$W/battery-85pre.out"
: > "$OUTF"
# A-SK36: a stale .done must never survive a re-run that fails.
rm -f "$W/.done"
FAILS=0
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
say() { echo "$@" | tee -a "$OUTF"; }
say "== battery-85pre git=$GIT_REV =="
run_gate() { # <label> <cmd...>
  local label="$1"; shift
  say "== $label =="
  if "$@" > "$W/$label.log" 2>&1; then
    say "OK  $label ($(tail -1 "$W/$label.log"))"
  else
    say "FAIL $label:"; tail -5 "$W/$label.log" | tee -a "$OUTF"
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
run_gate measure-cifre    bash "$REPO/wp81-harness/gate-measure-cifre.sh" --all
run_gate parity-full      bash "$REPO/wp83-harness/gate-parity-83p1.sh"

if [ "$FAILS" = 0 ]; then
  say "== BATTERY-85PRE PASS (15/15) git=$GIT_REV =="
  SHA=$(shasum -a 256 "$OUTF" | cut -d' ' -f1)
  echo "rev=$GIT_REV sha256=$SHA" > "$W/.done"
else
  say "== BATTERY-85PRE FAIL($FAILS) git=$GIT_REV =="
fi
exit $FAILS
