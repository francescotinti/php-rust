#!/bin/bash
# measure87-campaign.sh — S-87.0 p5: ordine Concilio WP-88 §Sintesi p5.
#   SLOPE (A-DL38≡A-BB52): metà fisica = slope di COMMITTED steady-state.
#     W∈{1,2,3,4}, N=100·W richieste hello SEQUENZIALI per run, R=5 per W,
#     quiesce + mi_collect(true) all'exit (PHPR_MI_COLLECT_EXIT=1 — snapshot
#     win=0 POST-collect = ultima riga mi_proc win=0), MIMALLOC_PURGE_DELAY=0
#     (protocollo storico). Giudice a valle: min-of-R per W, slope LSQ,
#     banda KL-85-2 3.605.572 B ±5% (esito WITHIN o NAMED-DEVIATION — la
#     fisica non è un gate; il gate è il protocollo). MAI max−min (KB-88-3).
#     DECLARED DEVIATION (per NOME): mi_collect gira sull'heap CONDIVISO v3
#     all'atexit, non per-theap (l'export per-theap è materia A-DL39).
#   Bracci di sottrazione candidati (per NOME): EAGER (W=2, R=3,
#     MIMALLOC_ARENA_EAGER_COMMIT=1) · MINSTACK (W=2, R=3,
#     RUST_MIN_STACK=1048576) — etichette in-band nella identity line.
#   DISJOINT (A-BB53): coppia pad87a/pad87b DISGIUNTA (prefissi unici,
#     nest-free). CAL: W=1, un fixture per run, R=2 per lato. CONC: W=2,
#     entrambe in parallelo, 2 attempt. PREDIZIONE EX-ANTE: sotto
#     net_window=process-counters gli span si intersecano e net(ord1 di
#     ciascun thread) ≈ cal_own + cal_other (firma-inghiottimento su coppia
#     NEST-FREE); floor_inc per lato ≈ floor_inc della propria cal (nessun
#     Δfloor: nessun nodo condiviso oltre il prelude per-thread). I net
#     concorrenti restano VOID come cifre per-thread (KB-88-1) — il run
#     esiste per SCOPARE la promozione, non per attribuire.
# Catena evidenza (Concilio WP-88 p3):
#   A-SK46/KS-SK-88-1: la battery è consumata SOLO via
#     battery-equivalence.sh --same-rev (denti v4: anchored PASS, sha256,
#     stamp committed 4 campi, matrix committed, toolchain A-AH44).
#   A-SK49/A-BG43/KG-88-1: filename con attempt= (mai riusati), ledger di
#     campagna APPEND-only, pre-flight fail-closed su raw m87.* same-label
#     non committati.
#   A-BG44: PHPR_CAMPAIGN_SEQ esportato per run (seq= + epoch in-band).
#   KG-88-3: m87.done scritto SOLO a FAILS==0 e CONSUMATO da verdict87.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$REPO/wp78-harness/measure-out"
OUTBIN="$HOME/Claude/php-rust-output/release"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
ORACLE="/opt/homebrew/opt/php/bin/php"
BATW="/Volumes/Extreme Pro/Claude/wp87-battery-out"
PORT=8296
CAMPAIGN_SHA=$(shasum -a 256 "$0" | cut -c1-16)
export PHPR_CAMPAIGN_SCRIPT="$0"
SEQ=0

fail() { echo "FAIL: $*"; ledger "abort: $*"; exit 1; }
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
HEAD_AT_START="$GIT_REV"
head_unmoved() {
  local now; now="$(git -C "$REPO" rev-parse --short HEAD)"
  [ "$now" = "$HEAD_AT_START" ] || { echo "FAIL: HEAD moved mid-campaign ($now != $HEAD_AT_START)"; exit 1; }
}

# --- A-BG43: APPEND-only campaign ledger (never truncated, ever) ------------
LEDGER="$OUT/m87.campaign.ledger"
ledger() { echo "attempt_epoch=$(date +%s) git=$GIT_REV campaign_sha=$CAMPAIGN_SHA $*" >> "$LEDGER"; }

