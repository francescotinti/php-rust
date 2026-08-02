#!/bin/bash
# measure86-campaign.sh — S-86.0 (Council WP-87 §Sintesi p5/p6/p7): the
# ATTRIBUTION campaign. Sequential, ENFORCE, one rev.
#
#   Phase CAL   (mem-census, W=1 ×2): NET_H (hello) and NET_P (pad) on THIS
#               binary — every downstream prediction is calibration-bound.
#   Phase INV   (A-BB45, counter-proof of the additive decomposition):
#               W=1, /__reqns up (NO dispatch), then pad (req1, ord=1),
#               then hello (req2, ord=2 SAME thread). Named ex-ante
#               prediction: net(hello ord2) = NET_H − NET_P + 460.146 B =
#               6.842 B on reproduced cals (460.146 = the MODEL-GRADE
#               pad-own under test, src=m85.dl28). KB-87-1: until this
#               passes, the decomposition is never a budget addend.
#   Phase BURST (A-DL33, pipeline fail-closed): W=1 hello with
#               PHPR_MEMCENSUS_TEST_BURST=1 — the row must move by EXACTLY
#               +1.048.576 B vs NET_H and the calibration tooth must
#               DECLASSIFY the run (proof the PIPELINE rejects a polluted
#               row; this run is never a measurement).
#   Phase OVL   (A-BB47, canary under real overlap): W=2, /__reqns up,
#               then hello+pad fired CONCURRENTLY. Attempted up to 10×
#               (fresh server each, attempt ledger in-band, KG-87-1): an
#               attempt QUALIFIES when the two tag=lower_span intervals
#               INTERSECT on distinct tids. Verdict judges nets vs cals on
#               the qualifying attempt; no qualifying attempt = A-BB47
#               OPEN by name (never a silent pass).
#   Phase W123  (A-DL31 physical half, reduced form — the per-thread heap
#               split was REFUTED BY INSTRUMENT: mimalloc v3 default heap
#               is process-shared, heap= tooth): union arm under
#               /usr/bin/time -l at W=1/2/3, one hello per worker.
#               Δpeak per added worker vs the KL-85-2 physical
#               3.605.572 B ±5%.
#   Phase VW500 (A-BB48): census arm MEASURED=30 on the committed len=500
#               fixture + hello control. Named ex-ante: a_calls=4,
#               a_bytes=2·500+2·501=2.002 B.
#   Phase VP-ABBA (A-BB46/A-DL35): union arm W=ncpu, R=9 per arm,
#               INTERLEAVED ABBA — arm A: PHPR_PURGE_DELAY=0 (historical
#               protocol), arm B: PHPR_PURGE_DELAY=default (mimalloc
#               native). Effect in-band per run (MIMALLOC_VERBOSE option
#               table + purge_env= label). Attribution thresholds (Bak):
#               spread(A) < 0,5×spread(B) ⇒ purge-timing attributed.
#               KL-87-1/KB-87-2: NO pin is founded here — attribution only.
#
# Preconditions: battery-86pre PASS at THIS rev (A-SK41 ledgered stamp,
# A-SK42 porcelain+counted, A-AH40 matrix in .done), tree+harness
# porcelain, raws never rm'd (KS-AH-83-2), head_unmoved before every
# driver invocation (A-BG37).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
M78="$REPO/wp78-harness/measure78.sh"
OUT="$REPO/wp78-harness/measure-out"
OUTBIN="$HOME/Claude/php-rust-output/release"
ORACLE="/opt/homebrew/opt/php/bin/php"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
export PHPR_CAMPAIGN_SCRIPT="$HERE/measure86-campaign.sh"   # A-AH30
FAILS=0
NC=$(sysctl -n hw.ncpu)
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
HEAD_AT_START="$GIT_REV"
fail() { echo "FAIL: $*"; FAILS=$((FAILS+1)); }

# VW500 fixture rel path (committed; exact abs len 500 at THIS fixdir).
D500="vw86_len500_$(printf 'a%.0s' $(seq 1 150))"
N500="f$(printf 'c%.0s' $(seq 1 243)).php"
REL500="$D500/$N500"

echo "== measure86-campaign git=$GIT_REV ncpu=$NC (attribution: INV/BURST/OVL/W123/VW500/VP-ABBA) =="

# --- preconditions ------------------------------------------------------------
BATT="/Volumes/Extreme Pro/Claude/wp86-battery-out"
if [ ! -f "$BATT/.done" ]; then echo "FAIL: battery-86pre not run"; exit 1; fi
BATT_REV=$(sed -n 's/^rev=\([0-9a-f]*\).*/\1/p' "$BATT/.done")
if grep -qE "^== BATTERY-86PRE PASS \([0-9]+/[0-9]+\) git=$GIT_REV ==\$" "$BATT/battery-86pre.out" 2>/dev/null \
   && [ "$BATT_REV" = "$GIT_REV" ]; then
  echo "battery-86pre PASS at rev $BATT_REV == HEAD (per NAME, anchored)"
