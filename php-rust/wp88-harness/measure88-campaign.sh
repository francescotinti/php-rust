#!/bin/bash
# measure88-campaign.sh — S-88.0 p5: ordine Concilio WP-89 §Sintesi p5.
#   SLOPE-HI (A-BB55≡A-DL42): il committed di processo a W∈{1..4} è una
#     FUNZIONE A GRADINI da 64 KiB (Bak+Leijen WP-89: min-of-R tutti multipli
#     esatti di 65.536 B; W=3 bimodale) — la LSQ W1..4 è ADVISORY (KB-89-1).
#     Qui: W∈{4,8,12,16}, N=100·W hello sequenziali, R=5 per W; giudice a
#     valle = mode-census per W (KB-89-2) + LSQ di b sul segmento alto;
#     banda KL-85-2 3.605.572 B ±5% confrontata SOLO con b (KL-89-2), mai
#     con la slope grezza. Righe tag=mi_arena/tag=mi_arena_json in-band
#     (A-BB55) nominano il termine quantizzato c(W): SOSTITUZIONE NOMINATA —
#     per-arena non enumerabile pubblicamente (mi_arena_id_t opaco in v3) ⇒
#     stats di processo mimalloc (arena_count, committed, reserved, purged,
#     commit_calls) + chunk_bins census, via mi_stats_get_json (struct
#     costruita DENTRO mimalloc, canale atomico nostro).
#   WARMPAIR (A-BB56, Bak Q3): surplus padA ~3.146.416 B = 3 MiB + resto.
#     PREDIZIONI EX-ANTE (nominate PRIMA dei run):
#       P-BASE: concbase riproduce la firma measure87 — padB ≈ calA+calB+~310 B,
#               padA ≈ calA+calB+~3,15 MB (intra-campagna, stessa rev).
#       P-WARM: warm-both-then-pair (1 hello per worker PRIMA della coppia):
#               se il driver del surplus è il FIRST-TOUCH del 2° worker,
#               dA(warm) CROLLA all'ordine di dB (~centinaia di B); se resta
#               ~3 MiB il driver è per-richiesta (buffer axum/tokio).
#       P-STAG: stagger 20 ms su pad87b: il first-touch di B esce dalla
#               finestra process-counters di A ⇒ dA(stag) << dA(base).
#     I net concorrenti restano VOID come cifre per-thread (KB-88-1) fino
#     ad A-BB50 attuato: i run esistono per DISCRIMINARE il driver.
#   UCLOG (A-DS45, Stogov): UNA fase W=1 con PHPR_UNIT_CACHE_LOG ARMATO su
#     log di PRODUZIONE: touch del fixture fra richieste ⇒ supersede ⇒
#     coppie main_evicted/evict-fp; putord-pair-guard su log NON-selftest
#     (≥1 coppia, KS-DS-88-1). W=1-only: putord senza thr= in-band non è
#     opponibile a W>1 (KH89-1).
#   DECLARED DEVIATION (per NOME): mi_collect all'atexit gira sull'heap
#     CONDIVISO v3, non per-theap (A-DL39 = design); il canale
#     $PHPR_MI_STATS è NON-CORPUS (A-DL40/KL-89-4) e QUI NON viene armato.
# Catena evidenza:
#   A-SK46/A-SK50: battery consumata SOLO via battery-equivalence
#     --same-rev (denti pieni + allowlist delta BREV..HEAD).
#   A-BG46/KG-89-1: fail() appende "verdict=VOID reason=…" AL LEDGER
#     (VOID-ness leggibile dal solo ledger); A-SK54: done PER-ATTEMPT
#     (m88.aN.done, name-reuse rifiutato) + pointer m88.done con coerenza
#     giudicata dal verdict.
#   A-BG47/KG-89-2: identity contract v2 — srv_pid= (figlio reale, mai $!)
#     + srv_boot_epoch= + server_exit= giudicati da VIDENT.
#   A-PP43/KS-PP-89-4: port-owner assert (lsof LISTEN pid==SRV) in OGNI
#     run; assert_server_gone anche in run_conc; failure-path kill sul
#     FIGLIO, mai solo sul wrapper.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$REPO/wp78-harness/measure-out"
OUTBIN="$HOME/Claude/php-rust-output/release"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
ORACLE="/opt/homebrew/opt/php/bin/php"
BATW="/Volumes/Extreme Pro/Claude/wp88-battery-out"
PORT=8296
CAMPAIGN_SHA=$(shasum -a 256 "$0" | cut -c1-16)
export PHPR_CAMPAIGN_SCRIPT="$0"
SEQ=0

GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
HEAD_AT_START="$GIT_REV"
head_unmoved() {
  local now; now="$(git -C "$REPO" rev-parse --short HEAD)"
  [ "$now" = "$HEAD_AT_START" ] || { echo "FAIL: HEAD moved mid-campaign ($now != $HEAD_AT_START)"; exit 1; }
}

# --- A-BG43/A-BG46: APPEND-only ledger; VOID rows IN-BAND -------------------
LEDGER="$OUT/m88.campaign.ledger"
ledger() { echo "attempt_epoch=$(date +%s) git=$GIT_REV campaign_sha=$CAMPAIGN_SHA $*" >> "$LEDGER"; }
ATT=0   # set for real below; fail() before that tags attempt=0 (pre-flight)
fail() {
  echo "FAIL: $*"
  ledger "abort: $*"
  # KG-89-1: the VOID-ness of an attempt must be readable from the ledger
  # ALONE — a terminal verdict=VOID row, appended, never edited.
  ledger "attempt=$ATT verdict=VOID reason=$(echo "$*" | tr ' ' '-')"
  exit 1
}

# --- Pre-flight -------------------------------------------------------------
echo "== measure88 pre-flight git=$GIT_REV campaign_sha=$CAMPAIGN_SHA =="
DIRTY="$(git -C "$REPO" status --porcelain 2>/dev/null)"
[ -z "$DIRTY" ] || { echo "$DIRTY" | head -5; fail "tree not porcelain at campaign start"; }
# A-PP43(4): -x sul NOME processo (pgrep -f matcha cmdline estranee/wrapper)
pgrep -x php-server >/dev/null && fail "php-server already running"
UNCOMMITTED=$(git -C "$REPO" status --porcelain -- "wp78-harness/measure-out" 2>/dev/null | grep "m88\." || true)
[ -z "$UNCOMMITTED" ] || { echo "$UNCOMMITTED" | head -5; fail "uncommitted m88.* raws in measure-out (A-SK49/KG-88-1)"; }

ATT=1
while ls "$OUT"/m88.slope.w4.r1.a$ATT.* >/dev/null 2>&1; do ATT=$((ATT+1)); done
echo "attempt=$ATT (filenames carry it; prior generations stay committed)"
ledger "attempt=$ATT phase=start"

# --- A-SK46/A-SK50: battery consumption via the toothed checker only --------
BOUT="$BATW/battery-88pre.out"
[ -f "$BOUT" ] || fail "battery-88pre.out missing — run the battery first"
BREV=$(sed -n 's/^== battery-88pre git=\([0-9a-f]*\) ==$/\1/p' "$BOUT" | head -1)
bash "$REPO/wp83-harness/battery-equivalence.sh" --same-rev "$BOUT" "$BREV" \
  || fail "same-rev consumption REFUSED (A-SK46/A-SK50/KS-SK-89-1)"
DONE="$BATW/.done"
MTX_NAME=$(sed -n 's/.* matrix=\([^ ]*\) .*/\1/p' "$DONE" | head -1)
MTX="$REPO/wp78-harness/matrix-archive/$MTX_NAME"
mtx_hash() { tr -d '\0' < "$MTX" | sed -n "s/^bin\[$1\] sha256\[0:16\]=\([0-9a-f]*\).*/\1/p"; }

# --- Build the mem-census arm, enforce identity -----------------------------
( cd "$REPO" && cargo build --release -p php-server --features mem-census ) \
  > "$OUT/m88.build.mem-census.a$ATT.log" 2>&1 || fail "build mem-census"
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

echo "MEASURED ex-ante: slope N=100*W R=5 W in {4,8,12,16} | cal 1 req R=2/side | concbase/concwarm/concstag 2 attempts each | uclog W=1 (armed)"
echo "PREDICTIONS ex-ante (A-BB56): P-BASE padB=calA+calB+~310 B, padA=calA+calB+~3,15 MB | P-WARM first-touch => dA(warm) collapses to O(dB) | P-STAG dA(stag) << dA(base)"