# --- Pre-flight -------------------------------------------------------------
echo "== measure87 pre-flight git=$GIT_REV campaign_sha=$CAMPAIGN_SHA =="
DIRTY="$(git -C "$REPO" status --porcelain 2>/dev/null)"
[ -z "$DIRTY" ] || { echo "$DIRTY" | head -5; fail "tree not porcelain at campaign start"; }
pgrep -f "php-server" >/dev/null && fail "php-server already running"
# A-SK49/KS-SK-88-4: any m87.* raw in the tree that is NOT committed is a
# leftover of an aborted attempt under a reusable name — fail-closed.
UNCOMMITTED=$(git -C "$REPO" status --porcelain -- "wp78-harness/measure-out" 2>/dev/null | grep "m87\." || true)
[ -z "$UNCOMMITTED" ] || { echo "$UNCOMMITTED" | head -5; fail "uncommitted m87.* raws in measure-out (A-SK49/KG-88-1)"; }

# attempt suffix: first free aN for this campaign generation (names NEVER
# reused across attempts — KG-88-1).
ATT=1
while ls "$OUT"/m87.slope.w1.r1.a$ATT.* >/dev/null 2>&1; do ATT=$((ATT+1)); done
echo "attempt=$ATT (filenames carry it; prior generations stay committed)"
ledger "attempt=$ATT phase=start"

# --- A-SK46: battery consumption via the v4-toothed checker only ------------
BOUT="$BATW/battery-87pre.out"
[ -f "$BOUT" ] || fail "battery-87pre.out missing — run the battery first"
BREV=$(sed -n 's/^== battery-87pre git=\([0-9a-f]*\) ==$/\1/p' "$BOUT" | head -1)
# Same-CODE protocol: the checker's teeth (i)/(iv) judge that BREV..HEAD
# moved no crates/Cargo and no gate object (the A-SK41 stamp commit is the
# one legal delta); everything else (anchored PASS, sha256, 4-field
# committed stamp, committed matrix, toolchain) runs in full.
bash "$REPO/wp83-harness/battery-equivalence.sh" --same-rev "$BOUT" "$BREV" \
  || fail "same-rev consumption REFUSED (A-SK46/KS-SK-88-1)"
DONE="$BATW/.done"
MTX_NAME=$(sed -n 's/.* matrix=\([^ ]*\) .*/\1/p' "$DONE" | head -1)
MTX="$REPO/wp78-harness/matrix-archive/$MTX_NAME"
mtx_hash() { tr -d '\0' < "$MTX" | sed -n "s/^bin\[$1\] sha256\[0:16\]=\([0-9a-f]*\).*/\1/p"; }

# --- Build the mem-census arm, enforce identity -----------------------------
( cd "$REPO" && cargo build --release -p php-server --features mem-census ) \
  > "$OUT/m87.build.mem-census.a$ATT.log" 2>&1 || fail "build mem-census"
head_unmoved
MEM_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
WANT=$(mtx_hash mem-census)
[ -n "$WANT" ] || fail "matrix has no bin[mem-census] row"
[ "$MEM_HASH" = "$WANT" ] || fail "mem-census hash $MEM_HASH != matrix $WANT (KS-AH-87-1)"
bash "$REPO/wp85-harness/gate-binary-noprobe.sh" "$OUTBIN/php-server" "$WANT" \
  || fail "gate-binary-noprobe on mem-census arm (KH86-1/A-TH37)"
ledger "attempt=$ATT phase=identity mem_hash=$MEM_HASH matrix=$MTX_NAME"

WANTH=$(cd "$FIXDIR" && "$ORACLE" -n hello.php 2>/dev/null)
WANTA=$(cd "$FIXDIR" && "$ORACLE" -n pad87a.php 2>/dev/null)
WANTB=$(cd "$FIXDIR" && "$ORACLE" -n pad87b.php 2>/dev/null)

# MEASURED ex-ante (in-band): the verdict refuses raws whose request count
# does not match this table.
echo "MEASURED ex-ante: slope N=100*W R=5 W=1..4 | eager W=2 R=3 | minstack W=2 R=3 | cal 1 req R=2/side | conc 2 req x2 attempts"

wait_up() {
  local up=0
  for _ in $(seq 1 100); do
    curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/__reqns" && { up=1; break; }
    sleep 0.1
  done
  [ $up = 1 ]
}