else
  if bash "$REPO/wp83-harness/battery-equivalence.sh" "$BATT/battery-86pre.out" "$BATT_REV"; then
    echo "battery equivalence LEGAL ($BATT_REV certifies $GIT_REV) — COMMIT the ledger append and cite the id (A-SK37)"
  else
    echo "FAIL: battery not PASS at HEAD and equivalence REFUSED"; exit 1
  fi
fi
DIRTY="$(git -C "$REPO" status --porcelain -- crates Cargo.toml Cargo.lock \
  'wp7*-harness/*.sh' 'wp7*-harness/*.pl' 'wp8*-harness/*.sh' 'wp8*-harness/*.pl' 2>/dev/null)"
[ -z "$DIRTY" ] || { echo "FAIL: tree/harness dirty (KS-AH-81-1/A-AH21)"; echo "$DIRTY" | head -5; exit 1; }
for f in hello.php hello_pad85.php "$REL500"; do
  [ -f "$FIXDIR/$f" ] || { echo "FAIL: committed fixture missing: $f"; exit 1; }
done
ABS500="$FIXDIR/$REL500"
[ "$(printf '%s' "$ABS500" | wc -c | tr -d ' ')" = 500 ] || { echo "FAIL: len500 fixture abs length != 500"; exit 1; }
# A-AH40 (KS-AH-87-1): matrix resolved FROM the battery .done, never tail -1.
MTX_NAME=$(sed -n 's/.*matrix=\([^ ]*\).*/\1/p' "$BATT/.done" | head -1)
MTX_WANT=$(sed -n 's/.*matrix_sha256=\([0-9a-f]*\).*/\1/p' "$BATT/.done" | head -1)
[ -n "$MTX_NAME" ] && [ -n "$MTX_WANT" ] || { echo "FAIL: battery .done lacks matrix=/matrix_sha256= (A-AH40)"; exit 1; }
MTX="$REPO/wp78-harness/matrix-archive/$MTX_NAME"
[ -f "$MTX" ] || { echo "FAIL: ledgered matrix archive missing (A-AH40)"; exit 1; }
[ "$(shasum -a 256 "$MTX" | cut -d' ' -f1)" = "$MTX_WANT" ] || { echo "FAIL: matrix sha != .done (KS-AH-87-1)"; exit 1; }
mtx_hash() { tr -d '\0' < "$MTX" | sed -n "s/^bin\[$1\] sha256\[0:16\]=\([0-9a-f]*\).*/\1/p"; }

build_arm() { # <features>
  ( cd "$REPO" && cargo build --release -p php-server --features "$1" ) \
    > "$OUT/m86.build.$1.log" 2>&1 || { echo "FAIL: build $1"; exit 1; }
}
head_unmoved() { # KG-84-2/A-BG37 chain refusal
  local now; now="$(git -C "$REPO" rev-parse --short HEAD)"
  [ "$now" = "$HEAD_AT_START" ] || { echo "FAIL: HEAD moved mid-campaign ($now != $HEAD_AT_START)"; exit 1; }
}
noprobe_enforce() { # <matrix-label>
  local want; want=$(mtx_hash "$1")
  [ -n "$want" ] || { echo "FAIL: matrix has no bin[$1] line"; exit 1; }
  bash "$REPO/wp85-harness/gate-binary-noprobe.sh" "$OUTBIN/php-server" "$want" \
    || { echo "FAIL: KH86-1/A-TH37 on arm $1 — campaign VOID"; exit 1; }
}

