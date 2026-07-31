#!/bin/bash
# measure82-campaign.sh — S-82.0 p7: the RESIDUAL A/B measures of the A-BB6
# lever (Council WP-83 §Sintesi step 7). Sequential, ENFORCE, one rev.
#   Phase F: footprint twin V2 N=100 vs N=200 (union twin, W=1, R=3) —
#            no-leak floor A-DL19(a): |V2(2N)−V2(N)| <= 0.1 MB.
#   Phase P: peak at W=num_cpus (union twin, R=3) + base arm R=3 (pre-lever
#            7593d8e, worktree --locked, external target) — A-DL19(b).
#   Phase C: CPU slope two-N, N=1000/2000 (KB), W=1, hello, R=3 per (arm,N):
#            slope_lever <= slope_base*1.05 + 25us/req; risoluzione ex-ante
#            spread(R=3 slopes) < banda/3 else N raddoppia (KB-83-3).
#   Phase H: battery-su-HIT TWIN-PAIR (A-SK25/KS-PP-83-2/KS-SK-83-3):
#            run B = union+uclog (put==0/hit==nreq/probe==nreq nel segmento);
#            run A = census witness (steady a_calls <= floor). Binari NOMINATI.
#   Phase R: retained ×W (KL-83-1): mem-census binary — full-body battery vs
#            oracle PRIMA (A-AH29/KS-AH-83-1), poi tag=unitcache: rw_bytes
#            (FLOOR) + rw_main_net (net-at-lower) + main_evicted DICHIARATO
#            (KS-DS-83-1) + uclog==0 (KS-PP-83-1).
#   Phase A: autoload-run fixture (KB-82-5): quota RUN dentro a3 misurata.
# Binari per fase: il driver measure78 ENFORCE l'hash contro la riga matrix —
# la campagna RICOSTRUISCE il binario giusto prima di ogni gruppo e RESTAURA
# union alla fine. KH83-2: void_runs contate; mai rm di raw (quarantine).
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
BASEWT="/Volumes/Extreme Pro/Claude/wp82-basewt"
BASE_TARGET="/Volumes/Extreme Pro/Claude/phpr-old-target"
export PHPR_CAMPAIGN_SCRIPT="$HERE/measure82-campaign.sh"   # A-AH30
VOID_RUNS=0
FAILS=0
NC=$(sysctl -n hw.ncpu)
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"

echo "== measure82-campaign git=$GIT_REV ncpu=$NC void_runs=$VOID_RUNS (KH83-2; final count at end) =="
fail() { echo "FAIL: $*"; FAILS=$((FAILS+1)); }

build_arm() { # <features>  (rebuild php-server; matrix row must match)
  ( cd "$REPO" && cargo build --release -p php-server --features "$1" ) \
    > "$OUT/m82.build.$1.log" 2>&1 || { echo "FAIL: build $1"; exit 1; }
}

# --- preconditions ------------------------------------------------------------
BATT="/Volumes/Extreme Pro/Claude/wp82-battery-out"
[ -f "$BATT/.done" ] || { echo "FAIL: battery-82pre not run"; exit 1; }
if ! grep -q "BATTERY-82PRE PASS" "$BATT/battery-82pre.out"; then
  # KH82-2 equivalence path (machine-checked, never a hand-written PASS):
  # >=14 OK gates in the battery run + a FEATURE-MATRIX PASS at HEAD in
  # feature-matrix-final.log, valid only for harness-only rev deltas
  # (crates-diff empty is what the matrix at HEAD itself certifies).
  NOK=$(grep -cE '^OK' "$BATT/battery-82pre.out" || echo 0)
  if [ "$NOK" -ge 14 ] && grep -q "== FEATURE-MATRIX PASS" "$BATT/feature-matrix-final.log" 2>/dev/null; then
    echo "battery equivalence: $NOK/15 OK + matrix-final PASS at HEAD (KH82-2)"
  else
    echo "FAIL: battery-82pre not PASS (OK=$NOK, no matrix-final PASS)"; exit 1
  fi
