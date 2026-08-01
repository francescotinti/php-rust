#!/bin/bash
# measure84-campaign.sh — S-84.0: the DECISIVE measures for the peak
# deliberation (Council WP-85 §Sintesi point 5). Sequential, ENFORCE, one
# rev. NO slope phase this campaign (VC is CLOSED verdict-grade ~55-75x,
# A-BB37); the next slope campaign must carry A-PP28 (w= in-band, cabled) +
# A-BG33 (base per-request via curl -w) + A-BG35 (regime defined).
#
#   Phase P    (A-BB34 rerun, KB-85-2): union twin, W=ncpu, hello, N=100,
#              R=3 — the 232±1MB W=10 identity pin re-executed
#              POST-partition. NO peak ×W figure is publishable without
#              this (KB-85-2); the verdict names the outcome either way.
#   Phase DL24 (A-DL24, KL-85-1 — the measure that DECIDES the ×W budget
#              formula): mem-census binary ARMED, axum W=2, ONE hello
#              request per distinct worker (round-robin, sequential),
#              graceful SIGTERM -> workers joined -> per-THREAD
#              tag=unitcache_main_entry rows (thr= in-band) at teardown.
#              net(ord=1) of the SECOND thread ~7.35MB => per-thread
#              (budget ×W holds); ~8KB => per-process (budget rewrites to
#              residue + W×(budget−residue)).
#   Phase A2   (VA remeasure — A-DS29/KS-DS-85-1): full-body vs oracle of
#              EVERY counted fixture on the UNION arm at THIS rev,
#              LEDGERED (evidence/fixture-oracle.ledger, tracked, append
#              committed in-campaign), BEFORE the counted window; then
#              census arm MEASURED=30 on autoload82/autoload83_reg/hello.
#   Phase W2   (VW bisection — A-BB38/KB-85-3): census arm, MEASURED=30 on
#              the TWO committed fixtures at canonical path len 383/384.
#              Site NAMED in code: std run_path_with_cstr stack buffer
#              MAX_STACK_ALLOCATION=384 — the two extra NUL-terminated
#              copies are the heap CString fallbacks of the TWO path
#              syscalls in main_unit_key (fs::canonicalize + fs::metadata).
#              Piecewise model under test: a_calls = 2 + 2·[len>=384],
#              a_bytes = 2·len + 2·(len+1)·[len>=384].
#
# Preconditions: battery-84pre PASS at THIS rev (per NAME; equivalence only
# via battery-equivalence.sh v2 — OUT↔BREV bound, canonical ledger), matrix
# at HEAD (in-battery, feature-matrix LAST-built pre-campaign), tree +
# harness porcelain clean. Raws NEVER rm'd: quarantine + manifest
# (KS-AH-83-2). No new twin-pair segment this campaign (A-PP27/KS-PP-85-3
# not engaged, DECLARED).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
M78="$REPO/wp78-harness/measure78.sh"
OUT="$REPO/wp78-harness/measure-out"
OUTBIN="$HOME/Claude/php-rust-output/release"
ORACLE="/opt/homebrew/opt/php/bin/php"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
FLEDGER="$HERE/evidence/fixture-oracle.ledger"
export PHPR_CAMPAIGN_SCRIPT="$HERE/measure84-campaign.sh"   # A-AH30
FAILS=0
NC=$(sysctl -n hw.ncpu)
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
HEAD_AT_START="$GIT_REV"
fail() { echo "FAIL: $*"; FAILS=$((FAILS+1)); }

# VW bisection fixture rel paths (committed; lengths are runtime-derived
# from the same padding rule that built them).
D1="vw84_bisect_$(printf 'a%.0s' $(seq 1 88))"
D2="vw84_pad_$(printf 'b%.0s' $(seq 1 91))"
N383="f$(printf 'c%.0s' $(seq 1 87)).php"
N384="f$(printf 'c%.0s' $(seq 1 88)).php"
REL383="$D1/$D2/$N383"
REL384="$D1/$D2/$N384"

echo "== measure84-campaign git=$GIT_REV ncpu=$NC (peak-deliberation measures) =="

