#!/bin/bash
# gate-concurrent.sh — KH78-1 per contract A-SK6 (Council WP-79, KS-SK-79.3:
# NO census may start without a conforming verdict from THIS gate).
#
# Contract (Klabnik A-SK6):
#   - W >= 2 workers (here: 4)
#   - >= 3xW requests IN FLIGHT CONCURRENTLY (here: 24 = 6xW, mixed fixtures)
#   - EVERY response body cmp'd against its oracle-generated expected file
#     (full body — never self-equality, KS-SK-79.2)
#   - server log grepped for panics
#   - PASS declared only with command + binary hash (+ git rev, A-SK9)
# G-APERTURA-2 stays sequential; THIS gate is the concurrency surface
# (statics must restart per request on EVERY worker, no cross-talk).
#
# S-79.0.5 (Council WP-80 A-SK1, KS-SK-80-1): the volley alone did not PROVE
# overlap — one worker draining 24 queued requests produced the same PASS.
# Overlap observable (instrumentation-free, works on the union twin):
# wall-clock over a volley of fixed-sleep requests. 8 × 400ms over W=4 must
# complete well under the 3.2s serial floor; the SAME volley over W=1 must
# take >= the serial floor (discriminator control: proves the observable can
# tell the shapes apart — a fixture that failed to sleep dies HERE).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
FIX="$HERE/gate-axum/fixtures"
OUTDIR="$HERE/gate-axum/out/concurrent"
mkdir -p "$OUTDIR"
PORT="${PORT:-8197}"
WORKERS=4
ROUNDS=6   # requests per fixture-slot: total = ROUNDS * 4 fixtures = 24 = 6xW
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
VERDICT="$HERE/gate-axum/out/kh78_1_concurrent.verdict"
[ -f "$VERDICT" ] && mv "$VERDICT" "$VERDICT.superseded-$(date +%Y%m%d%H%M%S)"
FAILS=0

find "$FIX" -name '._*' -delete

echo "== build -p php-server --features axum-server (json artifact capture) =="
( cd "$REPO" && cargo build --release -p php-server --features axum-server \
    --message-format=json ) > "$OUTDIR/build.json" 2> "$OUTDIR/build.log"
if [ $? -ne 0 ]; then
  echo "FAIL: build failed"; echo "FAIL build [git $GIT_REV]" > "$VERDICT"; exit 2
fi
BIN="$(python3 - "$OUTDIR/build.json" <<'PY'
import json, sys
path = ""
for line in open(sys.argv[1]):
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("reason") == "compiler-artifact" \
       and m.get("target", {}).get("name") == "php-server" \
       and m.get("executable"):
        path = m["executable"]
print(path)
PY
)"
if [ -z "$BIN" ] || [ ! -x "$BIN" ]; then
  echo "FAIL: no executable artifact (KS-AH-78-5)"
  echo "FAIL artifact [git $GIT_REV]" > "$VERDICT"; exit 2
fi
HASH=$(shasum -a 256 "$BIN" | cut -c1-16)
echo "php-server sha256[0:16]=$HASH workers=$WORKERS"

ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
if [ ! -x "$ORACLE" ]; then
  echo "FAIL: oracle missing"; echo "FAIL oracle [git $GIT_REV]" > "$VERDICT"; exit 2
fi
FIXTURES=(hello gate_stateful include_gate static_methods)
for fx in "${FIXTURES[@]}"; do
  "$ORACLE" "$FIX/$fx.php" > "$OUTDIR/$fx.expected"
done

pkill -f "php-server --axum --port $PORT" 2>/dev/null && sleep 1
"$BIN" --axum --port "$PORT" --workers "$WORKERS" -t "$FIX" \
  > /dev/null 2> "$OUTDIR/server.log" &
SRV=$!
up=0
for _ in $(seq 1 100); do
  if curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php"; then up=1; break; fi
  sleep 0.1
done
if [ "$up" != 1 ]; then
  echo "FAIL: server did not come up"; kill "$SRV" 2>/dev/null
  echo "FAIL server-up [git $GIT_REV]" > "$VERDICT"; exit 2
fi
LPID="$(lsof -nP -tiTCP:"$PORT" -sTCP:LISTEN | head -1)"
if [ "$LPID" != "$SRV" ]; then
  echo "FAIL: PID on port ($LPID) != SRV ($SRV) — run NULLA (KS-SK-79.1)"
  kill "$SRV" 2>/dev/null; echo "FAIL pid [git $GIT_REV]" > "$VERDICT"; exit 2
fi

# --- concurrent volley: 24 requests all in flight at once -------------------
echo "== volley: $((ROUNDS * ${#FIXTURES[@]})) concurrent requests over $WORKERS workers =="
PIDS=()
for r in $(seq 1 "$ROUNDS"); do
  for fx in "${FIXTURES[@]}"; do
    curl -s -m 15 -X POST "http://127.0.0.1:$PORT/$fx.php" \
      -o "$OUTDIR/$fx.r$r" &
    PIDS+=($!)
  done
done
CURL_FAIL=0
for p in "${PIDS[@]}"; do wait "$p" || CURL_FAIL=$((CURL_FAIL+1)); done
[ "$CURL_FAIL" -gt 0 ] && { echo "FAIL: $CURL_FAIL curl transfers failed"; FAILS=$((FAILS+1)); }

