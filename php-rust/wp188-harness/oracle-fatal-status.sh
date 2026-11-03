#!/bin/bash
# oracle-fatal-status.sh — S-188 Tappa 2 C0: what the ORACLE `php -S` (SAPI cli-server, the
# same SAPI the axum handler will emulate) answers for the four fatal fixtures of
# wp78-harness/gate-axum (fatal_runtime, fatal_compile, exit_code, parse_err): HTTP status +
# body. Pre-registered expectation (criterio s188-criterio-axum-wp.md p.3): 200 ×4, html
# banners (display_errors=1, html_errors=1 under the web SAPI). Output: oracle-fatal-status.out
# (status per fixture + body bytes saved beside it). Generated with AI assistance (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
H="$(cd "$(dirname "$0")" && pwd)"
FIX="$H/../wp78-harness/gate-axum/fixtures"
ORACLE=${ORACLE:-/opt/homebrew/opt/php/bin/php}
PORT=${PORT:-8199}
OUT="$H/oracle-fatal-status.out"
BODIES="$H/pair-out/oracle-fatal"; mkdir -p "$BODIES"
pkill -f "127.0.0.1:$PORT" 2>/dev/null; sleep 0.5
( cd "$FIX" && "$ORACLE" -S "127.0.0.1:$PORT" -t "$FIX" > /dev/null 2> "$BODIES/server.log" ) &
SRV=$!
for _ in $(seq 1 50); do curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php" && break; sleep 0.1; done
{
  echo "== oracle-fatal-status: $("$ORACLE" -v | head -1) · php -S (cli-server) · fixtures wp78-harness/gate-axum · $(date '+%F %T') =="
  for fx in fatal_runtime fatal_compile exit_code parse_err; do
    st=$(curl -s -m 5 -o "$BODIES/$fx.body" -w '%{http_code}' -X POST "http://127.0.0.1:$PORT/$fx.php")
    printf '%-14s http %s · %s B · head: %s\n' "$fx" "$st" "$(wc -c < "$BODIES/$fx.body" | tr -d ' ')" "$(head -c 60 "$BODIES/$fx.body" | tr '\n' '~')"
  done
  echo "-- cli (phpr -S expectation for the SAME fixtures is this SAPI's, not the CLI's) --"
  for fx in fatal_runtime fatal_compile exit_code parse_err; do
    "$ORACLE" "$FIX/$fx.php" > "$BODIES/$fx.cli" 2>/dev/null; printf '%-14s cli rc=%s · %s B\n' "$fx" "$?" "$(wc -c < "$BODIES/$fx.cli" | tr -d ' ')"
  done
  echo "-- server.log (stderr dell'oracle) --"; grep -v -E 'Accepted|Closing' "$BODIES/server.log" | head -8
} > "$OUT" 2>&1
kill "$SRV" 2>/dev/null; pkill -f "127.0.0.1:$PORT" 2>/dev/null; wait "$SRV" 2>/dev/null
cat "$OUT"
