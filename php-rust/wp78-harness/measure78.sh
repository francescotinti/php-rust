#!/bin/bash
# measure78.sh — WP-78/79 measurement driver (design78.md protocol; S-78.1,
# hardened S-79.0 per Council WP-80).
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
#   censuscli — instrumented build, --cli-server arm: per-request census-cli
#               lines (S-79.0.6/A-BG16 — the counters that make A-BB1
#               judgeable; footprint figures NULL here too)
#
# Protocol constants (design78): WARMUP=10, MEASURED=100, closed-sequential
# (single curl loop — mechanism per A-BG9), MIMALLOC_PURGE_DELAY=0,
# peak = /usr/bin/time -l over the WHOLE process (KG-79.C), vmmap V1 after
# warm-up / V2 after batch (A-DL8), SIGTERM clean exit (join line required
# for verdict-grade exit stats, KS-MS-5/KL-78-4).
#
# S-79.0 hardening (Council WP-80):
# - ENFORCE identity (A-SK7/A-BG15/A-AH12, KS-AH-80-1/KS-SK-80-4): the
#   measured binary's hash MUST equal the feature-matrix.log row for this
#   mode (census → bin[census], others → bin[union]); mismatch = refuse to
#   run. The c21c2959 incident becomes impossible, not just detectable.
# - census: expected census lines == WARMUP+MEASURED (A-PP5 — an
#   early-fatal request silently skips its line), depth>1 ⇒ exit 1
#   (KH78-2 enforcement, was print-and-exit-0), idle window drift probes
#   (A-AH13/A-DL5) via /__census_global bracketing an idle sleep.
# - R≥3 applies to EVERY census fixture including the include ones
#   (A-BG13: census.inc was R=1 undeclared) — the driver is single-run;
#   the operator invokes it 3× and cites all three logs.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
FIX="$HERE/gate-axum/fixtures"
OUT="$HERE/measure-out"
mkdir -p "$OUT"
MODE="${1:?usage: measure78.sh <tier0|axum|cliserver|census|censuscli> <label> [fixture] [nreq]}"
LABEL="${2:?label required}"
FIXTURE="${3:-hello.php}"
WARMUP=10
MEASURED="${4:-100}"
PORT="${PORT:-8194}"
IDLE_SECS="${IDLE_SECS:-10}"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
RUN="$OUT/$MODE.$LABEL"
BIN="$HOME/Claude/php-rust-output/release/php-server"
HASH=$(shasum -a 256 "$BIN" | cut -c1-16)
RC=0

# --- Identity quaterna ENFORCE (KS-AH-80-1/KS-SK-80-4) ----------------------
MATRIX_LOG="$HERE/gate-axum/out/feature-matrix.log"
case "$MODE" in
  census|censuscli) ROW="census" ;;
  *)                ROW="union" ;;
esac
if [ ! -f "$MATRIX_LOG" ]; then
  echo "FAIL: $MATRIX_LOG missing — run gate-feature-matrix.sh on THIS build first (KS-AH-78-1)"
  exit 1
fi
EXPECTED_HASH="$(awk -F= -v row="bin[$ROW]" '$0 ~ "^bin\\[" { if (index($0, row) == 1) print $2 }' "$MATRIX_LOG" | tail -1)"
if [ -z "$EXPECTED_HASH" ]; then
  echo "FAIL: no bin[$ROW] row in feature-matrix.log — matrix predates the quaterna (A-AH10)"
  exit 1
fi
if [ "$HASH" != "$EXPECTED_HASH" ]; then
  echo "FAIL: binary on disk $HASH != bin[$ROW] $EXPECTED_HASH in feature-matrix.log"
  echo "      (KS-AH-80-1: rebuild the $ROW configuration, or re-run the matrix gate)"
  exit 1
fi
echo "identity: bin=$HASH == matrix bin[$ROW] (git=$GIT_REV) — ENFORCED"

# W: pool size for axum/census (default 1 per protocollo; la probe di
# linearità passa W=num_cpus e scala nreq a >=100 per worker).
W="${W:-1}"
case "$MODE" in
  tier0)     ARGS=(--axum --tier0 --port "$PORT") ;;
  axum)      ARGS=(--axum --workers "$W" --port "$PORT" -t "$FIX") ;;
  census)    ARGS=(--axum --workers "$W" --port "$PORT" -t "$FIX") ;;
  cliserver) ARGS=(--port "$PORT" -t "$FIX") ;;
  censuscli) ARGS=(--port "$PORT" -t "$FIX") ;;
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

