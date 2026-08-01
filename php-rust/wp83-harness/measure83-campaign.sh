#!/bin/bash
# measure83-campaign.sh — S-83.0: the WP-83 measures in the FORMS the
# Council WP-84 ordered. Sequential, ENFORCE, one rev.
#
#   Phase C  (slope, NEW estimator — KB-84-1/A-BB31): arms lever+base,
#            N in {1000,2000}, R=9 per cell, union binary UNARMED
#            (KL-84-3: no counting build ever times), time -l user.
#            Estimator = MIN-OF-R per cell; slope = (min(2000)-min(1000))/1000.
#            f DECLARED EX-ANTE: machine noise on totals ~5% multiplicative;
#            resolution rule (verdict83): spread of the 3 smallest user
#            times per cell, scaled to us/req, must be < banda/3 where
#            banda = 0.05*slope_base_min + 25 us/req — else VC NULL again
#            (never ADVISORY). Claim scope NAMED: axum W=1 hello closed-seq.
#   Phase CR (lever attribution, A-BG32): lever arm ONLY (the pre-lever
#            base binary has no __reqns probe — DECLARED), R=3, N=2000,
#            PHPR_REQ_NS=1: per-request ns samples drained after the
#            measured window; verdict reports regime median/min [derivata]
#            as the lever-arm per-request distribution, never the A/B judge.
#   Phase R  (retained DECOMPOSED — A-DL20/21/22): mem-census binary,
#            instrument ARMED (PHPR_MEM_CENSUS — feature != env != alloc,
#            the S-82.0 lesson), cli-server arm (the dump lives in
#            run_module_with_hir teardown; arm=cli-server IN the row,
#            A-PP25). R1 = 4-fixture standard; R2 = hello-only (A-DL22
#            quadratura vs the VP marginal 3.44 MB/worker). Full-body vs
#            oracle FIRST on both (KS-AH-83-1).
#   Phase A  (register split — A-BB33/A-SK28): census arm, MEASURED=30:
#            autoload82 + autoload83_reg + hello; verdict emits
#            include-HIT = b(autoload82)-b(reg83) and reg = b(reg83)-b(hello)
#            [derivata], labeled "upper bound vs opcache inheritance-cache"
#            (A-DS25).
#   Phase W  (path>=384B — KB-83-1/KB-84-2): census arm, MEASURED=30 on the
#            415-byte-path fixture; verdict judges a_bytes == 2x len(path)
#            EXACT on steady rows — the floor model at its boundary, outcome
#            NAMED either way.
#
# Preconditions: battery-83pre PASS at THIS rev (per NAME; equivalence only
# via battery-equivalence.sh — KS-SK-84-1/KS-AH-84-3), matrix at HEAD with
# no commits in between (KG-81-2, enforced per-run by measure78), tree +
# harness porcelain clean (KS-AH-81-1/A-AH21). Raws NEVER rm'd: quarantine
# + manifest (KS-AH-83-2); supplements only per SUPPLEMENT_NORM.md (A-SK29).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
M78="$REPO/wp78-harness/measure78.sh"
OUT="$REPO/wp78-harness/measure-out"
OUTBIN="$HOME/Claude/php-rust-output/release"
ORACLE="/opt/homebrew/opt/php/bin/php"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
BASE_REV="7593d8e"
BASEWT="/Volumes/Extreme Pro/Claude/wp83-basewt"
BASE_TARGET="/Volumes/Extreme Pro/Claude/phpr-old-target"
LP83="lp83_segment_padding_0123456789_0123456789_0123456789_x1/lp83_segment_padding_0123456789_0123456789_0123456789_x2/lp83_segment_padding_0123456789_0123456789_0123456789_x3/lp83_segment_padding_0123456789_0123456789_0123456789_x4/lp83_segment_padding_0123456789_0123456789_0123456789_x5/bare_longpath_384_kb831_kb842_fixture.php"
export PHPR_CAMPAIGN_SCRIPT="$HERE/measure83-campaign.sh"   # A-AH30
FAILS=0
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
HEAD_AT_START="$GIT_REV"
fail() { echo "FAIL: $*"; FAILS=$((FAILS+1)); }

echo "== measure83-campaign git=$GIT_REV f_exante=5% estimator=min-of-R(R=9) =="

