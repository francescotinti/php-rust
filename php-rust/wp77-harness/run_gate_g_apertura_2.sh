#!/bin/bash
# G-APERTURA-2 gate runner (WP-77.6.5.2): Two sequential requests on SAME Vm instance
# must produce byte-identical output (tests request_end() reset mechanism).
#
# Against php-server (cli-server mode) serving fixtures/gate_two_reqs_same_vm.php.
# The fixture verifies:
# 1. GLOBALS state reset (should NOT find g2_marker on second request)
# 2. Static variables reset (counter = 1 on both requests)
# 3. Object IDs reset (spl_object_id = 1 on both requests)
#
# Verdict: PASS iff both outputs are byte-identical AND say "G2:PASS:object_id=1"
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
PHPSRV="${PHPSRV:-$HOME/Claude/php-rust-output/release/php-server}"
H77="$(cd "$(dirname "$0")" && pwd)"
OUTDIR="${OUTDIR:-/Volumes/Extreme Pro/Claude/wp77-harness/gate-out}"
mkdir -p "$OUTDIR"
PORT=8199
VERDICT="$OUTDIR/g_apertura_2.verdict"
[ -f "$VERDICT" ] && mv "$VERDICT" "$VERDICT.superseded-$(date +%Y%m%d%H%M%S)"
FAILS=0

# Kill any stale servers
if pgrep -f "127.0.0.1:$PORT" >/dev/null 2>&1; then
  echo "STALE SERVER on $PORT";
  pkill -f "127.0.0.1:$PORT" 2>/dev/null || true
  sleep 1
fi

DOC=$(mktemp -d)
cp "$H77/fixtures/gate_two_reqs_same_vm.php" "$DOC/g2.php"

# Start php-server (cli-server mode)
( "$PHPSRV" --port $PORT -t "$DOC" >/dev/null 2>"$OUTDIR/g_apertura_2-srv.log" ) &
SRV=$!
sleep 2

# Make TWO sequential requests to SAME endpoint
OUT1="$OUTDIR/g_apertura_2_req1.out"
OUT2="$OUTDIR/g_apertura_2_req2.out"
: > "$OUT1"
: > "$OUT2"

echo "Request 1..."
curl -s -m 5 "http://127.0.0.1:$PORT/g2.php" > "$OUT1"
echo "Request 2..."
curl -s -m 5 "http://127.0.0.1:$PORT/g2.php" > "$OUT2"

# Kill server
kill $SRV 2>/dev/null || true
pkill -f "127.0.0.1:$PORT" 2>/dev/null || true

# Verify both requests passed
pass1=$(grep -q '^G2:PASS' "$OUT1" && echo 1 || echo 0)
pass2=$(grep -q '^G2:PASS' "$OUT2" && echo 1 || echo 0)

if [ "$pass1" = 0 ] || [ "$pass2" = 0 ]; then
  echo "FAIL: Not both PASS verdicts"
  echo "Request 1: $(head -1 "$OUT1")"
  echo "Request 2: $(head -1 "$OUT2")"
  FAILS=$((FAILS+1))
fi

# Verify byte-identity
DIFF=$(diff "$OUT1" "$OUT2")
if [ -n "$DIFF" ]; then
  echo "FAIL: Outputs not byte-identical"
  echo "=== Request 1 ==="
  cat "$OUT1"
  echo "=== Request 2 ==="
  cat "$OUT2"
  echo "=== Diff ==="
  echo "$DIFF"
  FAILS=$((FAILS+1))
else
  echo "✓ Byte-identical outputs"
fi

rm -rf "$DOC"

if [ "$FAILS" = 0 ]; then
  echo "PASS fails=0 $(date +%F_%H:%M:%S) (gate_g_apertura_2.sh)" > "$VERDICT"
  echo "== G-APERTURA-2 PASS =="; exit 0
else
  echo "FAIL fails=$FAILS" > "$VERDICT"
  echo "== G-APERTURA-2 FAIL($FAILS) =="
  exit 1
fi
