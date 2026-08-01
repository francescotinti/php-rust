#!/bin/bash
# measure85-campaign.sh — S-85.0 (Council WP-86 §Sintesi punto 5): the
# DISCRIMINATION campaign — VDL24 gets its falsifier, VP gets R>=9.
# Sequential, ENFORCE, one rev. NO slope phase (VC binary-bound to
# 7a610457, A-BB43; any future slope verdict must pass
# wp85-harness/gate-slope-verdict.sh, KG-86-1).
#
#   Phase CAL  (A-DL28 calibration): mem-census arm, TWO separate W=1 runs
#              — hello.php only, then hello_pad85.php only. Each run's
#              teardown row gives the fixture's own net(ord1): NET_H, NET_P
#              (same binary, same chain). NET_P != NET_H or the canary is
#              vacuous (FAIL).
#   Phase DL28 (A-DL28≡A-BB41, canary MONOLATERALE): W=2, request 1 ->
#              hello (round-robin ADVISORY, A-PP32), request 2 -> pad.
#              Attribution is BY PATH IN-BAND on the rows (KS-PP-86-3),
#              never by request order. PER-THREAD holds iff the hello row
#              nets EXACTLY NET_H and the pad row nets EXACTLY NET_P on
#              DISTINCT threads; equal nets => frozen window => VDL24
#              VOID, classification RITIRATA (KL-86-2/KB-86-2).
#              NOTE (S-85.0, verified BY NAME): the net bracket wraps ONLY
#              `lower_source` (vm/mod.rs) — the path->CString site of the
#              VW piecewise is OUTSIDE the bracket, so the len>=384 path
#              canary of A-BB41 would NOT move the net; the content form
#              (Leijen A-DL28) with exact calibration is the operative one.
#   Phase P    (A-BB40/KB-86-1): union arm, W=ncpu R=9 min-of-R on
#              hello.php N=100 — the peak identity re-measured at the
#              cardinality a NEW pin requires; first run NAMED in the
#              verdict (cold-start attribution, KL-86-1).
#   KH86-1     every built arm passes gate-binary-noprobe: nm/strings
#              (necessary half) + hash == matrix line (sufficient half —
#              the S-85.0 positive control showed the probe symbol is
#              dead-stripped even in a tainted build: hash is the
#              discriminator).
#
# Preconditions: battery-85pre PASS at THIS rev (A-SK36 .done rev+sha256;
# equivalence only via battery-equivalence.sh v3), matrix archived at HEAD
# (in-battery), tree+harness porcelain. Raws never rm'd (quarantine +
# manifest, KS-AH-83-2).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
M78="$REPO/wp78-harness/measure78.sh"
OUT="$REPO/wp78-harness/measure-out"
OUTBIN="$HOME/Claude/php-rust-output/release"
ORACLE="/opt/homebrew/opt/php/bin/php"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
export PHPR_CAMPAIGN_SCRIPT="$HERE/measure85-campaign.sh"   # A-AH30
FAILS=0
NC=$(sysctl -n hw.ncpu)
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
HEAD_AT_START="$GIT_REV"
fail() { echo "FAIL: $*"; FAILS=$((FAILS+1)); }

echo "== measure85-campaign git=$GIT_REV ncpu=$NC (VDL24 discrimination + VP R=9) =="

# --- preconditions ------------------------------------------------------------
BATT="/Volumes/Extreme Pro/Claude/wp85-battery-out"
if [ ! -f "$BATT/.done" ]; then echo "FAIL: battery-85pre not run"; exit 1; fi
BATT_REV=$(sed -n 's/^rev=\([0-9a-f]*\).*/\1/p' "$BATT/.done")
if grep -qE "^== BATTERY-85PRE PASS \([0-9]+/[0-9]+\) git=$GIT_REV ==\$" "$BATT/battery-85pre.out" 2>/dev/null \
   && [ "$BATT_REV" = "$GIT_REV" ]; then
  echo "battery-85pre PASS at rev $BATT_REV == HEAD (per NAME, anchored)"
else
  if bash "$REPO/wp83-harness/battery-equivalence.sh" "$BATT/battery-85pre.out" "$BATT_REV"; then
    echo "battery equivalence LEGAL ($BATT_REV certifies $GIT_REV) — COMMIT the ledger append and cite the id (A-SK37)"
  else
    echo "FAIL: battery not PASS at HEAD and equivalence REFUSED"; exit 1
  fi
fi
DIRTY="$(git -C "$REPO" status --porcelain -- crates Cargo.toml Cargo.lock \
  'wp7*-harness/*.sh' 'wp7*-harness/*.pl' 'wp8*-harness/*.sh' 'wp8*-harness/*.pl' 2>/dev/null)"
[ -z "$DIRTY" ] || { echo "FAIL: tree/harness dirty (KS-AH-81-1/A-AH21)"; echo "$DIRTY" | head -5; exit 1; }
for f in hello.php hello_pad85.php; do
  [ -f "$FIXDIR/$f" ] || { echo "FAIL: committed fixture missing: $f"; exit 1; }
