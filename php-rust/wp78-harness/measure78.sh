#!/bin/bash
# measure78.sh — WP-78/79/80 measurement driver (design78.md protocol; S-78.1,
# hardened S-79.0 per Council WP-80, repaired S-80.0 per Council WP-81).
# Every figure this script emits cites: mode, binary hash, git rev, request
# counts, and the raw log path (KG-78.D: an untracked figure is void; KG-81-1:
# the raw must be COMMITTED with the campaign).
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
# S-80.0.2 repairs (Council WP-81):
# - IDENTITY GIT (A-BG18/KG-81-2): the matrix log's `git=` line MUST equal the
#   current HEAD — a valid hash from an older tree was attributed to the
#   current rev (misattribution was ACTIVE, 3a73be9 vs 36d62df). Mismatch ⇒
#   refuse. The source dirs must also be porcelain-clean (crates/, Cargo.*):
#   an edit after the matrix run invalidates the certification (KS-AH-81-1).
# - PER-RUN MATRIX ARCHIVE (A-SK11/KS-SK-81-2): the live feature-matrix.log is
#   overwritten each build — this driver copies it to $RUN.matrix so every
#   run's identity survives; a cited figure whose hash lives only in the
#   overwritten live log is NULL.
# - EXACT ROW MATCH (A-SK11): bin row matched as `^bin\[<row>\] ` — anchored
#   with the closing bracket AND the trailing space; a prefix match would
#   accept a future bin[census-x] row.
# - BOOT PROBE OUT OF CHANNEL (A-BG19/KG-81-1 recount): the old boot probe
#   curled the FIXTURE, producing one extra census line, so lines==N VOIDed
#   every legitimate run (historical census.r1.census: 111 = 1 boot + 10 + 100
#   under the old in-channel rule — recounted, consistent). Census modes now
#   boot-probe /__census_global (dispatcher/accept-side, NO census line);
#   EXPECTED stays WARMUP+MEASURED.
# - SPLIT UNDEFINED AT W>1 (A-BB17/KB-81-1): snap() is process-global while
#   the split accumulators are thread-local — census rows at --workers >1 are
#   undefined and the driver now REFUSES census modes with W>1.
# - IDLE WINDOW BOTH ARMS (A-DL8/A-BG20/A-AH20/KL-81-3): the idle bracket now
#   runs on censuscli too (the arm that makes A-BB1 judgeable had NO idle
#   probe). A campaign must include one LONG idle run per census arm:
#   IDLE_SECS=60 measure78.sh census idle60 … (default 10s misses slow drift).
# - TRIPWIRE FIELDS (KS-MS-81-2/KH81-1): census lines now carry a3_trip= and
#   (axum arm) inflight_max=. a3_trip>0 ⇒ ALL split figures of the run VOID;
#   inflight_max>1 ⇒ the closed-sequential claim is refuted (the queue-depth
#   watermark alone measures the QUEUE, not in-flight overlap — A-TH9). A
#   missing field on a hash-enforced build is itself a FAIL (defensive).
# - R≥3 applies to EVERY census fixture including the include ones
#   (A-BG13) — the driver is single-run; the operator invokes it 3× and
#   cites all three logs.
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

# A-BG24 (Council WP-82): the summary this driver emits must survive in a
# COMMITTED raw — the campaign .log files are the server's stderr, so the
# driver's own stdout (identity line, verdicts, idle summary) is archived
# per-run as $RUN.summary and committed with the raws (KG-81-1). Without
# this, every derived figure in the summary is recomputable-only (KG-82-3).
exec > >(tee "$RUN.summary")

# --- A-SK14 (KS-SK-82-1): idle-channel parser = cardinality pin + self-test --
# On a census arm the idle channel is EXACTLY 4 census-global lines: the
# boot probe (A-BG19, out of the census-line channel but IN this one) plus
# the 3 bracket probes. Any other count means a line source moved and every
# positional consumer is unanchored (the S-80.0.6 off-by-one class).
IDLE_EXPECTED=4

idle_cardinality_ok() { # <file> -> rc 0 iff exactly IDLE_EXPECTED lines
  [ "$(wc -l < "$1" | tr -d ' ')" -eq "$IDLE_EXPECTED" ]
}