# ANTI-ORPHAN teeth (attempt=1 lesson, S-87.0): /usr/bin/time was the $! —
# TERMing it left the FIRST server alive on the port for the whole
# campaign; every later server died at bind and wait_up was answered by
# the orphan (bodies identical, rows landing in run 1's raw). The
# fail-closed verdict caught it (VDISP/VORD zero rows). From attempt=2:
#   (a) the process we own must be ALIVE after wait_up;
#   (b) exactly ONE php-server may exist, and it must be OUR descendant;
#   (c) after teardown NO php-server may remain.
assert_single_server() { # <label> <owner-pid> -> echoes the server pid
  # NB: runs inside $() — diagnostics go to stderr, stdout is the pid only;
  # the caller's `|| exit 1` propagates the subshell's failure.
  local label="$1" owner="$2" pids srv
  pids=$(pgrep -f "php-server --axum" || true)
  [ -n "$pids" ] || { fail "m87.$label no php-server process after up (anti-orphan a)" 1>&2; }
  [ "$(echo "$pids" | grep -c .)" = 1 ] || { echo "$pids" 1>&2; fail "m87.$label MULTIPLE php-server processes (anti-orphan b)" 1>&2; }
  srv="$pids"
  # our descendant: either the pid we spawned or a direct child of it
  if [ "$srv" != "$owner" ] && [ "$(ps -o ppid= -p "$srv" | tr -d ' ')" != "$owner" ]; then
    fail "m87.$label the live php-server ($srv) is not ours ($owner) — orphan serving (anti-orphan b)" 1>&2
  fi
  echo "$srv"
}
assert_server_gone() { # <label>
  local i
  for i in 1 2 3 4 5 6 7 8 9 10; do
    pgrep -f "php-server --axum" >/dev/null || return 0
    sleep 0.5
  done
  fail "m87.$1 php-server still alive after teardown (anti-orphan c)"
}

# run_arm <label> <workers> <nreq> <extra-env...>
run_arm() {
  local label="$1" workers="$2" nreq="$3"; shift 3
  SEQ=$((SEQ+1))
  local MC="$OUT/m87.$label.a$ATT.memcensus"
  local LOG="$OUT/m87.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  env "$@" PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 \
      PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers "$workers" --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || { kill -TERM $DPID 2>/dev/null; fail "m87.$label server not up"; }
  # under /usr/bin/time the server is DPID's CHILD: TERM must target IT.
  local SRV
  SRV=$(assert_single_server "$label" "$DPID") || exit 1
  local ok=0 i=0 B
  B=$(mktemp)
  while [ $i -lt "$nreq" ]; do
    curl -s -m 10 -o "$B" "http://127.0.0.1:$PORT/hello.php"
    [ "$(cat "$B")" = "$WANTH" ] && ok=$((ok+1))
    i=$((i+1))
  done
  rm -f "$B"
  [ "$ok" = "$nreq" ] || { kill -TERM "$SRV" 2>/dev/null; fail "m87.$label bodies $ok/$nreq != oracle"; }
  sleep 2   # quiesce before teardown (drain + purge_delay=0 already active)
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m87.$label panic in server log (KH88-3)"
  echo "mem_hash=$MEM_HASH git=$GIT_REV campaign_sha=$CAMPAIGN_SHA arm=mem-census phase=$label attempt=$ATT w=$workers nreq=$nreq envtag=${ENVTAG:-none} server_exit=$rc seq=$SEQ epoch=$(date +%s)" >> "$MC"
  ledger "attempt=$ATT phase=$label raw=m87.$label.a$ATT.memcensus esito=ok nreq=$nreq"
}

# run_pad <label> <fixture> — W=1 single-request calibration run (A-BB53 CAL)
run_pad() {
  local label="$1" fixture="$2" want="$3"
  SEQ=$((SEQ+1))
  local MC="$OUT/m87.$label.a$ATT.memcensus"
  local LOG="$OUT/m87.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 PHPR_CAMPAIGN_SEQ=$SEQ \
    "$OUTBIN/php-server" --axum --workers 1 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || { kill -TERM $DPID 2>/dev/null; fail "m87.$label server not up"; }
  local SRV
  SRV=$(assert_single_server "$label" "$DPID") || exit 1
  local B; B=$(mktemp)
  curl -s -m 10 -o "$B" "http://127.0.0.1:$PORT/$fixture"
  if [ "$(cat "$B")" != "$want" ]; then kill -TERM "$SRV" 2>/dev/null; fail "m87.$label body != oracle"; fi
  rm -f "$B"
  sleep 1
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m87.$label panic in server log (KH88-3)"
  echo "mem_hash=$MEM_HASH git=$GIT_REV campaign_sha=$CAMPAIGN_SHA arm=mem-census phase=$label attempt=$ATT w=1 nreq=1 fixture=$fixture server_exit=$rc seq=$SEQ epoch=$(date +%s)" >> "$MC"
  ledger "attempt=$ATT phase=$label raw=m87.$label.a$ATT.memcensus esito=ok"
}

