#!/bin/bash
# battery-88pre.sh — S-88.0: FULL battery on the WP-88 tree, sequential
# (project rule). Verdict per gate BY NAME (`OK <name>`). v5 behaviors
# retained (A-SK42 porcelain fail-closed + k/k counted, A-SK41 stamp
# ledgered, A-AH40 matrix in .done, A-SK40 cifre --all, A-SK36 anchored
# terminal PASS, A-AH45 bidirectional snapshot). v6 changes (Council WP-89):
#   A-PP41: gate axum-tests EXECUTES the php-server axum-server suite in
#     the battery perimeter (a_pp38 pinned by name — KS-PP-89-1 lifted).
#   A-AH48: the matrix-archive snapshot is per NAME+sha256 — a same-name
#     SWAP inside the battery window (delete own product + create intruder
#     under the same name) passed the A-AH45 endpoint comm; now any sha
#     mutation on a persistent name FAILS the battery (KS-AH-89-2).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
W="/Volumes/Extreme Pro/Claude/wp88-battery-out"
mkdir -p "$W"
OUTF="$W/battery-88pre.out"
: > "$OUTF"
rm -f "$W/.done"
FAILS=0
TOTAL=0
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
say() { echo "$@" | tee -a "$OUTF"; }

DIRTY="$(git -C "$REPO" status --porcelain 2>/dev/null)"
if [ -n "$DIRTY" ]; then
  say "FAIL: tree not porcelain at battery start — battery VOID (A-SK42/KS-SK-87-1):"
  echo "$DIRTY" | head -10 | tee -a "$OUTF"
  say "== BATTERY-88PRE REFUSED git=$GIT_REV =="
  exit 1
fi

say "== battery-88pre git=$GIT_REV =="
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

MTXDIR="$REPO/wp78-harness/matrix-archive"
MTX_BEFORE="$W/.mtx-before"
/bin/ls "$MTXDIR" > "$MTX_BEFORE" 2>/dev/null || : > "$MTX_BEFORE"
# A-AH48: per-name sha snapshot (names AND content identity).
MTX_SHA_BEFORE="$W/.mtx-sha-before"
( cd "$MTXDIR" 2>/dev/null && shasum -a 256 -- * 2>/dev/null | sort -k2 ) > "$MTX_SHA_BEFORE" || : > "$MTX_SHA_BEFORE"

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
run_gate axum-tests       bash "$REPO/wp88-harness/gate-axum-tests.sh"
run_gate parity-full      bash "$REPO/wp83-harness/gate-parity-83p1.sh"

MTX_AFTER="$W/.mtx-after"
/bin/ls "$MTXDIR" > "$MTX_AFTER" 2>/dev/null || : > "$MTX_AFTER"
NEWMTX=$(comm -13 "$MTX_BEFORE" "$MTX_AFTER")
NMTX=$(printf '%s' "$NEWMTX" | grep -c . || true)
if [ "$NMTX" != 1 ]; then
  say "FAIL: matrix archives created by this battery == $NMTX, expected exactly 1 (A-AH40)"
  FAILS=$((FAILS+1))
fi
DELMTX=$(comm -23 "$MTX_BEFORE" "$MTX_AFTER")
NDEL=$(printf '%s' "$DELMTX" | grep -c . || true)
if [ "$NDEL" != 0 ]; then
  say "FAIL: $NDEL matrix archive(s) DELETED mid-battery (A-AH45/KS-AH-88-2):"
  printf '%s\n' "$DELMTX" | head -3 | tee -a "$OUTF"
  FAILS=$((FAILS+1))
fi
# A-AH48: any PERSISTENT name whose sha mutated inside the window is a
# same-name swap — the endpoint comm above cannot see it (KS-AH-89-2).
# A-AH52 (Council WP-90) — DECLARED LIMIT: this tooth compares snapshots
# taken at the battery's ENDPOINTS only; a swap-in + restore fully inside
# the window is invisible to it (no gate re-reads the archive in-window
# today; the limit is declared, not closed).
MTX_SHA_AFTER="$W/.mtx-sha-after"
( cd "$MTXDIR" 2>/dev/null && shasum -a 256 -- * 2>/dev/null | sort -k2 ) > "$MTX_SHA_AFTER" || : > "$MTX_SHA_AFTER"
# A-AH52 guard: archive NAMES with whitespace would cross-join below
# (join splits on the first token) and produce a MISLEADING swap FAIL —
# make it a FAIL per NOME instead (names are space-free by construction).
BADNAME=$(awk 'NF > 2 {for (i=2;i<=NF;i++) printf "%s%s", $i, (i<NF?" ":"\n")}' "$MTX_SHA_BEFORE" "$MTX_SHA_AFTER" | sort -u)
if [ -n "$BADNAME" ]; then
  say "FAIL: matrix archive name(s) with whitespace — A-AH48 join undefined on these (A-AH52):"
  printf '%s\n' "$BADNAME" | head -3 | tee -a "$OUTF"
  FAILS=$((FAILS+1))
fi
SWAPPED=$(join -j 2 -o 1.2,1.1,2.1 "$MTX_SHA_BEFORE" "$MTX_SHA_AFTER" 2>/dev/null | awk '$2 != $3 {print $1}')
if [ -n "$SWAPPED" ]; then
  say "FAIL: persistent matrix archive(s) with MUTATED sha mid-battery — same-name swap (A-AH48/KS-AH-89-2):"
  printf '%s\n' "$SWAPPED" | head -3 | tee -a "$OUTF"
  FAILS=$((FAILS+1))
fi

PASSN=$((TOTAL-FAILS))
if [ "$FAILS" = 0 ]; then
  say "== BATTERY-88PRE PASS ($PASSN/$TOTAL) git=$GIT_REV =="
  SHA=$(shasum -a 256 "$OUTF" | cut -d' ' -f1)
  MTX_SHA=$(shasum -a 256 "$MTXDIR/$NEWMTX" | cut -d' ' -f1)
  echo "rev=$GIT_REV sha256=$SHA matrix=$NEWMTX matrix_sha256=$MTX_SHA" > "$W/.done"
  LEDGER="$REPO/wp83-harness/evidence/battery-stamps.ledger"
  echo "battery=88pre rev=$GIT_REV sha256=$SHA matrix=$NEWMTX matrix_sha256=$MTX_SHA" >> "$LEDGER"
  echo "stamp ledgered (A-SK41): commit $LEDGER NOW — the claim is PROVISIONAL until the append is at HEAD"
else
  say "== BATTERY-88PRE FAIL($FAILS/$TOTAL) git=$GIT_REV =="
fi
exit $FAILS