# S-79.0.3 (A-AH13/A-DL5): idle window — dispatcher-side probes (no pool, no
# Vm, no census line) bracket IDLE_SECS with zero requests. probe1→probe2
# back-to-back declares the probe SELF-cost; probe2→probe3 across the sleep
# is self-cost + true idle drift. Both reported; the a-phase diffs may absorb
# at most the drift figure as cross-thread noise.
if [ "$MODE" = census ]; then
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__census_global"
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__census_global"
  sleep "$IDLE_SECS"
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__census_global"
fi

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
if [ "$MODE" = census ]; then
  tr -d '\0' < "$RUN.log" | grep "^census:" > "$RUN.census"
  DEPTH_BAD=$(awk '{for(i=1;i<=NF;i++) if ($i ~ /^depth_max=/) {split($i,a,"="); if (a[2]+0>1) print}}' "$RUN.census" | head -1)
  CENSUS_LINES=$(wc -l < "$RUN.census" | tr -d ' ')
  EXPECTED_LINES=$((WARMUP + MEASURED))
  tr -d '\0' < "$RUN.log" | grep "^census-global:" > "$RUN.idle" || true
fi
if [ "$MODE" = censuscli ]; then
  tr -d '\0' < "$RUN.log" | grep "^census-cli:" > "$RUN.census"
  CENSUS_LINES=$(wc -l < "$RUN.census" | tr -d ' ')
  EXPECTED_LINES=$((WARMUP + MEASURED))
fi

echo "peak_rss_bytes=$PEAK (whole process, time -l)"
echo "vmmap_V1=$V1 vmmap_V2=$V2 (Physical footprint)"
case "$MODE" in
  tier0)
    # A-BG15: explicit case — no pool exists, so no join line can: N/A by
    # construction, never "ADVISORY" (that label means a MISSING join).
    echo "exit_stats=VERDICT-GRADE-N/A-JOIN (tier0: no pool by construction, graceful shutdown)" ;;
  cliserver)
    echo "exit_stats=UNAVAILABLE-BY-DESIGN (KG-79.B: cli-server arm has no join) — vmmap only" ;;
  censuscli)
    echo "NOTE: footprint figures from this run are NULL (KB-78-5) — census-cli counters only in $RUN.census"
    echo "exit_stats=UNAVAILABLE-BY-DESIGN (KG-79.B) — counters do not need them (A-BG16)"
    if [ "$CENSUS_LINES" -ne "$EXPECTED_LINES" ]; then
      echo "FAIL: census-cli lines $CENSUS_LINES != expected $EXPECTED_LINES (A-PP5) — run VOID"
      RC=1
    else
      echo "census_cli_lines=$CENSUS_LINES == expected (A-PP5 OK)"
    fi ;;
  census)
    echo "NOTE: footprint figures from this run are NULL (KB-78-5) — counters only in $RUN.census"
    if [ "$CENSUS_LINES" -ne "$EXPECTED_LINES" ]; then
      echo "FAIL: census lines $CENSUS_LINES != expected $EXPECTED_LINES (A-PP5: an early-fatal request skips its line) — run VOID"
      RC=1
    else
      echo "census_lines=$CENSUS_LINES == expected (A-PP5 OK)"
    fi
    if [ -n "$DEPTH_BAD" ]; then
      echo "FAIL: KH78-2 VIOLATION: depth>1 sample — run VOID (enforced, A-SK7)"
      RC=1
    fi
    if [ -s "$RUN.idle" ]; then
      awk -F'[= ]' 'NR==1{c1=$3;b1=$5} NR==2{c2=$3;b2=$5} NR==3{c3=$3;b3=$5}
        END{ if (NR>=3) printf "idle: probe_self_cost calls=%d bytes=%d | idle_window(+self) calls=%d bytes=%d (A-AH13/A-DL5)\n",
             c2-c1, b2-b1, c3-c2, b3-b2;
             else print "idle: FEWER THAN 3 PROBES — idle window not measured" }' "$RUN.idle"
    else
      echo "idle: NO census-global probes in log — idle window not measured (A-AH13)"
    fi ;;
  *)
    if [ "$JOIN" -ge 1 ]; then
      echo "exit_stats=VERDICT-GRADE (join line present, KL-78-4)"
    else
      echo "exit_stats=ADVISORY (no join line!)"
    fi ;;
esac
echo "raw: $RUN.log | vmmap: $RUN.vmmap.V1/V2"
echo "== measure78 done mode=$MODE label=$LABEL bin=$HASH git=$GIT_REV rc=$RC =="
exit "$RC"