# --- preconditions ------------------------------------------------------------
BATT="/Volumes/Extreme Pro/Claude/wp84-battery-out"
if [ ! -f "$BATT/.done" ]; then echo "FAIL: battery-84pre not run"; exit 1; fi
BATT_REV=$(awk -F= '/^rev=/{print $2}' "$BATT/.done")
if grep -q "BATTERY-84PRE PASS" "$BATT/battery-84pre.out" 2>/dev/null && [ "$BATT_REV" = "$GIT_REV" ]; then
  echo "battery-84pre PASS at rev $BATT_REV == HEAD (per NAME, 15/15)"
else
  # A-SK32/A-SK33: the ONLY legal equivalence path — v2 usage (canonical
  # ledger pinned in the script, no ledger argument).
  if bash "$REPO/wp83-harness/battery-equivalence.sh" "$BATT/battery-84pre.out" "$BATT_REV"; then
    echo "battery equivalence LEGAL ($BATT_REV certifies $GIT_REV)"
  else
    echo "FAIL: battery not PASS at HEAD and equivalence REFUSED"; exit 1
  fi
fi
DIRTY="$(git -C "$REPO" status --porcelain -- crates Cargo.toml Cargo.lock \
  'wp7*-harness/*.sh' 'wp7*-harness/*.pl' 'wp8*-harness/*.sh' 'wp8*-harness/*.pl' 2>/dev/null)"
[ -z "$DIRTY" ] || { echo "FAIL: tree/harness dirty (KS-AH-81-1/A-AH21)"; echo "$DIRTY" | head -5; exit 1; }
for f in autoload82.php autoload83_reg.php hello.php "$REL383" "$REL384"; do
  [ -f "$FIXDIR/$f" ] || { echo "FAIL: committed fixture missing: $f"; exit 1; }
done
[ -f "$FLEDGER" ] || { echo "FAIL: fixture-oracle ledger missing (A-DS29 — tracked file, never touch-create)"; exit 1; }
git -C "$REPO" ls-files --error-unmatch "wp84-harness/evidence/fixture-oracle.ledger" >/dev/null 2>&1 \
  || { echo "FAIL: fixture-oracle ledger UNTRACKED (A-DS29)"; exit 1; }

build_arm() { # <features>
  ( cd "$REPO" && cargo build --release -p php-server --features "$1" ) \
    > "$OUT/m84.build.$1.log" 2>&1 || { echo "FAIL: build $1"; exit 1; }
}
head_unmoved() { # KG-84-2 chain refusal
  local now; now="$(git -C "$REPO" rev-parse --short HEAD)"
  [ "$now" = "$HEAD_AT_START" ] || { echo "FAIL: HEAD moved mid-campaign ($now != $HEAD_AT_START, KG-81-2/KG-84-2)"; exit 1; }
}

# --- Phase P: A-BB34 rerun (union, W=ncpu, R=3) --------------------------------
build_arm axum-server
UNION_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
echo "union php-server=$UNION_HASH"
echo "== Phase P: peak W=$NC (union, R=3) — A-BB34 rerun post-partition =="
for R in 1 2 3; do
  head_unmoved
  W=$NC bash "$M78" axum "84p.w$NC.r$R" hello.php 100 || fail "phaseP r$R rc"
done

# --- Phase DL24: per-thread first-lower attribution (mem-census, W=2) ----------
echo "== Phase DL24: hello-only W=2, per-thread rows (mem-census ARMED) =="
build_arm mem-census
MEM_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
MC="$OUT/m84.dl24.memcensus"
: > "$MC"
PORT=8296
head_unmoved
PHPR_MEM_CENSUS="$MC" "$OUTBIN/php-server" --axum --workers 2 --port $PORT -t "$FIXDIR" \
  > /dev/null 2> "$OUT/m84.dl24.log" &
DPID=$!
up=0
for _ in $(seq 1 100); do
  # A-PP32 (Council WP-86): the request->worker mapping claim is ADVISORY
  # only — a REFUSED connect never dispatches, but a connect that succeeds
  # and then times out on -m 1 HAS dispatched (fetch_add consumed).
  # Attribution is NEVER taken from request order: only thr= in-band rows
  # count (KS-PP-86-3), and the verdict fail-closes on thread cardinality.
  curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php" && { up=1; break; }
  sleep 0.1