# mem-census run, /__reqns up-probe (NO worker dispatch — the router
# answers), then the NAMED request sequence; oracle full-body parity on
# each counted request (KS-AH-83-1); server exit + burst env in-band.
memrun86() { # <label> <workers> <burst:0|1> <fx...>
  local label="$1" workers="$2" burst="$3"; shift 3
  local MC="$OUT/m86.$label.memcensus"
  : > "$MC"
  local PORT=8296
  head_unmoved
  # NB: no empty-array expansion — macOS bash 3.2 + set -u aborts on
  # "${arr[@]}" when arr is empty (bit the first campaign launch).
  if [ "$burst" = 1 ]; then
    PHPR_MEMCENSUS_TEST_BURST=1 PHPR_MEM_CENSUS="$MC" \
      "$OUTBIN/php-server" --axum --workers "$workers" --port $PORT -t "$FIXDIR" \
      > /dev/null 2> "$OUT/m86.$label.log" &
  else
    PHPR_MEM_CENSUS="$MC" \
      "$OUTBIN/php-server" --axum --workers "$workers" --port $PORT -t "$FIXDIR" \
      > /dev/null 2> "$OUT/m86.$label.log" &
  fi
  local DPID=$!
  local up=0
  for _ in $(seq 1 100); do
    curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/__reqns" && { up=1; break; }
    sleep 0.1
  done
  [ $up = 1 ] || { fail "m86.$label server not up"; kill -TERM $DPID 2>/dev/null; return 1; }
  local n=0
  for fx in "$@"; do
    n=$((n+1))
    local want got
    want=$(cd "$FIXDIR" && "$ORACLE" -n "$fx" 2>/dev/null)
    got=$(curl -s -m 10 "http://127.0.0.1:$PORT/$fx")
    [ "$got" = "$want" ] || fail "m86.$label req$n body $fx != oracle (KS-AH-83-1)"
  done
  kill -TERM "$DPID" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  echo "mem_hash=$MEM_HASH git=$GIT_REV campaign=$PHPR_CAMPAIGN_SCRIPT arm=axum-worker w=$workers phase=$label server_exit=$rc burst=$burst ndisp=$n" >> "$MC"
  if grep -qE "panicked|aborting" "$OUT/m86.$label.log"; then
    fail "m86.$label stderr carries panic/abort markers — rows VOID (KS-MS-86-1)"
  fi
}

# --- mem-census arm ------------------------------------------------------------
echo "== Phase CAL+INV+BURST+OVL: mem-census arm =="
build_arm mem-census
MEM_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
noprobe_enforce mem-census
memrun86 cal-h  1 0 hello.php
memrun86 cal-p  1 0 hello_pad85.php
# INV: pad FIRST (ord=1), hello SECOND (ord=2, same thread) — A-BB45.
memrun86 inv    1 0 hello_pad85.php hello.php
# BURST: dedicated A-DL33 run — never a measurement.
memrun86 burst  1 1 hello.php

# OVL (A-BB47): concurrent first-requests; attempts ledgered in-band.
OVLED="$OUT/m86.ovl.attempts.ledger"
: > "$OVLED"
ovl_attempt() { # <n> -> 0 if spans intersect on distinct tids
  local n="$1"
  local MC="$OUT/m86.ovl.a$n.memcensus"
  : > "$MC"
  local PORT=8296
  head_unmoved
  PHPR_MEM_CENSUS="$MC" "$OUTBIN/php-server" --axum --workers 2 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$OUT/m86.ovl.a$n.log" &
  local DPID=$!
  local up=0
  for _ in $(seq 1 100); do
    curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/__reqns" && { up=1; break; }
    sleep 0.1
  done
  [ $up = 1 ] || { fail "m86.ovl.a$n server not up"; kill -TERM $DPID 2>/dev/null; return 1; }
  local BH BP
  BH=$(mktemp); BP=$(mktemp)
  curl -s -m 10 -o "$BH" "http://127.0.0.1:$PORT/hello.php" &
  local C1=$!
  curl -s -m 10 -o "$BP" "http://127.0.0.1:$PORT/hello_pad85.php" &
  local C2=$!
  wait "$C1" "$C2"
  local wanth wantp
  wanth=$(cd "$FIXDIR" && "$ORACLE" -n hello.php 2>/dev/null)
  wantp=$(cd "$FIXDIR" && "$ORACLE" -n hello_pad85.php 2>/dev/null)
  [ "$(cat "$BH")" = "$wanth" ] || fail "m86.ovl.a$n hello body != oracle"
  [ "$(cat "$BP")" = "$wantp" ] || fail "m86.ovl.a$n pad body != oracle"
  rm -f "$BH" "$BP"
  kill -TERM "$DPID" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  echo "mem_hash=$MEM_HASH git=$GIT_REV campaign=$PHPR_CAMPAIGN_SCRIPT arm=axum-worker w=2 phase=ovl.a$n server_exit=$rc attempt=$n" >> "$MC"
  # qualify: two lower_span rows (hello+pad fixtures), distinct tids,
  # intervals intersect
  # A-BB49 (WP-88): +0 coercion — after sub() awk fields lose strnum status
  # and '<' compares STRINGS ("4"<"13300" is false); this qualifier reported
  # 0/10 overlaps on a campaign whose raws intersect 10/10.
  tr -d '\0' < "$MC" | awk '
    /tag=lower_span/ {
      for (i=1;i<=NF;i++) { if ($i ~ /^tid=/) tid=$i; if ($i ~ /^t0_us=/) {sub("t0_us=","",$i); t0=$i+0}
                            if ($i ~ /^t1_us=/) {sub("t1_us=","",$i); t1=$i+0} }
      if ($NF ~ /hello_pad85[.]php$/) { pt0=t0; pt1=t1; ptid=tid }
      else if ($NF ~ /hello[.]php$/)  { ht0=t0; ht1=t1; htid=tid }
    }
    END {
      if (htid == "" || ptid == "" || htid == ptid) exit 1
      if (ht0 < pt1 && pt0 < ht1) exit 0
      exit 1
    }'
}
OVL_OK=""
for a in 1 2 3 4 5 6 7 8 9 10; do
  if ovl_attempt "$a"; then
    echo "attempt=$a QUALIFIED (spans intersect, distinct tids)" >> "$OVLED"
    OVL_OK="$a"
    break
  else
    echo "attempt=$a no-overlap-or-invalid (raw kept: m86.ovl.a$a.memcensus)" >> "$OVLED"
  fi