idle_summary_from() { # <file> -> summary computed from the LAST THREE probes
  tail -3 "$1" | awk -F'[= ]' 'NR==1{c1=$3;b1=$5} NR==2{c2=$3;b2=$5} NR==3{c3=$3;b3=$5}
    END{ if (NR>=3) printf "idle: probe_self_cost calls=%d bytes=%d | idle_window(+self) calls=%d bytes=%d (A-AH13/A-DL5; churn-only, KL-81-3)\n",
         c2-c1, b2-b1, c3-c2, b3-b2;
         else print "idle: FEWER THAN 3 PROBES — idle window not measured" }'
}

# Self-test on a SYNTHETIC log BEFORE any real log is parsed (A-SK14/A-BG24):
# 4 known lines must yield the exact expected deltas; a 5th line must be
# rejected by the cardinality pin.
IDLE_ST=$(mktemp)
{
  echo "census-global: calls=100 bytes=1000 gross=1"
  echo "census-global: calls=150 bytes=1500 gross=1"
  echo "census-global: calls=160 bytes=1600 gross=1"
  echo "census-global: calls=190 bytes=1900 gross=1"
} > "$IDLE_ST"
WANT_ST="idle: probe_self_cost calls=10 bytes=100 | idle_window(+self) calls=30 bytes=300 (A-AH13/A-DL5; churn-only, KL-81-3)"
GOT_ST=$(idle_summary_from "$IDLE_ST")
if [ "$GOT_ST" != "$WANT_ST" ]; then
  echo "SELF-TEST BROKEN: idle parser wrong on synthetic 4-line log (A-SK14):"
  echo "  want: $WANT_ST"
  echo "  got:  $GOT_ST"
  rm -f "$IDLE_ST"; exit 2
fi
if ! idle_cardinality_ok "$IDLE_ST"; then
  echo "SELF-TEST BROKEN: 4-line synthetic rejected by the cardinality pin (A-SK14)"
  rm -f "$IDLE_ST"; exit 2
fi
echo "census-global: calls=200 bytes=2000 gross=1" >> "$IDLE_ST"
if idle_cardinality_ok "$IDLE_ST"; then
  echo "SELF-TEST BROKEN: 5-line synthetic PASSED the cardinality pin (A-SK14)"
  rm -f "$IDLE_ST"; exit 2
fi
rm -f "$IDLE_ST"
echo "self-test PASS: idle parser exact on synthetic 4-line log; 5-line rejected (A-SK14)"

# --- Identity quaterna+git ENFORCE (KS-AH-80-1/KS-SK-80-4, KG-81-2) ---------
MATRIX_LOG="$HERE/gate-axum/out/feature-matrix.log"
case "$MODE" in
  census|censuscli) ROW="census" ;;
  *)                ROW="union" ;;
esac
if [ ! -f "$MATRIX_LOG" ]; then
  echo "FAIL: $MATRIX_LOG missing — run gate-feature-matrix.sh on THIS build first (KS-AH-78-1)"
  exit 1
fi
# A-BG18/KG-81-2: the matrix certifies a rev — it must be THIS tree's rev.
MATRIX_GIT="$(awk -F= '/^git=/{print $2; exit}' "$MATRIX_LOG")"
if [ "$MATRIX_GIT" != "$GIT_REV" ]; then
  echo "FAIL: feature-matrix.log git=$MATRIX_GIT != current HEAD $GIT_REV (KG-81-2)"
  echo "      (re-run gate-feature-matrix.sh on this tree — a binary from an"
  echo "       older tree must not be attributed to the current rev)"
  exit 1