done
[ $up = 1 ] || { fail "phaseDL24 server not up"; kill -TERM $DPID 2>/dev/null; }
WANT=$(cd "$FIXDIR" && "$ORACLE" -n hello.php 2>/dev/null)
B1=$(curl -s -m 10 "http://127.0.0.1:$PORT/hello.php")   # request 2 -> worker 1 (its ord=1)
[ "$B1" = "$WANT" ] || fail "phaseDL24 full-body request-2 != oracle (KS-AH-83-1)"
B2=$(curl -s -m 10 "http://127.0.0.1:$PORT/hello.php")   # request 3 -> worker 0 (HIT)
[ "$B2" = "$WANT" ] || fail "phaseDL24 full-body request-3 != oracle (KS-AH-83-1)"
kill -TERM "$DPID" 2>/dev/null; wait "$DPID" 2>/dev/null
NROWS=$(grep -c "tag=unitcache_main_entry thr=" "$MC" || true)
NTHR=$(grep "tag=unitcache_thr " "$MC" | wc -l | tr -d ' ')
[ "$NROWS" -ge 2 ] || fail "phaseDL24: $NROWS per-thread main rows (want >=2 — teardown dump missing?)"
[ "$NTHR" -ge 2 ]  || fail "phaseDL24: $NTHR thread summary rows (want 2)"
echo "mem_hash=$MEM_HASH git=$GIT_REV campaign=$PHPR_CAMPAIGN_SCRIPT arm=axum-worker w=2 phase=dl24" >> "$MC"

# --- Phase A2: VA remeasure with per-fixture oracle ledger (A-DS29) ------------
echo "== Phase A2: per-fixture oracle on the UNION arm, LEDGERED, then census =="
build_arm axum-server
U2=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
[ "$U2" = "$UNION_HASH" ] || fail "union rebuild hash moved ($U2 != $UNION_HASH)"
PORT=8295
head_unmoved
"$OUTBIN/php-server" --axum --workers 1 --port $PORT -t "$FIXDIR" \
  > /dev/null 2> "$OUT/m84.a2oracle.log" &
APID=$!
up=0; for _ in $(seq 1 100); do curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php" && { up=1; break; }; sleep 0.1; done
[ $up = 1 ] || { fail "phaseA2 oracle server not up"; kill -TERM $APID 2>/dev/null; }
for fx in autoload82.php autoload83_reg.php hello.php "$REL383" "$REL384"; do
  want=$(cd "$FIXDIR" && "$ORACLE" -n "$fx" 2>/dev/null)
  okfx=1
  for _ in 1 2 3; do
    got=$(curl -s -m 10 "http://127.0.0.1:$PORT/$fx")
    [ "$got" = "$want" ] || { okfx=0; fail "phaseA2 full-body $fx != oracle (A-DS29/KS-DS-85-1)"; break; }
  done
  if [ "$okfx" = 1 ]; then
    bsha=$(printf '%s' "$want" | shasum -a 256 | cut -c1-16)
    echo "fixture=$fx rev=$GIT_REV arm=union union_hash=$UNION_HASH body_sha=$bsha verdict=PASS" >> "$FLEDGER"
    echo "OK  A-DS29 oracle ledgered: $fx"
  fi
done
kill -TERM "$APID" 2>/dev/null; wait "$APID" 2>/dev/null

echo "== Phase A2 counted window (census arm, MEASURED=30) =="
build_arm census-instrumentation
for fx in autoload82 autoload83_reg hello; do
  head_unmoved
  bash "$M78" census "84a.$fx" "$fx.php" 30 || fail "phaseA2 census $fx rc"
done

# --- Phase W2: VW bisection at the named site's threshold ----------------------
echo "== Phase W2: bisection len=383/384 (census arm, MEASURED=30) =="
# A-BG37 (Council WP-86): head_unmoved before EVERY driver invocation —
# one refusal must never cover two runs.
head_unmoved
bash "$M78" census "84w.len383" "$REL383" 30 || fail "phaseW2 len383 rc"
head_unmoved
bash "$M78" census "84w.len384" "$REL384" 30 || fail "phaseW2 len384 rc"

# --- restore the union binary on disk ------------------------------------------
build_arm axum-server
FINAL_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
[ "$FINAL_HASH" = "$UNION_HASH" ] || fail "union restore hash moved ($FINAL_HASH != $UNION_HASH)"

echo "== measure84-campaign DONE fails=$FAILS =="
touch "$OUT/m84.done"
exit $FAILS
