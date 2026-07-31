#!/bin/bash
# G-APERTURA-2 executable gate (A-SK2, Council WP-78) — KS-SK-78.1 / KS-SK-78.2 / KS-DS-78-1.
#
# Contract (Klabnik A-SK2 + Stogov A-DS3):
#   1. Build php-server WITH the axum-server feature (the deployed binary must
#      contain the code under test — the WP-77.6.5.2.4 "PASS" was vacuous
#      precisely because cargo test never compiled worker_pool.rs).
#   2. Record shasum-256 of the php-server binary (A-SK5: the Axum baseline is
#      php-server, NEVER phpr).
#   3. Start the server in --axum mode, run sequential POST requests:
#        - hello.php          x2  → byte-identical bodies (cmp)
#        - gate_stateful.php  x3  → byte-identical bodies (cmp) AND each equal to
#          the EXPECTED line (positive control, WP-72 lesson: all-identical-but-
#          wrong must not pass). Statics/static-props/closure-statics/define()
#          must restart per request (FPM semantics) → KS-DS-78-1 on drift.
#   4. Deterministic exit code: 0 = PASS, 1 = FAIL.
#
# A PASS of G-APERTURA-2 may only be declared by citing THIS command and the
# binary hash it logs.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
BIN="${BIN:-$HOME/Claude/php-rust-output/release/php-server}"
OUTDIR="$HERE/out"
mkdir -p "$OUTDIR"
PORT="${PORT:-8199}"
VERDICT="$OUTDIR/g_apertura_2.verdict"
[ -f "$VERDICT" ] && mv "$VERDICT" "$VERDICT.superseded-$(date +%Y%m%d%H%M%S)"
FAILS=0

# 1. Build with the feature (union with default is fine for the dual-mode binary;
#    the feature identity of the DEFAULT binary is gated by gate-feature-matrix.sh).
echo "== build -p php-server --features axum-server =="
( cd "$REPO" && cargo build --release -p php-server --features axum-server ) \
  > "$OUTDIR/build.log" 2>&1
if [ $? -ne 0 ]; then
  echo "FAIL: build failed (see $OUTDIR/build.log)"
  echo "FAIL build" > "$VERDICT"; exit 1
fi

# 2. Binary identity (A-SK5)
HASH=$(shasum -a 256 "$BIN" | cut -c1-16)
echo "php-server sha256[0:16]=$HASH"

# Kill stale servers on our port
pkill -f "php-server --axum --port $PORT" 2>/dev/null && sleep 1

# 3. Start server
"$BIN" --axum --port "$PORT" -t "$HERE/fixtures" \
  > /dev/null 2> "$OUTDIR/server.log" &
SRV=$!
# Readiness poll (max 10s) — no blind sleep
up=0
for _ in $(seq 1 100); do
  if curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php"; then up=1; break; fi
  sleep 0.1
done
if [ "$up" != 1 ]; then
  echo "FAIL: server did not come up (see $OUTDIR/server.log)"
  kill "$SRV" 2>/dev/null; echo "FAIL server-up" > "$VERDICT"; exit 1
fi

req() { curl -s -m 5 -X POST "http://127.0.0.1:$PORT/$1"; }

# hello.php x2 — byte parity (KS-SK-78.2)
req hello.php > "$OUTDIR/hello.1"
req hello.php > "$OUTDIR/hello.2"
if cmp -s "$OUTDIR/hello.1" "$OUTDIR/hello.2"; then
  echo "OK  hello: 2 bodies byte-identical"
else
  echo "FAIL hello: bodies differ (KS-SK-78.2 → REJECT S-77.6.5.2.3)"
  FAILS=$((FAILS+1))
fi
if [ ! -s "$OUTDIR/hello.1" ]; then
  echo "FAIL hello: empty body"
  FAILS=$((FAILS+1))
fi

# gate_stateful.php x3 — byte parity + positive control (KS-DS-78-1)
EXPECTED='G3:fn=12;prop=12;closure=12;const=fresh;constval=1'
for i in 1 2 3; do req gate_stateful.php > "$OUTDIR/stateful.$i"; done
for i in 2 3; do
  if ! cmp -s "$OUTDIR/stateful.1" "$OUTDIR/stateful.$i"; then
    echo "FAIL stateful: request 1 vs $i differ (KS-DS-78-1)"
    FAILS=$((FAILS+1))
  fi
done
for i in 1 2 3; do
  if [ "$(head -1 "$OUTDIR/stateful.$i")" != "$EXPECTED" ]; then
    echo "FAIL stateful req $i: got '$(head -1 "$OUTDIR/stateful.$i")' expected '$EXPECTED' (KS-DS-78-1)"
    FAILS=$((FAILS+1))
  fi
done
[ "$FAILS" = 0 ] && echo "OK  stateful: 3 bodies byte-identical and == expected"

# Teardown
kill "$SRV" 2>/dev/null
wait "$SRV" 2>/dev/null

if [ "$FAILS" = 0 ]; then
  echo "PASS fails=0 bin=$HASH $(date +%F_%H:%M:%S) (run-gate.sh)" > "$VERDICT"
  echo "== G-APERTURA-2 PASS (bin=$HASH) =="; exit 0
else
  echo "FAIL fails=$FAILS bin=$HASH" > "$VERDICT"
  echo "== G-APERTURA-2 FAIL($FAILS) bin=$HASH =="; exit 1
fi