fi
# KS-AH-81-1 (measure-side): source dirs clean — an edit after the matrix run
# means the certified rev is no longer the source on disk. Campaign raws in
# the harness dirs are expected to be untracked mid-campaign and do not count.
# A-AH21 (Council WP-82): the PROTOCOL is this script + the gates — an edit
# to a harness .sh/.pl between matrix and measure produces figures under an
# identity that does not describe the executed protocol (the S-80.0.6 idle
# episode proved it: raws right, derived summary wrong — from the SCRIPT).
# The porcelain therefore covers the harness scripts too (never measure-out).
SRC_DIRTY="$(git -C "$REPO" status --porcelain -- crates Cargo.toml Cargo.lock \
  'wp7*-harness/*.sh' 'wp7*-harness/*.pl' 'wp8*-harness/*.sh' 'wp8*-harness/*.pl' 2>/dev/null)"
if [ -n "$SRC_DIRTY" ]; then
  echo "FAIL: source tree or harness scripts dirty since the matrix run (KS-AH-81-1/A-AH21) — refuse:"
  echo "$SRC_DIRTY" | head -10
  exit 1
fi
# A-AH21: driver_sha = fingerprint of the protocol scripts actually executed
# (this driver + the matrix gate that certified the identity). Emitted in the
# run header (archived in $RUN.summary) AND appended to the raw log at the
# end of the run — KS-AH-82-1: a summary from a script whose sha does not
# match the committed rev's is NULL and gets recomputed from the raws.
# A-AH30 (Council WP-83): the campaign WRAPPER orders the runs — it is
# protocol too. A campaign exports PHPR_CAMPAIGN_SCRIPT=<path to itself>;
# the sha covers it and the header names it. Unset = direct single run
# (legitimate), named as "none" so the reader can tell the two apart.
CAMPAIGN_SCRIPT="${PHPR_CAMPAIGN_SCRIPT:-}"
if [ -n "$CAMPAIGN_SCRIPT" ] && [ ! -f "$CAMPAIGN_SCRIPT" ]; then
  echo "FAIL: PHPR_CAMPAIGN_SCRIPT set but not a file: $CAMPAIGN_SCRIPT (A-AH30)"
  exit 1
fi
DRIVER_SHA="$(shasum -a 256 "$HERE/measure78.sh" "$HERE/gate-feature-matrix.sh" \
  ${CAMPAIGN_SCRIPT:+"$CAMPAIGN_SCRIPT"} | shasum -a 256 | cut -c1-16)"
# A-SK11: EXACT row match — `^bin\[<row>\] ` with bracket+space anchored.
EXPECTED_HASH="$(awk -F= -v row="$ROW" '$0 ~ ("^bin\\[" row "\\] ") {print $2}' "$MATRIX_LOG" | tail -1)"
if [ -z "$EXPECTED_HASH" ]; then
  echo "FAIL: no bin[$ROW] row in feature-matrix.log — matrix predates the quintet (A-AH10/A-AH16)"
  exit 1
fi
if [ "$HASH" != "$EXPECTED_HASH" ]; then
  echo "FAIL: binary on disk $HASH != bin[$ROW] $EXPECTED_HASH in feature-matrix.log"
  echo "      (KS-AH-80-1: rebuild the $ROW configuration, or re-run the matrix gate)"
  exit 1
fi
# A-SK11/KS-SK-81-2: per-run archive of the identity record.
cp "$MATRIX_LOG" "$RUN.matrix"
echo "identity: bin=$HASH == matrix bin[$ROW] (git=$GIT_REV == matrix git) — ENFORCED; archived: $RUN.matrix"

# W: pool size for axum/census (default 1 per protocollo; la probe di
# linearità passa W=num_cpus e scala nreq a >=100 per worker — UNION build
# only: KB-81-1 makes split rows at W>1 VOID by construction).
W="${W:-1}"
case "$MODE" in
  census|censuscli)
    if [ "$W" -gt 1 ]; then
      echo "FAIL: census modes at --workers $W refused (A-BB17/KB-81-1: snap() is"
      echo "      global, split accumulators thread-local — the rows are undefined)"
      exit 1
    fi ;;
esac
case "$MODE" in
  tier0)     ARGS=(--axum --tier0 --port "$PORT") ;;
  axum)      ARGS=(--axum --workers "$W" --port "$PORT" -t "$FIX") ;;
  census)    ARGS=(--axum --workers "$W" --port "$PORT" -t "$FIX") ;;
  cliserver) ARGS=(--port "$PORT" -t "$FIX") ;;
  censuscli) ARGS=(--port "$PORT" -t "$FIX") ;;
  *) echo "unknown mode $MODE"; exit 2 ;;