done
if [ -n "$OVL_OK" ]; then
  echo "OVL qualified at attempt $OVL_OK (ledger: $OVLED)"
else
  echo "OVL: no qualifying overlap in 10 attempts — A-BB47 stays OPEN by name (ledgered)"
fi

# --- Phase W123: union arm, time -l, one hello per worker ----------------------
echo "== Phase W123: union arm Δpeak per added worker (A-DL31 physical half) =="
build_arm axum-server
UNION_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
WANT_UNION=$(mtx_hash union)
[ "$UNION_HASH" = "$WANT_UNION" ] || { echo "FAIL: union hash $UNION_HASH != matrix $WANT_UNION"; exit 1; }
noprobe_enforce union
for w in 1 2 3; do
  head_unmoved
  LOG="$OUT/m86.w123.w$w.log"
  MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$OUTBIN/php-server" --axum --workers "$w" --port 8296 -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  TPID=$!
  up=0
  for _ in $(seq 1 100); do
    curl -s -o /dev/null -m 1 "http://127.0.0.1:8296/__reqns" && { up=1; break; }
    sleep 0.1
  done
  [ $up = 1 ] || { fail "m86.w123.w$w not up"; kill "$TPID" 2>/dev/null; continue; }
  # one request per worker (round-robin): w hello hits
  for _ in $(seq 1 "$w"); do
    curl -s -m 10 -o /dev/null "http://127.0.0.1:8296/hello.php"
  done
  SRV=$(lsof -nP -tiTCP:8296 -sTCP:LISTEN | head -1)
  kill -TERM "$SRV" 2>/dev/null
  wait "$TPID" 2>/dev/null
  echo "phase=w123 w=$w git=$GIT_REV union=$UNION_HASH" >> "$LOG"
done

# --- Phase VW500: census arm MEASURED=30 ---------------------------------------
echo "== Phase VW500: census arm len=500 + hello control (A-BB48) =="
build_arm census-instrumentation
CEN_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
WANT_CEN=$(mtx_hash census)
[ "$CEN_HASH" = "$WANT_CEN" ] || { echo "FAIL: census hash $CEN_HASH != matrix $WANT_CEN"; exit 1; }
noprobe_enforce census
head_unmoved
bash "$M78" census "86w.len500" "$REL500" 30 || fail "phaseVW500 len500 rc"
head_unmoved
bash "$M78" census "86w.hello" "hello.php" 30 || fail "phaseVW500 hello control rc"

# --- Phase VP-ABBA: union arm, R=9 per arm, interleaved ------------------------
echo "== Phase VP-ABBA: union W=$NC, 9×A (purge=0) + 9×B (default), ABBA interleave =="
build_arm axum-server
FINAL_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
[ "$FINAL_HASH" = "$WANT_UNION" ] || { echo "FAIL: union rebuild hash moved"; exit 1; }
noprobe_enforce union
NA=0; NB=0
run_arm() { # A|B
  head_unmoved
  if [ "$1" = A ]; then
    NA=$((NA+1))
    PHPR_PURGE_DELAY=0 W=$NC bash "$M78" axum "86p.pa.r$NA" hello.php 100 || fail "VP-ABBA A r$NA rc"
  else
    NB=$((NB+1))
    PHPR_PURGE_DELAY=default W=$NC bash "$M78" axum "86p.pd.r$NB" hello.php 100 || fail "VP-ABBA B r$NB rc"
  fi
}
for blk in 1 2 3 4; do run_arm A; run_arm B; run_arm B; run_arm A; done
run_arm A; run_arm B

echo "== measure86-campaign DONE fails=$FAILS =="
touch "$OUT/m86.done"
exit $FAILS
