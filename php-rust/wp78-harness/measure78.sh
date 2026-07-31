#!/bin/bash
# measure78.sh — WP-78 measurement driver (design78.md protocol; S-78.1).
# Every figure this script emits cites: mode, binary hash, git rev, request
# counts, and the raw log path (KG-78.D: an untracked figure is void).
#
# Modes:
#   tier0     — Axum floor (--tier0): no pool, no Vm            [union twin]
#   axum      — worker pool --workers 1, fixture workload       [union twin]
#   cliserver — same binary, --cli-server mode (KG-79.B: NO exit stats on
#               this arm — vmmap V1/V2 only)                    [union twin]
#   census    — instrumented build --workers 1: per-phase counters ONLY
#               (KB-78-5: any footprint figure from this build is NULL)
#
# Protocol constants (design78): WARMUP=10, MEASURED=100, closed-sequential
# (single curl loop — mechanism per A-BG9), MIMALLOC_PURGE_DELAY=0,
# peak = /usr/bin/time -l over the WHOLE process (KG-79.C), vmmap V1 after
# warm-up / V2 after batch (A-DL8), SIGTERM clean exit (join line required
# for verdict-grade exit stats, KS-MS-5/KL-78-4).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
FIX="$HERE/gate-axum/fixtures"
OUT="$HERE/measure-out"
mkdir -p "$OUT"
MODE="${1:?usage: measure78.sh <tier0|axum|cliserver|census> <label> [fixture] [nreq]}"
LABEL="${2:?label required}"
FIXTURE="${3:-hello.php}"
WARMUP=10
MEASURED="${4:-100}"
PORT="${PORT:-8194}"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
RUN="$OUT/$MODE.$LABEL"
BIN="$HOME/Claude/php-rust-output/release/php-server"
HASH=$(shasum -a 256 "$BIN" | cut -c1-16)

# W: pool size for axum/census (default 1 per protocollo; la probe di
# linearità passa W=num_cpus e scala nreq a >=100 per worker).
W="${W:-1}"
case "$MODE" in
  tier0)     ARGS=(--axum --tier0 --port "$PORT") ;;
  axum)      ARGS=(--axum --workers "$W" --port "$PORT" -t "$FIX") ;;
  census)    ARGS=(--axum --workers "$W" --port "$PORT" -t "$FIX") ;;
  cliserver) ARGS=(--port "$PORT" -t "$FIX") ;;
  *) echo "unknown mode $MODE"; exit 2 ;;
esac

echo "== measure78 mode=$MODE label=$LABEL fixture=$FIXTURE bin=$HASH git=$GIT_REV warmup=$WARMUP measured=$MEASURED =="
find "$FIX" -name '._*' -delete

pkill -f "php-server" 2>/dev/null && sleep 1
MIMALLOC_PURGE_DELAY=0 MIMALLOC_SHOW_STATS=1 \
  /usr/bin/time -l "$BIN" "${ARGS[@]}" \
  > /dev/null 2> "$RUN.log" &
TIMEPID=$!
# The server is a CHILD of time; find the php-server pid for vmmap.
up=0
for _ in $(seq 1 100); do
  if curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/$FIXTURE"; then up=1; break; fi
  sleep 0.1
done
if [ "$up" != 1 ]; then
  echo "FAIL: server did not come up (see $RUN.log)"; kill "$TIMEPID" 2>/dev/null; exit 1
fi
SRVPID="$(lsof -nP -tiTCP:"$PORT" -sTCP:LISTEN | head -1)"
[ -z "$SRVPID" ] && { echo "FAIL: no pid on port"; exit 1; }

# Closed-sequential loop (A-BG9 mechanism: request N+1 only after N read).
for _ in $(seq 1 "$WARMUP"); do
  curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/$FIXTURE"
done
vmmap "$SRVPID" 2>/dev/null > "$RUN.vmmap.V1"
for _ in $(seq 1 "$MEASURED"); do
  curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/$FIXTURE"
done
vmmap "$SRVPID" 2>/dev/null > "$RUN.vmmap.V2"

kill -TERM "$SRVPID" 2>/dev/null
wait "$TIMEPID" 2>/dev/null

V1=$(awk '/^Physical footprint:/ {print $3}' "$RUN.vmmap.V1")
V2=$(awk '/^Physical footprint:/ {print $3}' "$RUN.vmmap.V2")
PEAK=$(awk '/maximum resident set size/ {print $1}' "$RUN.log")
JOIN=$(grep -c "workers joined" "$RUN.log")
DEPTH_BAD=""
if [ "$MODE" = census ]; then
  grep "^census:" "$RUN.log" > "$RUN.census"
  DEPTH_BAD=$(awk '{for(i=1;i<=NF;i++) if ($i ~ /^depth_max=/) {split($i,a,"="); if (a[2]+0>1) print}}' "$RUN.census" | head -1)
fi

echo "peak_rss_bytes=$PEAK (whole process, time -l)"
echo "vmmap_V1=$V1 vmmap_V2=$V2 (Physical footprint)"
case "$MODE" in
  cliserver)
    echo "exit_stats=UNAVAILABLE-BY-DESIGN (KG-79.B: cli-server arm has no join) — vmmap only" ;;
  census)
    echo "NOTE: footprint figures from this run are NULL (KB-78-5) — counters only in $RUN.census"
    [ -n "$DEPTH_BAD" ] && echo "KH78-2 VIOLATION: depth>1 sample — run VOID" ;;
  *)
    if [ "$JOIN" -ge 1 ]; then
      echo "exit_stats=VERDICT-GRADE (join line present, KL-78-4)"
    else
      echo "exit_stats=ADVISORY (no join line!)"
    fi ;;
esac
echo "raw: $RUN.log | vmmap: $RUN.vmmap.V1/V2"
echo "== measure78 done mode=$MODE label=$LABEL bin=$HASH git=$GIT_REV =="