done
MTX=$(/bin/ls "$REPO/wp78-harness/matrix-archive"/feature-matrix.$GIT_REV.*.log 2>/dev/null | tail -1)
[ -n "$MTX" ] || { echo "FAIL: no archived matrix at HEAD rev $GIT_REV (battery must regenerate it)"; exit 1; }
mtx_hash() { tr -d '\0' < "$MTX" | sed -n "s/^bin\[$1\] sha256\[0:16\]=\([0-9a-f]*\).*/\1/p"; }

build_arm() { # <features>
  ( cd "$REPO" && cargo build --release -p php-server --features "$1" ) \
    > "$OUT/m85.build.$1.log" 2>&1 || { echo "FAIL: build $1"; exit 1; }
}
head_unmoved() { # KG-84-2 chain refusal
  local now; now="$(git -C "$REPO" rev-parse --short HEAD)"
  [ "$now" = "$HEAD_AT_START" ] || { echo "FAIL: HEAD moved mid-campaign ($now != $HEAD_AT_START, KG-81-2/KG-84-2)"; exit 1; }
}
noprobe_enforce() { # <matrix-label> — KH86-1 both halves
  local want; want=$(mtx_hash "$1")
  [ -n "$want" ] || { echo "FAIL: matrix has no bin[$1] line (KH86-1)"; exit 1; }
  bash "$HERE/gate-binary-noprobe.sh" "$OUTBIN/php-server" "$want" \
    || { echo "FAIL: KH86-1 on arm $1 — campaign VOID"; exit 1; }
}

# mem-census W-run: <label> <workers> <up-fixture> [second-fixture]
# The up-probe's first SUCCESS is a real request (A-PP32: mapping ADVISORY;
# attribution is by path in-band). Records server exit code in-band
# (KS-MS-86-1: rows from a run with exit!=0 are VOID).
memrun() {
  local label="$1" workers="$2" fx1="$3" fx2="${4:-}"
  local MC="$OUT/m85.$label.memcensus"
  : > "$MC"
  local PORT=8297
  head_unmoved
  PHPR_MEM_CENSUS="$MC" "$OUTBIN/php-server" --axum --workers "$workers" --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$OUT/m85.$label.log" &
  local DPID=$!
  local up=0
  for _ in $(seq 1 100); do
    curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/$fx1" && { up=1; break; }
    sleep 0.1
  done
  [ $up = 1 ] || { fail "m85.$label server not up"; kill -TERM $DPID 2>/dev/null; return 1; }
  local want got
  want=$(cd "$FIXDIR" && "$ORACLE" -n "$fx1" 2>/dev/null)
  got=$(curl -s -m 10 "http://127.0.0.1:$PORT/$fx1")
  [ "$got" = "$want" ] || fail "m85.$label full-body $fx1 != oracle (KS-AH-83-1)"
  if [ -n "$fx2" ]; then
    want=$(cd "$FIXDIR" && "$ORACLE" -n "$fx2" 2>/dev/null)
    got=$(curl -s -m 10 "http://127.0.0.1:$PORT/$fx2")
    [ "$got" = "$want" ] || fail "m85.$label full-body $fx2 != oracle (KS-AH-83-1)"
  fi
  kill -TERM "$DPID" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  echo "mem_hash=$MEM_HASH git=$GIT_REV campaign=$PHPR_CAMPAIGN_SCRIPT arm=axum-worker w=$workers phase=$label server_exit=$rc" >> "$MC"
  if grep -qE "panicked|aborting" "$OUT/m85.$label.log"; then
    fail "m85.$label stderr carries panic/abort markers — rows VOID (KS-MS-86-1)"
  fi
}

# --- Phase CAL + DL28 (mem-census arm) -----------------------------------------
echo "== Phase CAL+DL28: mem-census arm (KH86-1 enforced) =="
build_arm mem-census
MEM_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
noprobe_enforce mem-census
memrun cal-h  1 hello.php
memrun cal-p  1 hello_pad85.php
memrun dl28   2 hello.php hello_pad85.php

# --- Phase P: A-BB40 peak W=ncpu R=9 -------------------------------------------
build_arm axum-server
UNION_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
WANT_UNION=$(mtx_hash union)
[ "$UNION_HASH" = "$WANT_UNION" ] || { echo "FAIL: union hash $UNION_HASH != matrix $WANT_UNION"; exit 1; }
noprobe_enforce union
echo "union php-server=$UNION_HASH"
echo "== Phase P: peak W=$NC R=9 (union) — A-BB40 min-of-R, first run NAMED =="
for R in 1 2 3 4 5 6 7 8 9; do
  head_unmoved
  W=$NC bash "$M78" axum "85p.w$NC.r$R" hello.php 100 || fail "phaseP r$R rc"
done

# --- restore the union binary on disk (already union) ---------------------------
FINAL_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
[ "$FINAL_HASH" = "$UNION_HASH" ] || fail "union final hash moved ($FINAL_HASH != $UNION_HASH)"

echo "== measure85-campaign DONE fails=$FAILS =="
touch "$OUT/m85.done"
exit $FAILS