# run_conc <label> — W=2, pad87a + pad87b in parallel (A-BB53 CONC)
run_conc() {
  local label="$1"
  SEQ=$((SEQ+1))
  local MC="$OUT/m87.$label.a$ATT.memcensus"
  local LOG="$OUT/m87.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 PHPR_CAMPAIGN_SEQ=$SEQ \
    "$OUTBIN/php-server" --axum --workers 2 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || { kill -TERM $DPID 2>/dev/null; fail "m87.$label server not up"; }
  local SRV
  SRV=$(assert_single_server "$label" "$DPID") || exit 1
  local BA BB
  BA=$(mktemp); BB=$(mktemp)
  curl -s -m 10 -o "$BA" "http://127.0.0.1:$PORT/pad87a.php" &
  local C1=$!
  curl -s -m 10 -o "$BB" "http://127.0.0.1:$PORT/pad87b.php" &
  local C2=$!
  wait "$C1" "$C2"
  if [ "$(cat "$BA")" != "$WANTA" ]; then kill -TERM "$DPID" 2>/dev/null; fail "m87.$label pad87a body != oracle"; fi
  if [ "$(cat "$BB")" != "$WANTB" ]; then kill -TERM "$DPID" 2>/dev/null; fail "m87.$label pad87b body != oracle"; fi
  rm -f "$BA" "$BB"
  sleep 1
  kill -TERM "$DPID" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  grep -qE "panicked|aborting" "$LOG" && fail "m87.$label panic in server log (KH88-3)"
  echo "mem_hash=$MEM_HASH git=$GIT_REV campaign_sha=$CAMPAIGN_SHA arm=mem-census phase=$label attempt=$ATT w=2 nreq=2 fixture=pad87a+pad87b server_exit=$rc seq=$SEQ epoch=$(date +%s)" >> "$MC"
  ledger "attempt=$ATT phase=$label raw=m87.$label.a$ATT.memcensus esito=ok"
}

# --- Phase SLOPE ------------------------------------------------------------
echo "== Phase SLOPE (A-DL38≡A-BB52) =="
ENVTAG=none
for w in 1 2 3 4; do
  for r in 1 2 3 4 5; do
    run_arm "slope.w$w.r$r" "$w" $((100*w))
  done
done

# --- Candidate arms (named) -------------------------------------------------
echo "== Phase EAGER (candidate: MIMALLOC_ARENA_EAGER_COMMIT=1) =="
ENVTAG=eager1
for r in 1 2 3; do
  run_arm "eager.w2.r$r" 2 200 MIMALLOC_ARENA_EAGER_COMMIT=1
done
echo "== Phase MINSTACK (candidate: RUST_MIN_STACK=1048576) =="
ENVTAG=minstack1m
for r in 1 2 3; do
  run_arm "minstack.w2.r$r" 2 200 RUST_MIN_STACK=1048576
done
ENVTAG=none

# --- Phase DISJOINT (A-BB53) ------------------------------------------------
echo "== Phase DISJOINT CAL (A-BB53) =="
for r in 1 2; do
  run_pad "cala.r$r" pad87a.php "$WANTA"
  run_pad "calb.r$r" pad87b.php "$WANTB"
done
echo "== Phase DISJOINT CONC (A-BB53) =="
for a in 1 2; do
  run_conc "conc.r$a"
done

head_unmoved
ledger "attempt=$ATT phase=done esito=ok"
# KG-88-3: .done ONLY here (every fail() above exits before this line) and
# verdict87 CONSUMES it (campaign git + attempt inside).
echo "campaign=measure87 git=$GIT_REV attempt=$ATT mem_hash=$MEM_HASH campaign_sha=$CAMPAIGN_SHA" > "$OUT/m87.done"
echo "== measure87 DONE (attempt=$ATT, raws under $OUT/m87.*) =="