# --- verify every body against its oracle expected (full-body cmp) ----------
BAD=0
for r in $(seq 1 "$ROUNDS"); do
  for fx in "${FIXTURES[@]}"; do
    if ! cmp -s "$OUTDIR/$fx.r$r" "$OUTDIR/$fx.expected"; then
      echo "FAIL $fx round $r: body != oracle expected:"
      diff "$OUTDIR/$fx.expected" "$OUTDIR/$fx.r$r" | head -4
      BAD=$((BAD+1))
    fi
  done
done
if [ "$BAD" = 0 ]; then
  echo "OK  all $((ROUNDS * ${#FIXTURES[@]})) concurrent bodies == oracle expected"
else
  FAILS=$((FAILS+BAD))
fi

# --- A-SK1 overlap observable (KS-SK-80-1) ----------------------------------
"$ORACLE" "$FIX/sleep_gate.php" > "$OUTDIR/sleep_gate.expected"
now_s() { python3 -c 'import time; print(f"{time.time():.3f}")'; }
sleep_volley() { # sleep_volley <port> <n> -> elapsed seconds on stdout
  local port="$1" n="$2" t0 t1 pids=() i
  t0=$(now_s)
  for i in $(seq 1 "$n"); do
    curl -s -m 20 "http://127.0.0.1:$port/sleep_gate.php" -o "$OUTDIR/sleep.$port.$i" &
    pids+=($!)
  done
  for p in "${pids[@]}"; do wait "$p" || true; done
  t1=$(now_s)
  python3 -c "print(f'{$t1 - $t0:.3f}')"
  for i in $(seq 1 "$n"); do
    if ! cmp -s "$OUTDIR/sleep.$port.$i" "$OUTDIR/sleep_gate.expected"; then
      echo "BODY-FAIL sleep req $i" >&2
    fi
  done
}
echo "== A-SK1 overlap: 8 x 400ms sleep over W=$WORKERS =="
ELAPSED=$(sleep_volley "$PORT" 8 2> "$OUTDIR/sleep-bodyfail.log")
if [ -s "$OUTDIR/sleep-bodyfail.log" ]; then
  cat "$OUTDIR/sleep-bodyfail.log"; FAILS=$((FAILS+1))
fi
if python3 -c "import sys; sys.exit(0 if $ELAPSED < 1.7 else 1)"; then
  echo "OK  overlap PROVEN: 8x400ms in ${ELAPSED}s < 1.7s (serial floor 3.2s) — >=2 workers concurrent"
else
  echo "FAIL: 8x400ms took ${ELAPSED}s — no overlap observed (KS-SK-80-1: PASS would be vacuous)"
  FAILS=$((FAILS+1))
fi

# --- teardown + panic sweep -------------------------------------------------
kill -TERM "$SRV" 2>/dev/null
wait "$SRV" 2>/dev/null
if grep -iE "panic|aborting" "$OUTDIR/server.log"; then
  echo "FAIL: panic in server log (KS-PP-6)"; FAILS=$((FAILS+1))
else
  echo "OK  no panic in server log"
fi
if grep -q "workers joined" "$OUTDIR/server.log"; then
  echo "OK  KL-78-4 join line present after SIGTERM"
else
  echo "FAIL: join line missing after SIGTERM (KL-78-4)"; FAILS=$((FAILS+1))
fi

# --- A-SK1 discriminator control: same volley on W=1 must be SERIAL ---------
CPORT=$((PORT + 1))
pkill -f "php-server --axum --port $CPORT" 2>/dev/null && sleep 1
"$BIN" --axum --port "$CPORT" --workers 1 -t "$FIX" \
  > /dev/null 2> "$OUTDIR/server-w1.log" &
SRV1=$!
up=0
for _ in $(seq 1 100); do
  if curl -s -o /dev/null -m 1 "http://127.0.0.1:$CPORT/hello.php"; then up=1; break; fi
  sleep 0.1
done
if [ "$up" != 1 ]; then
  echo "FAIL: W=1 control server did not come up"; kill "$SRV1" 2>/dev/null
  FAILS=$((FAILS+1))
else
  ELAPSED1=$(sleep_volley "$CPORT" 8 2>> "$OUTDIR/sleep-bodyfail.log")
  if python3 -c "import sys; sys.exit(0 if $ELAPSED1 >= 3.0 else 1)"; then
    echo "OK  discriminator: W=1 serialized (8x400ms in ${ELAPSED1}s >= 3.0s) — the observable can tell the shapes apart"
  else
    echo "FAIL: W=1 volley took only ${ELAPSED1}s — the overlap observable cannot discriminate (fixture not sleeping?)"
    FAILS=$((FAILS+1))
  fi
  kill -TERM "$SRV1" 2>/dev/null
  wait "$SRV1" 2>/dev/null
fi

if [ "$FAILS" = 0 ]; then
  echo "PASS fails=0 bin=$HASH git=$GIT_REV W=$WORKERS reqs=$((ROUNDS * ${#FIXTURES[@]})) overlap=${ELAPSED:-n/a}s w1_serial=${ELAPSED1:-n/a}s $(date +%F_%H:%M:%S) (gate-concurrent.sh)" > "$VERDICT"
  echo "== KH78-1 PASS (bin=$HASH git=$GIT_REV) =="; exit 0
else
  echo "FAIL fails=$FAILS bin=$HASH git=$GIT_REV" > "$VERDICT"
  echo "== KH78-1 FAIL($FAILS) bin=$HASH git=$GIT_REV =="; exit 1
fi