# --- preconditions -------------------------------------------------------------
BATT="/Volumes/Extreme Pro/Claude/wp83-battery-out"
if [ ! -f "$BATT/.done" ]; then echo "FAIL: battery-83pre not run"; exit 1; fi
BATT_REV=$(awk -F= '/^rev=/{print $2}' "$BATT/.done")
if grep -q "BATTERY-83PRE PASS" "$BATT/battery-83pre.out" 2>/dev/null && [ "$BATT_REV" = "$GIT_REV" ]; then
  echo "battery-83pre PASS at rev $BATT_REV == HEAD (per NAME, 15/15)"
else
  # A-SK30/A-AH34: the ONLY legal equivalence path — per NAME, ledgered.
  if bash "$HERE/battery-equivalence.sh" "$BATT/battery-83pre.out" "$BATT_REV" \
       "$HERE/evidence/equivalence.ledger"; then
    echo "battery equivalence LEGAL ($BATT_REV certifies $GIT_REV)"
  else
    echo "FAIL: battery not PASS at HEAD and equivalence REFUSED"; exit 1
  fi
fi
DIRTY="$(git -C "$REPO" status --porcelain -- crates Cargo.toml Cargo.lock \
  'wp7*-harness/*.sh' 'wp7*-harness/*.pl' 'wp8*-harness/*.sh' 'wp8*-harness/*.pl' 2>/dev/null)"
[ -z "$DIRTY" ] || { echo "FAIL: tree/harness dirty (KS-AH-81-1/A-AH21)"; echo "$DIRTY" | head -5; exit 1; }
for f in autoload82.php autocls82_al82.php autoload83_reg.php "$LP83"; do
  [ -f "$FIXDIR/$f" ] || { echo "FAIL: committed fixture missing: $f"; exit 1; }
done

build_arm() { # <features>
  ( cd "$REPO" && cargo build --release -p php-server --features "$1" ) \
    > "$OUT/m83.build.$1.log" 2>&1 || { echo "FAIL: build $1"; exit 1; }
}
head_unmoved() { # KG-84-2 chain refusal for every custom run
  local now; now="$(git -C "$REPO" rev-parse --short HEAD)"
  [ "$now" = "$HEAD_AT_START" ] || { echo "FAIL: HEAD moved mid-campaign ($now != $HEAD_AT_START, KG-81-2/KG-84-2)"; exit 1; }
}

# --- Phase C: slope, min-of-R, both arms (union binary, UNARMED) --------------
build_arm axum-server
UNION_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
echo "union php-server=$UNION_HASH"
echo "== Phase C lever: N in {1000,2000} R=9 (unarmed union) =="
for N in 1000 2000; do for R in 1 2 3 4 5 6 7 8 9; do
  bash "$M78" axum "83c.lever.n$N.r$R" hello.php "$N" || fail "phaseC lever n$N r$R rc"
done; done

echo "== Phase CR: lever per-request attribution (PHPR_REQ_NS=1, R=3, N=2000) =="
for R in 1 2 3; do
  PHPR_REQ_NS=1 bash "$M78" axum "83cr.lever.n2000.r$R" hello.php 2000 || fail "phaseCR r$R rc"
done

# --- base arm: A-AH31 helper (lock-cmp A-AH32, header A-TH26) ------------------
echo "== base arm build ($BASE_REV via base-arm-build.sh) =="
BASE_HDR="$OUT/m83.base.header"
: > "$BASE_HDR"
bash "$HERE/base-arm-build.sh" "$BASE_REV" "$BASEWT" "$BASE_TARGET" axum-server "$BASE_HDR" \
  || { echo "FAIL: base arm build/lock-cmp (KS-AH-84-1)"; exit 1; }