esac

echo "== measure78 mode=$MODE label=$LABEL fixture=$FIXTURE bin=$HASH git=$GIT_REV driver_sha=$DRIVER_SHA campaign=${CAMPAIGN_SCRIPT:-none} warmup=$WARMUP measured=$MEASURED idle_secs=$IDLE_SECS =="
find "$FIX" -name '._*' -delete

pkill -f "php-server" 2>/dev/null && sleep 1
MIMALLOC_PURGE_DELAY=0 MIMALLOC_SHOW_STATS=1 \
  /usr/bin/time -l "$BIN" "${ARGS[@]}" \
  > /dev/null 2> "$RUN.log" &
TIMEPID=$!
# The server is a CHILD of time; find the php-server pid for vmmap.
# A-BG19 (S-80.0.2): census modes boot-probe /__census_global — an endpoint
# that produces NO census line — so the boot probe stays OUT of the counted
# channel and EXPECTED lines == WARMUP+MEASURED exactly. Non-census modes
# keep the fixture probe (their lines are not counted).
case "$MODE" in
  census|censuscli) BOOTURL="http://127.0.0.1:$PORT/__census_global" ;;
  *)                BOOTURL="http://127.0.0.1:$PORT/$FIXTURE" ;;
esac
up=0
for _ in $(seq 1 100); do
  if curl -s -o /dev/null -m 1 "$BOOTURL"; then up=1; break; fi
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
# A-BG32 (Council WP-84): per-request ns drain 1 — DISCARDS the warmup
# samples from the buffer (the drained line lands in $RUN.log, labeled;
# the verdict reads the LAST reqns line = the measured window only).
# Armed only when the campaign exports PHPR_REQ_NS=1 (server inherits);
# the __reqns probe answers from the ROUTER, no worker sample is added.
# Sits OUTSIDE both timed loops (WP-64).
if [ "${PHPR_REQ_NS:-}" = "1" ]; then
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__reqns"
fi
vmmap "$SRVPID" 2>/dev/null > "$RUN.vmmap.V1"

# S-79.0.3 (A-AH13/A-DL5), extended S-80.0.2 (A-DL8/A-BG20/A-AH20): idle
# window on BOTH census arms — probes bracket IDLE_SECS with zero requests.
# probe1→probe2 back-to-back declares the probe SELF-cost; probe2→probe3
# across the sleep is self-cost + true idle drift. Both reported; the a-phase
# diffs may absorb at most the drift figure as cross-thread noise. The window
# is CHURN-ONLY (counter drift): it says nothing about resident idle growth
# (KL-81-3 — resident claims need the union twin + vmmap + a declared floor).
if [ "$MODE" = census ] || [ "$MODE" = censuscli ]; then
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__census_global"
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__census_global"
  sleep "$IDLE_SECS"
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__census_global"
fi

for _ in $(seq 1 "$MEASURED"); do
  curl -s -m 10 -o /dev/null "http://127.0.0.1:$PORT/$FIXTURE"
done
# A-BG32 drain 2: the MEASURED-window samples — the verdict's reqns line.
if [ "${PHPR_REQ_NS:-}" = "1" ]; then
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__reqns"
fi
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
  tr -d '\0' < "$RUN.log" | grep "^census-global:" > "$RUN.idle" || true
fi

# S-80.0.2 tripwire/field checks shared by both census arms. A field the
# hash-enforced build is supposed to emit but does not is a FAIL in itself:
# "absent" must never read as "zero" (KS-MS-81-2 / KH81-1).
check_census_fields() { # $1 = field regex (e.g. a3_trip), $2 = max allowed
  local field="$1" maxv="$2" bad missing
  missing=$(awk -v f="$field" '$0 !~ (" " f "=") {print; exit}' "$RUN.census")
  if [ -n "$missing" ]; then
    echo "FAIL: census line without $field= field on an enforced build — run VOID (S-80.0.2 defensive)"
    RC=1; return
  fi
  bad=$(awk -v f="$field" -v m="$maxv" '{for(i=1;i<=NF;i++){split($i,a,"="); if (a[1]==f && a[2]+0>m) {print; exit}}}' "$RUN.census" | head -1)
  if [ -n "$bad" ]; then
    echo "FAIL: $field > $maxv in a census line — run VOID ($field tripwire)"
    RC=1
  else
    echo "OK: $field <= $maxv on all $CENSUS_LINES lines"
  fi
}