wait_up() {
  local up=0
  for _ in $(seq 1 100); do
    curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/__reqns" && { up=1; break; }
    sleep 0.1
  done
  [ $up = 1 ]
}

# A-PP43 failure-path: kill the CHILD of the wrapper (the real server),
# then the wrapper — never ONLY the wrapper (attempt=1 lesson applied to
# the abort path too).
kill_server_tree() { # <wrapper-pid>
  local wpid="$1" child
  child=$(pgrep -P "$wpid" -x php-server 2>/dev/null || true)
  [ -n "$child" ] && kill -TERM $child 2>/dev/null
  kill -TERM "$wpid" 2>/dev/null
}

assert_single_server() { # <label> <owner-pid> -> echoes the server pid
  local label="$1" owner="$2" pids srv
  pids=$(pgrep -x php-server || true)
  [ -n "$pids" ] || { fail "m88.$label no php-server process after up (anti-orphan a)" 1>&2; }
  [ "$(echo "$pids" | grep -c .)" = 1 ] || { echo "$pids" 1>&2; fail "m88.$label MULTIPLE php-server processes (anti-orphan b)" 1>&2; }
  srv="$pids"
  if [ "$srv" != "$owner" ] && [ "$(ps -o ppid= -p "$srv" | tr -d ' ')" != "$owner" ]; then
    fail "m88.$label the live php-server ($srv) is not ours ($owner) — orphan serving (anti-orphan b)" 1>&2
  fi
  # A-PP43/KS-PP-89-4: the LISTEN-owner of the port must BE that pid — a
  # name census alone never proves ownership (renamed binary, third party).
  local lowner
  lowner=$(lsof -nP -iTCP:$PORT -sTCP:LISTEN -t 2>/dev/null | sort -u)
  [ "$lowner" = "$srv" ] || { echo "listen-owner=$lowner srv=$srv" 1>&2; fail "m88.$label port $PORT LISTEN-owner is not our server (A-PP43/KS-PP-89-4)" 1>&2; }
  echo "$srv"
}
assert_server_gone() { # <label>
  local i
  for i in 1 2 3 4 5 6 7 8 9 10; do
    pgrep -x php-server >/dev/null || return 0
    sleep 0.5
  done
  fail "m88.$1 php-server still alive after teardown (anti-orphan c)"
}

# identity v2 (A-BG47): srv_pid + srv_boot_epoch in-band, judged by VIDENT.
identity_row() { # <MC> <phase> <w> <nreq> <extra>
  echo "mem_hash=$MEM_HASH git=$GIT_REV campaign_sha=$CAMPAIGN_SHA arm=mem-census phase=$2 attempt=$ATT w=$3 nreq=$4 envtag=${ENVTAG:-none} srv_pid=$SRV srv_boot_epoch=$BOOT_EPOCH server_exit=$5 seq=$SEQ epoch=$(date +%s)${6:+ $6}" >> "$1"
}

# run_arm <label> <workers> <nreq> <extra-env...>
run_arm() {
  local label="$1" workers="$2" nreq="$3"; shift 3
  SEQ=$((SEQ+1))
  local MC="$OUT/m88.$label.a$ATT.memcensus"
  local LOG="$OUT/m88.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  env "$@" PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 \
      PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers "$workers" --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || { kill_server_tree $DPID; fail "m88.$label server not up"; }
  local SRV BOOT_EPOCH
  SRV=$(assert_single_server "$label" "$DPID") || exit 1
  BOOT_EPOCH=$(date +%s)
  local ok=0 i=0 B
  B=$(mktemp)
  while [ $i -lt "$nreq" ]; do
    curl -s -m 10 -o "$B" "http://127.0.0.1:$PORT/hello.php"
    [ "$(cat "$B")" = "$WANTH" ] && ok=$((ok+1))
    i=$((i+1))
  done
  rm -f "$B"
  [ "$ok" = "$nreq" ] || { kill -TERM "$SRV" 2>/dev/null; fail "m88.$label bodies $ok/$nreq != oracle"; }
  sleep 2
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m88.$label panic in server log (KH88-3)"
  identity_row "$MC" "$label" "$workers" "$nreq" "$rc"
  ledger "attempt=$ATT phase=$label raw=m88.$label.a$ATT.memcensus esito=ok nreq=$nreq"
}

