#!/bin/bash
# battery-87pre.sh — S-87.0: FULL battery on the WP-87 tree, sequential
# (project rule). Verdict per gate BY NAME (`OK <name>`). v4 behaviors
# retained (A-SK42 porcelain fail-closed + k/k counted, A-SK41 stamp
# ledgered, A-AH40 matrix in .done, A-SK40 cifre --all, A-SK36 anchored
# terminal PASS). v5 changes (Council WP-88):
#   A-AH45: the matrix-archive snapshot is compared in BOTH directions —
#     `comm -13` (additions: exactly one, this battery's own) AND
#     `comm -23` (deletions: exactly zero). A deletion/substitution
#     mid-battery let an intruder archive get NAMED in the .done while the
#     gate's own product was gone (KS-AH-88-2: battery VOID).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
# OUTSIDE the repo (whole-tree porcelain rule).
W="/Volumes/Extreme Pro/Claude/wp87-battery-out"
mkdir -p "$W"
OUTF="$W/battery-87pre.out"
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
  say "== BATTERY-87PRE REFUSED git=$GIT_REV =="
  exit 1
fi

say "== battery-87pre git=$GIT_REV =="
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

# A-AH40/A-AH45: snapshot the matrix archive dir — this battery's own
# feature-matrix run is identified by set-difference in BOTH directions.
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
MTX_AFTER="$W/.mtx-after"
/bin/ls "$MTXDIR" > "$MTX_AFTER" 2>/dev/null || : > "$MTX_AFTER"
NEWMTX=$(comm -13 "$MTX_BEFORE" "$MTX_AFTER")
NMTX=$(printf '%s' "$NEWMTX" | grep -c . || true)
if [ "$NMTX" != 1 ]; then
  say "FAIL: matrix archives created by this battery == $NMTX, expected exactly 1 (A-AH40)"
  FAILS=$((FAILS+1))
fi
# A-AH45: ZERO deletions — an archive that vanished mid-battery means the
# .done could name an intruder (KS-AH-88-2).
DELMTX=$(comm -23 "$MTX_BEFORE" "$MTX_AFTER")
NDEL=$(printf '%s' "$DELMTX" | grep -c . || true)
if [ "$NDEL" != 0 ]; then
  say "FAIL: $NDEL matrix archive(s) DELETED mid-battery (A-AH45/KS-AH-88-2):"
  printf '%s\n' "$DELMTX" | head -3 | tee -a "$OUTF"
  FAILS=$((FAILS+1))
fi

PASSN=$((TOTAL-FAILS))
if [ "$FAILS" = 0 ]; then
  say "== BATTERY-87PRE PASS ($PASSN/$TOTAL) git=$GIT_REV =="
  SHA=$(shasum -a 256 "$OUTF" | cut -d' ' -f1)
  MTX_SHA=$(shasum -a 256 "$MTXDIR/$NEWMTX" | cut -d' ' -f1)
  echo "rev=$GIT_REV sha256=$SHA matrix=$NEWMTX matrix_sha256=$MTX_SHA" > "$W/.done"
  # A-SK41: ledger the stamp in the TRACKED canonical evidence file. The
  # claim is PROVISIONAL until this append is COMMITTED (A-SK37 law) —
  # commit it NOW and cite the commit-id.
  LEDGER="$REPO/wp83-harness/evidence/battery-stamps.ledger"
  echo "battery=87pre rev=$GIT_REV sha256=$SHA matrix=$NEWMTX matrix_sha256=$MTX_SHA" >> "$LEDGER"
  # STDOUT ONLY — never say/tee: the anchored PASS line must stay the
  # FINAL line of OUT (A-SK36).
  echo "stamp ledgered (A-SK41): commit $LEDGER NOW — the claim is PROVISIONAL until the append is at HEAD"
else
  say "== BATTERY-87PRE FAIL($FAILS/$TOTAL) git=$GIT_REV =="
fi
exit $FAILS