fi
DIRTY="$(git -C "$REPO" status --porcelain -- crates Cargo.toml Cargo.lock \
  'wp7*-harness/*.sh' 'wp7*-harness/*.pl' 'wp8*-harness/*.sh' 'wp8*-harness/*.pl' 2>/dev/null)"
[ -z "$DIRTY" ] || { echo "FAIL: tree/harness dirty (KS-AH-81-1/A-AH21)"; echo "$DIRTY" | head -5; exit 1; }
for f in autoload82.php autocls82_al82.php; do
  [ -f "$FIXDIR/$f" ] || { echo "FAIL: committed fixture missing: $f"; exit 1; }
done

# --- union arm phases (F, P, C-lever, H-runB) ---------------------------------
build_arm axum-server
UNION_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
echo "union php-server=$UNION_HASH"

echo "== Phase F: footprint twin (union, W=1, R=3, N=100/200) =="
for N in 100 200; do for R in 1 2 3; do
  bash "$M78" axum "82f.n$N.r$R" hello.php "$N" || fail "phaseF n$N r$R rc"
done; done

echo "== Phase P: peak W=$NC (union, R=3) =="
for R in 1 2 3; do
  W=$NC bash "$M78" axum "82p.w$NC.r$R" hello.php 100 || fail "phaseP r$R rc"
done

echo "== Phase C: CPU slope lever arm (N=1000/2000, R=3) =="
for N in 1000 2000; do for R in 1 2 3; do
  bash "$M78" axum "82c.lever.n$N.r$R" hello.php "$N" || fail "phaseC lever n$N r$R rc"
done; done

echo "== Phase H run B: union + uclog twin (KS-PP-83-2) =="
HLOG="$OUT/m82.hitpair.uclog"
: > "$HLOG"
HFIX="hello.php include_gate.php include_heavy.php bare.php"
PORT=8298
PHPR_UNIT_CACHE_LOG="$HLOG" "$OUTBIN/php-server" --axum --workers 1 --port $PORT -t "$FIXDIR" \
  > /dev/null 2> "$OUT/m82.hitpair.b.log" &
BPID=$!
up=0; for _ in $(seq 1 100); do curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php" && { up=1; break; }; sleep 0.1; done
[ $up = 1 ] || fail "phaseH runB server not up"
for f in $HFIX; do curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/$f"; done   # warm 1 req/file
MARK=$(wc -l < "$HLOG" 2>/dev/null | tr -d ' '); MARK=${MARK:-0}
NREQ=20
for f in $HFIX; do for _ in $(seq 1 $NREQ); do curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/$f"; done; done
kill -TERM "$BPID" 2>/dev/null; wait "$BPID" 2>/dev/null
tail -n +"$((MARK+1))" "$HLOG" > "$OUT/m82.hitpair.segment"
echo "union_hash=$UNION_HASH nreq=$NREQ warm_lines=$MARK campaign=$PHPR_CAMPAIGN_SCRIPT" > "$OUT/m82.hitpair.meta"

# --- base arm (pre-lever 7593d8e): peak + CPU slope ---------------------------
echo "== Base arm build: $BASE_REV (--locked, external target, WP-65 rule) =="
if [ ! -d "$BASEWT" ]; then
  git -C "$REPO" worktree add "$BASEWT" "$BASE_REV" > /dev/null 2>&1 || { echo "FAIL: worktree add"; exit 1; }
fi
cp "$REPO/Cargo.lock" "$BASEWT/Cargo.lock"
( cd "$BASEWT" && CARGO_TARGET_DIR="$BASE_TARGET" cargo build --release --locked -p php-server --features axum-server ) \
  > "$OUT/m82.base-build.log" 2>&1 || { echo "FAIL: base build"; exit 1; }