# run_pad <label> <fixture> <want> — W=1 single-request calibration
run_pad() {
  local label="$1" fixture="$2" want="$3"
  SEQ=$((SEQ+1))
  local MC="$OUT/m88.$label.a$ATT.memcensus"
  local LOG="$OUT/m88.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers 1 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || { kill_server_tree $DPID; fail "m88.$label server not up"; }
  local SRV BOOT_EPOCH
  SRV=$(assert_single_server "$label" "$DPID") || exit 1
  BOOT_EPOCH=$(date +%s)
  local B; B=$(mktemp)
  curl -s -m 10 -o "$B" "http://127.0.0.1:$PORT/$fixture"
  if [ "$(cat "$B")" != "$want" ]; then kill -TERM "$SRV" 2>/dev/null; fail "m88.$label body != oracle"; fi
  rm -f "$B"
  sleep 1
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m88.$label panic in server log (KH88-3)"
  identity_row "$MC" "$label" 1 1 "$rc" "fixture=$fixture"
  ledger "attempt=$ATT phase=$label raw=m88.$label.a$ATT.memcensus esito=ok"
}

# run_conc <label> <mode> — W=2 pair; mode=base|warm|stag (A-BB56)
run_conc() {
  local label="$1" mode="$2"
  SEQ=$((SEQ+1))
  local MC="$OUT/m88.$label.a$ATT.memcensus"
  local LOG="$OUT/m88.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers 2 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || { kill_server_tree $DPID; fail "m88.$label server not up"; }
  local SRV BOOT_EPOCH
  SRV=$(assert_single_server "$label" "$DPID") || exit 1
  BOOT_EPOCH=$(date +%s)
  local nreq=2
  if [ "$mode" = warm ]; then
    # P-WARM: one hello PER WORKER (round-robin W=2 => exactly one each,
    # judged by VDISP) BEFORE the pair: the first-touch happens OUTSIDE
    # the pair's lower windows.
    local BW; BW=$(mktemp)
    curl -s -m 10 -o "$BW" "http://127.0.0.1:$PORT/hello.php"
    [ "$(cat "$BW")" = "$WANTH" ] || { rm -f "$BW"; kill -TERM "$SRV" 2>/dev/null; fail "m88.$label warm body 1 != oracle"; }
    curl -s -m 10 -o "$BW" "http://127.0.0.1:$PORT/hello.php"
    [ "$(cat "$BW")" = "$WANTH" ] || { rm -f "$BW"; kill -TERM "$SRV" 2>/dev/null; fail "m88.$label warm body 2 != oracle"; }
    rm -f "$BW"
    nreq=4
    sleep 0.3
  fi
  local BA BB
  BA=$(mktemp); BB=$(mktemp)
  curl -s -m 10 -o "$BA" "http://127.0.0.1:$PORT/pad87a.php" &
  local C1=$!
  if [ "$mode" = stag ]; then sleep 0.02; fi   # P-STAG: 20 ms
  curl -s -m 10 -o "$BB" "http://127.0.0.1:$PORT/pad87b.php" &
  local C2=$!
  wait "$C1" "$C2"
  if [ "$(cat "$BA")" != "$WANTA" ]; then kill -TERM "$SRV" 2>/dev/null; fail "m88.$label pad87a body != oracle"; fi
  if [ "$(cat "$BB")" != "$WANTB" ]; then kill -TERM "$SRV" 2>/dev/null; fail "m88.$label pad87b body != oracle"; fi
  rm -f "$BA" "$BB"
  sleep 1
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"   # A-PP43(2): the 87 run_conc lacked this
  grep -qE "panicked|aborting" "$LOG" && fail "m88.$label panic in server log (KH88-3)"
  identity_row "$MC" "$label" 2 "$nreq" "$rc" "fixture=pad87a+pad87b mode=$mode"
  ledger "attempt=$ATT phase=$label raw=m88.$label.a$ATT.memcensus esito=ok mode=$mode"
}

