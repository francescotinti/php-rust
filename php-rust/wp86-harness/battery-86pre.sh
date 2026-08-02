#!/bin/bash
# battery-86pre.sh — S-86.0: FULL battery on the WP-86 tree, sequential
# (project rule). Verdict per gate BY NAME (`OK <name>`). v4 changes
# (Council WP-87):
#   A-SK42: (a) `git status --porcelain` FAIL-CLOSED at the top — a battery
#     on a dirty tree stamps a rev that is not the compiled source
#     (KS-SK-87-1: battery VOID); (b) the terminal stamp counts k/k — the
#     v3 battery printed "(15/15)" HARDWIRED (with 14 gates in the list it
#     would still have said 15/15).
#   A-SK41: the PASS stamp (rev+sha256(OUT)) is APPENDED to the canonical
#     TRACKED evidence ledger (same law as A-SK33/A-SK37) — the claim is
#     PROVISIONAL until the append is committed; battery-equivalence v4
#     compares against the COMMITTED stamp (an OUT+.done pair forged
#     outside a battery run has no ledgered stamp).
#   A-AH40: the `.done` names WHICH matrix archive this battery's own
#     feature-matrix run produced (`matrix=<basename> matrix_sha256=<sha>`)
#     — campaigns resolve the archive FROM THE .done, never by `tail -1`
#     (two same-rev archives no longer break the battery⊃matrix chain in
#     silence; KS-AH-87-1).
#   (A-SK40/A-SK36 v3 behaviors retained: cifre --all in perimeter, OUT
#   self-written, .done only on PASS with recomputed sha256.)
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
# OUTSIDE the repo (whole-tree porcelain rule).
W="/Volumes/Extreme Pro/Claude/wp86-battery-out"
mkdir -p "$W"
OUTF="$W/battery-86pre.out"
: > "$OUTF"
# A-SK36: a stale .done must never survive a re-run that fails.
rm -f "$W/.done"
FAILS=0
TOTAL=0
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
say() { echo "$@" | tee -a "$OUTF"; }

# A-SK42(a): porcelain fail-closed BEFORE any gate runs.
DIRTY="$(git -C "$REPO" status --porcelain 2>/dev/null)"
if [ -n "$DIRTY" ]; then
  say "FAIL: tree not porcelain at battery start — battery VOID (A-SK42/KS-SK-87-1):"
  echo "$DIRTY" | head -10 | tee -a "$OUTF"
  say "== BATTERY-86PRE REFUSED git=$GIT_REV =="
  exit 1
fi

say "== battery-86pre git=$GIT_REV =="
run_gate() { # <label> <cmd...>
  local label="$1"; shift
  TOTAL=$((TOTAL+1))
  say "== $label =="
  if "$@" > "$W/$label.log" 2>&1; then
    say "OK  $label ($(tail -1 "$W/$label.log"))"
  else
    say "FAIL $label:"; tail -5 "$W/$label.log" | tee -a "$OUTF"
    FAILS=$((FAILS+1))
  fi
}

# A-AH40: snapshot the matrix archive dir so THIS battery's own
# feature-matrix run is identified by set-difference, never by mtime order.
MTXDIR="$REPO/wp78-harness/matrix-archive"
MTX_BEFORE="$W/.mtx-before"
/bin/ls "$MTXDIR" > "$MTX_BEFORE" 2>/dev/null || : > "$MTX_BEFORE"

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

# A-AH40: exactly ONE new archive must exist (the one this run produced).
NEWMTX=$(/bin/ls "$MTXDIR" | comm -13 "$MTX_BEFORE" - )
NMTX=$(printf '%s' "$NEWMTX" | grep -c . || true)
if [ "$NMTX" != 1 ]; then
  say "FAIL: matrix archives created by this battery == $NMTX, expected exactly 1 (A-AH40)"
  FAILS=$((FAILS+1))
fi

PASSN=$((TOTAL-FAILS))
if [ "$FAILS" = 0 ]; then
  say "== BATTERY-86PRE PASS ($PASSN/$TOTAL) git=$GIT_REV =="
  SHA=$(shasum -a 256 "$OUTF" | cut -d' ' -f1)
  MTX_SHA=$(shasum -a 256 "$MTXDIR/$NEWMTX" | cut -d' ' -f1)
  echo "rev=$GIT_REV sha256=$SHA matrix=$NEWMTX matrix_sha256=$MTX_SHA" > "$W/.done"
  # A-SK41: ledger the stamp in the TRACKED canonical evidence file. The
  # claim is PROVISIONAL until this append is COMMITTED (A-SK37 law) —
  # commit it NOW and cite the commit-id.
  LEDGER="$REPO/wp83-harness/evidence/battery-stamps.ledger"
  echo "battery=86pre rev=$GIT_REV sha256=$SHA matrix=$NEWMTX matrix_sha256=$MTX_SHA" >> "$LEDGER"
  # STDOUT ONLY — never say/tee: the anchored PASS line must stay the
  # FINAL line of OUT (A-SK36; the first 86pre run violated this and the
  # checker refused its own battery — the tooth worked).
  echo "stamp ledgered (A-SK41): commit $LEDGER NOW — the claim is PROVISIONAL until the append is at HEAD"
else
  say "== BATTERY-86PRE FAIL($FAILS/$TOTAL) git=$GIT_REV =="
fi
exit $FAILS