BASE_BIN="$BASE_TARGET/release/php-server"
BASE_HASH=$(shasum -a 256 "$BASE_BIN" | cut -c1-16)
echo "base php-server=$BASE_HASH (rev $BASE_REV)"
base_run() { # <label> <W> <N>  — same loop shape as measure78 axum, DECLARED
  local label="$1" W="$2" N="$3" PORT=8297
  local RUN="$OUT/m82.base.$label"
  /usr/bin/time -l "$BASE_BIN" --axum --workers "$W" --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$RUN.log" &
  local TPID=$!
  local up=0
  for _ in $(seq 1 100); do
    curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php" && { up=1; break; }
    sleep 0.1
  done
  [ $up = 1 ] || { fail "base $label: server not up"; kill -TERM $TPID 2>/dev/null; return; }
  for _ in $(seq 1 10); do curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/hello.php"; done
  local SRVPID
  SRVPID=$(pgrep -P "$TPID" | head -1); [ -z "$SRVPID" ] && SRVPID=$TPID
  vmmap "$SRVPID" 2>/dev/null > "$RUN.vmmap.V1"
  for _ in $(seq 1 "$N"); do curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/hello.php"; done
  vmmap "$SRVPID" 2>/dev/null > "$RUN.vmmap.V2"
  kill -TERM "$SRVPID" 2>/dev/null; wait "$TPID" 2>/dev/null
  echo "base_rev=$BASE_REV base_hash=$BASE_HASH label=$label W=$W N=$N campaign=$PHPR_CAMPAIGN_SCRIPT" >> "$RUN.log"
}
echo "== Phase P/C base arm (R=3 each) =="
for R in 1 2 3; do base_run "82p.w$NC.r$R" "$NC" 100; done
for N in 1000 2000; do for R in 1 2 3; do base_run "82c.n$N.r$R" 1 "$N"; done; done

# --- census arm phases (H-runA witness + A autoload) --------------------------
build_arm census-instrumentation
echo "== Phase H run A: census HIT witness (MEASURED=30 per fixture) =="
for f in hello include_gate include_heavy bare; do
  bash "$M78" census "82h.$f" "$f.php" 30 || fail "phaseH census $f rc"
done
echo "== Phase A: autoload-run a3 bound (census arm, MEASURED=30) =="
bash "$M78" census "82a.autoload" "autoload82.php" 30 || fail "phaseA rc"

# --- Phase R: retained on the mem-census binary -------------------------------
echo "== Phase R: retained (mem-census binary, full-body FIRST) =="
build_arm mem-census
MEM_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
PORT=8299
"$OUTBIN/php-server" --axum --workers 1 --port $PORT -t "$FIXDIR" \
  > /dev/null 2> "$OUT/m82.retained.log" &
MPID=$!
up=0; for _ in $(seq 1 100); do curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php" && { up=1; break; }; sleep 0.1; done
[ $up = 1 ] || fail "phaseR server not up"
RBODYFAIL=0
for f in $HFIX; do
  want=$(cd "$FIXDIR" && "$ORACLE" -n "$f" 2>/dev/null)
  for _ in 1 2 3; do
    got=$(curl -s -m 10 "http://127.0.0.1:$PORT/$f")
    [ "$got" = "$want" ] || { RBODYFAIL=1; fail "phaseR full-body $f != oracle (KS-AH-83-1)"; break; }
  done
done
[ $RBODYFAIL = 0 ] && echo "OK phaseR: full-body battery vs oracle PASS on mem-census binary ($MEM_HASH)"
for f in $HFIX; do for _ in $(seq 1 5); do curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/$f"; done; done
kill -TERM "$MPID" 2>/dev/null; wait "$MPID" 2>/dev/null
echo "mem_hash=$MEM_HASH campaign=$PHPR_CAMPAIGN_SCRIPT" >> "$OUT/m82.retained.log"

# --- restore the union binary on disk (leave the tree in twin state) ----------
build_arm axum-server
FINAL_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
[ "$FINAL_HASH" = "$UNION_HASH" ] || fail "union restore hash moved ($FINAL_HASH != $UNION_HASH)"

echo "== measure82-campaign DONE void_runs=$VOID_RUNS fails=$FAILS =="
touch "$OUT/m82.done"
exit $FAILS