# run_uclog <label> — A-DS45: W=1, PHPR_UNIT_CACHE_LOG armed, PRODUCTION
# pairs via supersede (mtime bump between requests). W=1-only (KH89-1).
run_uclog() {
  local label="$1"
  SEQ=$((SEQ+1))
  local MC="$OUT/m88.$label.a$ATT.memcensus"
  local LOG="$OUT/m88.$label.a$ATT.log"
  local UCL="$OUT/m88.$label.a$ATT.uclog"
  for f in "$MC" "$UCL"; do
    [ -e "$f" ] && fail "raw $f already exists — name reuse refused (KG-88-1)"
  done
  : > "$MC"
  head_unmoved
  PHPR_MEM_CENSUS="$MC" PHPR_UNIT_CACHE_LOG="$UCL" PHPR_MI_COLLECT_EXIT=1 \
      MIMALLOC_PURGE_DELAY=0 PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers 1 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || { kill_server_tree $DPID; fail "m88.$label server not up"; }
  local SRV BOOT_EPOCH
  SRV=$(assert_single_server "$label" "$DPID") || exit 1
  BOOT_EPOCH=$(date +%s)
  local B; B=$(mktemp)
  # 3 generations of hello.php: 2 supersedes => >=2 production pairs.
  local g
  for g in 1 2 3; do
    curl -s -m 10 -o "$B" "http://127.0.0.1:$PORT/hello.php"
    [ "$(cat "$B")" = "$WANTH" ] || { rm -f "$B"; kill -TERM "$SRV" 2>/dev/null; fail "m88.$label gen$g body != oracle"; }
    [ "$g" = 3 ] || { sleep 1.1; touch "$FIXDIR/hello.php"; }
  done
  rm -f "$B"
  sleep 1
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m88.$label panic in server log (KH88-3)"
  identity_row "$MC" "$label" 1 3 "$rc" "uclog=m88.$label.a$ATT.uclog"
  # In-campaign positive (fail-closed): the guard must verify >=1 pair on
  # this PRODUCTION log (KS-DS-88-1 lifted from vacuous-selftest-only).
  local GOUT
  GOUT=$("$REPO/wp87-harness/putord-pair-guard.pl" "$UCL") || fail "m88.$label putord-pair-guard VOID on production log (A-DS45)"
  echo "$GOUT"
  echo "$GOUT" | grep -qE "^putord-pair-guard: [1-9][0-9]* pair" \
    || fail "m88.$label production log has ZERO pairs — phase vacuous (A-DS45)"
  ledger "attempt=$ATT phase=$label raw=m88.$label.a$ATT.memcensus uclog=m88.$label.a$ATT.uclog esito=ok pairs=$(echo "$GOUT" | sed -n 's/^putord-pair-guard: \([0-9]*\) pair.*/\1/p')"
}

# --- Phase SLOPE-HI (A-BB55≡A-DL42) -----------------------------------------
echo "== Phase SLOPE-HI (A-BB55: W in {4,8,12,16}, mode-census downstream) =="
ENVTAG=none
for w in 4 8 12 16; do
  for r in 1 2 3 4 5; do
    run_arm "slope.w$w.r$r" "$w" $((100*w))
  done
done

# --- Phase WARMPAIR (A-BB56) ------------------------------------------------
echo "== Phase WARMPAIR CAL =="
for r in 1 2; do
  run_pad "cala.r$r" pad87a.php "$WANTA"
  run_pad "calb.r$r" pad87b.php "$WANTB"
done
echo "== Phase WARMPAIR CONC (base | warm | stag) =="
for a in 1 2; do run_conc "concbase.r$a" base; done
for a in 1 2; do run_conc "concwarm.r$a" warm; done
for a in 1 2; do run_conc "concstag.r$a" stag; done

# --- Phase UCLOG (A-DS45) ---------------------------------------------------
echo "== Phase UCLOG (armed, production pairs) =="
run_uclog "uclog"

head_unmoved
ledger "attempt=$ATT phase=done esito=ok"
# A-SK54: per-attempt done (name NEVER reused) + guarded pointer; the
# verdict judges pointer<->suffixed coherence.
ADONE="$OUT/m88.a$ATT.done"
[ -e "$ADONE" ] && fail "m88.a$ATT.done already exists — name reuse refused (A-SK54/KG-88-1)"
echo "campaign=measure88 git=$GIT_REV attempt=$ATT mem_hash=$MEM_HASH campaign_sha=$CAMPAIGN_SHA" > "$ADONE"
cp "$ADONE" "$OUT/m88.done"
echo "== measure88 DONE (attempt=$ATT, raws under $OUT/m88.*) =="
