#!/bin/bash
# battery-91pre.sh — S-91.0: FULL battery on the WP-91 tree, sequential
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
# v7 changes (Council WP-92, DELIBERA UNICA di formato ledger — design91-ledger.md):
#   A-AH58: EVERY attempts row carries writer=script:<sha16(script)> or
#     writer=operator; esito=ABORT is legal ONLY with writer=operator. The
#     S-90.0 ABORT row was hand-written with a grammar identical to
#     att_row's — provenance was a convention, not a field. The script now
#     TRAPS INT/TERM/HUP and ledgers the ABORT itself (the signal IS the
#     operator's act, recorded mechanically), and detects a HEAD move
#     before emitting its terminal row (the S-90.0 a1 failure mode).
#   A-AH59: FAIL and REFUSE rows carry sha256=<sha256(OUT-parziale)> — the
#     attempts↔OUT anchor closes on EVERY esito, not only the PASS
#     (KS-AH-92-2: FAIL/REFUSE without an anchor => battery VOID).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
W="/Volumes/Extreme Pro/Claude/wp91-battery-out"
mkdir -p "$W"
OUTF="$W/battery-91pre.out"
: > "$OUTF"
rm -f "$W/.done"
FAILS=0
TOTAL=0
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
say() { echo "$@" | tee -a "$OUTF"; }

# A-AH50≡A-BG49 (Council WP-90, KS-AH-90-1/KG-90-1): EVERY battery
# attempt leaves an append-only row — PASS, FAIL and REFUSE alike. The
# two S-88.0 fails (F16b at 94661d8, porcelain-refuse at b4ad60b) lived
# only in commit messages; the asymmetry with the campaign VOID rows
# (A-BG46) is closed here.
ATTLEDGER="$REPO/wp83-harness/evidence/battery-attempts.ledger"
# A-AH58: every row is signed by its writer — the running script by
# default; writer=operator ONLY on ABORT (emitted by the trap below).
SCRIPT_SHA="$(shasum -a 256 "$0" | cut -c1-16)"
att_row() { # esito=... [k=v ...] — appends writer= unless already given
  case "$*" in
    *writer=*) echo "attempt_epoch=$(date +%s) battery=91pre rev=$GIT_REV $*" >> "$ATTLEDGER";;
    *) echo "attempt_epoch=$(date +%s) battery=91pre rev=$GIT_REV $* writer=script:$SCRIPT_SHA" >> "$ATTLEDGER";;
  esac
}
out_sha() { shasum -a 256 "$OUTF" | cut -d' ' -f1; }
# A-AH58 trap: a signal is the OPERATOR's act — ledger it mechanically as
# ABORT writer=operator with the partial-OUT anchor (A-AH59), never leave
# the ledger silent (the S-90.0 ABORT was a hand-written row).
on_signal() {
  att_row "esito=ABORT reason=signal writer=operator sha256=$(out_sha)"
  exit 1
}
trap on_signal INT TERM HUP
# A-AH58: HEAD moved mid-battery = operator error (S-90.0 a1) — detect it
# mechanically before ANY terminal row.
assert_head_unmoved() {
  local now; now="$(git -C "$REPO" rev-parse --short HEAD)"
  if [ "$now" != "$GIT_REV" ]; then
    say "ABORT: HEAD moved mid-battery ($GIT_REV -> $now) — battery VOID (A-AH58)"
    att_row "esito=ABORT reason=head-moved writer=operator sha256=$(out_sha)"
    exit 1
  fi
}

DIRTY="$(git -C "$REPO" status --porcelain 2>/dev/null)"
if [ -n "$DIRTY" ]; then
  say "FAIL: tree not porcelain at battery start — battery VOID (A-SK42/KS-SK-87-1):"
  echo "$DIRTY" | head -10 | tee -a "$OUTF"
  say "== BATTERY-91PRE REFUSED git=$GIT_REV =="
  att_row "esito=REFUSE reason=tree-not-porcelain sha256=$(out_sha)"
  exit 1
fi

say "== battery-91pre git=$GIT_REV =="
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
# A-SK-79/KS-SK-93-4 (Council WP-93): rc=0 alone no longer closes the cifre
# row — the log must carry the VERDICT-grade PASS line signed judge_sha=
# (ADVISORY-PASS now exits 64 and is a FAIL above by construction).
if ! grep -q '^PASS gate-measure-cifre --all.*judge_sha=' "$W/measure-cifre.log"; then
  say "FAIL: measure-cifre closed with NO verdict-grade PASS line (judge_sha=) in the log (A-SK-79/KS-SK-93-4)"
  FAILS=$((FAILS+1))
fi
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
# A-AH58: no terminal row from a battery whose HEAD moved mid-run (the
# S-90.0 a1 failure mode, now detected mechanically).
assert_head_unmoved
if [ "$FAILS" = 0 ]; then
  say "== BATTERY-91PRE PASS ($PASSN/$TOTAL) git=$GIT_REV =="
  SHA=$(shasum -a 256 "$OUTF" | cut -d' ' -f1)
  MTX_SHA=$(shasum -a 256 "$MTXDIR/$NEWMTX" | cut -d' ' -f1)
  echo "rev=$GIT_REV sha256=$SHA matrix=$NEWMTX matrix_sha256=$MTX_SHA" > "$W/.done"
  LEDGER="$REPO/wp83-harness/evidence/battery-stamps.ledger"
  echo "battery=91pre rev=$GIT_REV sha256=$SHA matrix=$NEWMTX matrix_sha256=$MTX_SHA" >> "$LEDGER"
  att_row "esito=PASS k=$PASSN/$TOTAL sha256=$SHA"
  echo "stamp ledgered (A-SK41): commit $LEDGER + battery-attempts.ledger NOW — the claim is PROVISIONAL until the appends are at HEAD"
else
  say "== BATTERY-91PRE FAIL($FAILS/$TOTAL) git=$GIT_REV =="
  # A-BG49: the FAIL branch ledgers too — the failing gate names are in
  # the OUT; the row carries the count and the first failing label.
  FIRSTFAIL=$(sed -n 's/^FAIL \([a-z0-9-]*\):.*/\1/p' "$OUTF" | head -1)
  # A-AH59: the FAIL row anchors to its (complete-at-emission) OUT — the
  # forge class closed on the PASS stays closed on every esito.
  att_row "esito=FAIL fails=$FAILS/$TOTAL first_fail=${FIRSTFAIL:-unknown} sha256=$(out_sha)"
fi
exit $FAILS