BASE_BIN="$BASE_TARGET/release/php-server"
BASE_HASH=$(shasum -a 256 "$BASE_BIN" | cut -c1-16)
echo "rustc_live=$(rustc -V)" >> "$BASE_HDR"
base_run() { # <label> <N> — A-BG31: identity refusal -> response parity -> run
  local label="$1" N="$2" PORT=8297
  local RUN="$OUT/m83.base.$label"
  head_unmoved
  # A-BG31 pin 1 (identity): driver_sha covering campaign + helper, in-header.
  local DSHA
  DSHA="$(shasum -a 256 "$PHPR_CAMPAIGN_SCRIPT" "$HERE/base-arm-build.sh" | shasum -a 256 | cut -c1-16)"
  /usr/bin/time -l "$BASE_BIN" --axum --workers 1 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$RUN.log" &
  local TPID=$!
  local up=0
  for _ in $(seq 1 100); do
    curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php" && { up=1; break; }
    sleep 0.1
  done
  [ $up = 1 ] || { fail "base $label: server not up"; kill -TERM $TPID 2>/dev/null; return; }
  # A-BG31 pin 2 (response parity): body vs oracle BEFORE the timed loop.
  local want got
  want=$(cd "$FIXDIR" && "$ORACLE" -n hello.php 2>/dev/null)
  got=$(curl -s -m 10 "http://127.0.0.1:$PORT/hello.php")
  [ "$got" = "$want" ] || { fail "base $label: body != oracle (A-BG31)"; }
  for _ in $(seq 1 9); do curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/hello.php"; done
  for _ in $(seq 1 "$N"); do curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/hello.php"; done
  local SRVPID
  SRVPID=$(pgrep -P "$TPID" | head -1); [ -z "$SRVPID" ] && SRVPID=$TPID
  kill -TERM "$SRVPID" 2>/dev/null; wait "$TPID" 2>/dev/null
  {
    echo "base_rev=$BASE_REV base_hash=$BASE_HASH label=$label W=1 N=$N head=$HEAD_AT_START driver_sha=$DSHA campaign=$PHPR_CAMPAIGN_SCRIPT"
    cat "$BASE_HDR"
  } >> "$RUN.log"
}
echo "== Phase C base: N in {1000,2000} R=9 (interleaved same-evening, A-TH26) =="
for N in 1000 2000; do for R in 1 2 3 4 5 6 7 8 9; do
  base_run "83c.n$N.r$R" "$N"
done; done

# --- Phase R: retained DECOMPOSED (mem-census, ARMED, cli-server) --------------
echo "== Phase R: retained per-entry (mem-census ARMED, cli-server arm) =="
build_arm mem-census
MEM_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
run_phase_r() { # <label> <fixtures...>
  local label="$1"; shift
  local PORT=8299 MCFILE="$OUT/m83.$label.memcensus"
  head_unmoved
  : > "$MCFILE"
  PHPR_MEM_CENSUS="$MCFILE" "$OUTBIN/php-server" --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$OUT/m83.$label.log" &
  local MPID=$!
  local up=0
  for _ in $(seq 1 100); do curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php" && { up=1; break; }; sleep 0.1; done
  [ $up = 1 ] || { fail "$label server not up"; kill -TERM $MPID 2>/dev/null; return; }
  local f want got
  for f in "$@"; do
    want=$(cd "$FIXDIR" && "$ORACLE" -n "$f" 2>/dev/null)
    for _ in 1 2 3; do
      got=$(curl -s -m 10 "http://127.0.0.1:$PORT/$f")
      [ "$got" = "$want" ] || { fail "$label full-body $f != oracle (KS-AH-83-1)"; break; }
    done
  done
  for f in "$@"; do for _ in $(seq 1 5); do curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/$f"; done; done
  kill -TERM "$MPID" 2>/dev/null; wait "$MPID" 2>/dev/null
  echo "mem_hash=$MEM_HASH git=$GIT_REV campaign=$PHPR_CAMPAIGN_SCRIPT arm=cli-server" >> "$MCFILE"
}
run_phase_r "r1.fourfix" hello.php include_gate.php include_heavy.php bare.php
run_phase_r "r2.helloonly" hello.php

# --- Phase A: register split (census arm, MEASURED=30) -------------------------
echo "== Phase A: autoload register split (census arm) =="
build_arm census-instrumentation
for fx in autoload82 autoload83_reg hello; do
  bash "$M78" census "83a.$fx" "$fx.php" 30 || fail "phaseA $fx rc"
done

# --- Phase W: long-path floor boundary (census arm, MEASURED=30) ---------------
echo "== Phase W: path>=384B a_bytes floor boundary =="
bash "$M78" census "83w.longpath" "$LP83" 30 || fail "phaseW rc"

# --- restore the union binary on disk ------------------------------------------
build_arm axum-server
FINAL_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
[ "$FINAL_HASH" = "$UNION_HASH" ] || fail "union restore hash moved ($FINAL_HASH != $UNION_HASH)"

echo "== measure83-campaign DONE fails=$FAILS =="
touch "$OUT/m83.done"
exit $FAILS