report_idle() {
  # S-80.0.6 fix (boot probe is census-global too, A-BG19: bracket probes =
  # LAST THREE lines) + A-SK14 pin: the count itself is now ENFORCED at
  # exactly $IDLE_EXPECTED — a positional parser without a cardinality pin
  # regresses in silence exactly like the original off-by-one (KS-SK-82-1:
  # unpinned positional summary = ADVISORY, never verdict-grade).
  if [ ! -s "$RUN.idle" ]; then
    echo "FAIL: NO census-global probes in log — idle window not measured (A-AH13) — run VOID"
    RC=1; return
  fi
  local nl
  nl=$(wc -l < "$RUN.idle" | tr -d ' ')
  if [ "$nl" -ne "$IDLE_EXPECTED" ]; then
    echo "FAIL: $nl census-global lines != pinned $IDLE_EXPECTED (boot + 3 probes) — idle summary VOID (A-SK14/KS-SK-82-1)"
    RC=1; return
  fi
  idle_summary_from "$RUN.idle"
}

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
    echo "NOTE: all census byte figures are GROSS churn, upper bound (A-DL1/A-DL10)"
    echo "exit_stats=UNAVAILABLE-BY-DESIGN (KG-79.B) — counters do not need them (A-BG16)"
    if [ "$CENSUS_LINES" -ne "$EXPECTED_LINES" ]; then
      echo "FAIL: census-cli lines $CENSUS_LINES != expected $EXPECTED_LINES (A-PP5; boot probe is out-of-channel since S-80.0.2) — run VOID"
      RC=1
    else
      echo "census_cli_lines=$CENSUS_LINES == expected (A-PP5 OK)"
    fi
    check_census_fields a3_trip 0
    report_idle ;;
  census)
    echo "NOTE: footprint figures from this run are NULL (KB-78-5) — counters only in $RUN.census"
    echo "NOTE: all census byte figures are GROSS churn, upper bound (A-DL1/A-DL10)"
    if [ "$CENSUS_LINES" -ne "$EXPECTED_LINES" ]; then
      echo "FAIL: census lines $CENSUS_LINES != expected $EXPECTED_LINES (A-PP5: an early-fatal request skips its line; boot probe is out-of-channel since S-80.0.2) — run VOID"
      RC=1
    else
      echo "census_lines=$CENSUS_LINES == expected (A-PP5 OK)"
    fi
    if [ -n "$DEPTH_BAD" ]; then
      echo "FAIL: KH78-2 VIOLATION: depth>1 sample — run VOID (enforced, A-SK7)"
      RC=1
    fi
    # KH81-1: the queue watermark measures the QUEUE (dec at pickup) — the
    # in-flight watermark is the observable that catches pipelined overlap.
    check_census_fields inflight_max 1
    check_census_fields a3_trip 0
    report_idle ;;
  *)
    if [ "$JOIN" -ge 1 ]; then
      echo "exit_stats=VERDICT-GRADE (join line present, KL-78-4)"
    else
      echo "exit_stats=ADVISORY (no join line!)"
    fi ;;
esac
# A-AH21: the raw log carries the protocol fingerprint too (grep-anchored
# lines are untouched — this is an appended driver line, not a census line).
echo "driver_sha=$DRIVER_SHA git=$GIT_REV bin=$HASH" >> "$RUN.log"
echo "raw: $RUN.log | vmmap: $RUN.vmmap.V1/V2 | matrix: $RUN.matrix | summary: $RUN.summary"
echo "== measure78 done mode=$MODE label=$LABEL bin=$HASH git=$GIT_REV driver_sha=$DRIVER_SHA rc=$RC =="
exit "$RC"
