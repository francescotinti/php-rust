#!/bin/bash
# measure89-campaign.sh — S-89.0 p6: ordine Concilio WP-90 Sintesi p6 —
# UNA campagna, ATTRIBUZIONE di b (~21.195.981 B = 20,21 MiB/worker di
# committed marginale, reale e non attribuito in m88).
#   SLOPE-BASE: W in {4,8,12,16}, N=100*W hello, R=5 — protocollo m88
#     invariato (baseline dell attribuzione).
#   SLOPE-RET0 (A-DL46-census/KL-90-4): stesso protocollo con
#     MIMALLOC_PAGE_FULL_RETAIN=0 e read-back ord 36 in-band (default 2 =>
#     il positivo morde da solo, classe A-DL41). Candidato n.1 di Leijen:
#     ogni theap ritiene fino a 2 pagine small PIENE per size-class,
#     committed e mai-free anche a purge_delay=0.
#   EAGER-POS (A-DL44): un run con MIMALLOC_ARENA_EAGER_COMMIT=1 e
#     read-back ord 4 atteso val=1 — il positivo che mancava all ordinale
#     4 (la meta mai armata del read-back v88).
#   CENSUS PER-THEAP (A-DL46-census): ogni raw porta tag=mi_theap_pages/
#     tag=mi_theap_bin al collect — pagine live/free-ritenute per heap,
#     binnate per block_size. DICHIARATO (KL-90-3): identita heap =
#     indice di visita (mapping heap->thread non pubblico); a win=0 il
#     collect e post-teardown (il grosso vive negli abandoned, censiti
#     da tag=mi_bin src=aband).
#   SWEEP (A-BB59, Bak): stagger-sweep INVERTITO — coppie {pad87a,pad87b}
#     con dt in {0,1,2,5,10,20} ms e ordine afirst/bfirst (il flip
#     dell ordine e anche lo swap-fixture control: sotto round-robin
#     inverte il binding fixture<->prima-lane).
#   PREDIZIONI EX-ANTE (nominate PRIMA dei run):
#     P-RET0: se b e ritenzione full-page, b_ret0 <= 0,5*b_base
#             (soglia EX-ANTE; se b_ret0 ~ b_base la ritenzione e
#             esclusa e il residuo va nominato).
#     P-DT0:  a dt=0 le finestre si sovrappongono (spans=OVERLAP
#             OBBLIGATORIO — mapping A-SK58 dichiarato).
#     P-DT20: a dt=20 ms spans=NO-OVERLAP e net==cal AL BYTE su entrambi
#             i lati (ancora m88: zero-swallow a finestre disgiunte).
#     P-ORD:  in regime overlap il surplus sta sul lato che ha SPARATO
#             PER PRIMO (finestra aperta durante il first-touch
#             dell altro) — segue l ORDINE, non il fixture (afirst vs
#             bfirst lo discrimina; m87/m88 hanno mostrato instabilita:
#             una refutazione di P-ORD e essa stessa informazione).
#   DECLARED DEVIATION (per NOME): mi_collect all atexit gira sull heap
#     CONDIVISO v3 (A-DL39 = design); canale $PHPR_MI_STATS NON-CORPUS
#     (A-DL40/KL-89-4) e QUI NON armato; nessuna fase uclog (A-DS45
#     consumata in m88; positivo >=1-coppia in F16b battery).
# Catena evidenza:
#   A-SK46/A-SK50: battery consumata SOLO via battery-equivalence
#     --same-rev (denti pieni + allowlist delta BREV..HEAD).
#   A-BG46/KG-89-1: fail() appende "verdict=VOID reason=…" AL LEDGER
#     (VOID-ness leggibile dal solo ledger); A-SK54: done PER-ATTEMPT
#     (m89.aN.done, name-reuse rifiutato) + pointer m89.done con coerenza
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
BATW="/Volumes/Extreme Pro/Claude/wp89-battery-out"
PORT=8297
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
LEDGER="$OUT/m89.campaign.ledger"
ledger() { echo "attempt_epoch=$(date +%s) git=$GIT_REV campaign_sha=$CAMPAIGN_SHA $*" >> "$LEDGER"; }
ATT=0   # set for real below; fail() before that tags attempt=0 (pre-flight)
fail() {
  echo "FAIL: $*"
  ledger "abort: $*"
  # KG-89-1: the VOID-ness of an attempt must be readable from the ledger
  # ALONE — a terminal verdict=VOID row, appended, never edited.
  # A-BG51 (Council WP-90, Gregg): the pre-flight lane is DECLARED —
  # attempt=0 rows carry phase=preflight (attempt=0 is not an "attempt"
  # under KG-89-1; the lane name keeps the ledger self-explanatory).
  if [ "$ATT" = 0 ]; then
    ledger "attempt=0 phase=preflight verdict=VOID reason=$(echo "$*" | tr ' ' '-')"
  else
    ledger "attempt=$ATT verdict=VOID reason=$(echo "$*" | tr ' ' '-')"
  fi
  exit 1
}

# --- Pre-flight -------------------------------------------------------------
echo "== measure89 pre-flight git=$GIT_REV campaign_sha=$CAMPAIGN_SHA =="
DIRTY="$(git -C "$REPO" status --porcelain 2>/dev/null)"
[ -z "$DIRTY" ] || { echo "$DIRTY" | head -5; fail "tree not porcelain at campaign start"; }
# A-PP43(4): -x sul NOME processo (pgrep -f matcha cmdline estranee/wrapper)
pgrep -x php-server >/dev/null && fail "php-server already running"
UNCOMMITTED=$(git -C "$REPO" status --porcelain -- "wp78-harness/measure-out" 2>/dev/null | grep "m89\." || true)
[ -z "$UNCOMMITTED" ] || { echo "$UNCOMMITTED" | head -5; fail "uncommitted m89.* raws in measure-out (A-SK49/KG-88-1)"; }

ATT=1
while ls "$OUT"/m89.slope.w4.r1.a$ATT.* >/dev/null 2>&1; do ATT=$((ATT+1)); done
echo "attempt=$ATT (filenames carry it; prior generations stay committed)"
ledger "attempt=$ATT phase=start"

# --- A-SK46/A-SK50: battery consumption via the toothed checker only --------
BOUT="$BATW/battery-89pre.out"
[ -f "$BOUT" ] || fail "battery-89pre.out missing — run the battery first"
BREV=$(sed -n 's/^== battery-89pre git=\([0-9a-f]*\) ==$/\1/p' "$BOUT" | head -1)
bash "$REPO/wp83-harness/battery-equivalence.sh" --same-rev "$BOUT" "$BREV" \
  || fail "same-rev consumption REFUSED (A-SK46/A-SK50/KS-SK-89-1)"
DONE="$BATW/.done"
MTX_NAME=$(sed -n 's/.* matrix=\([^ ]*\) .*/\1/p' "$DONE" | head -1)
MTX="$REPO/wp78-harness/matrix-archive/$MTX_NAME"
mtx_hash() { tr -d '\0' < "$MTX" | sed -n "s/^bin\[$1\] sha256\[0:16\]=\([0-9a-f]*\).*/\1/p"; }

# --- Build the mem-census arm, enforce identity -----------------------------
( cd "$REPO" && cargo build --release -p php-server --features mem-census ) \
  > "$OUT/m89.build.mem-census.a$ATT.log" 2>&1 || fail "build mem-census"
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

echo "MEASURED ex-ante: slope-base + slope-ret0 N=100*W R=5 W in {4,8,12,16} | eagerpos 1 run | cal 1 req R=2/side | sweep dt in {0,1,2,5,10,20} ms x {afirst,bfirst}"
echo "PREDICTIONS ex-ante (A-BB59/KL-90-4): P-RET0 b_ret0<=0,5*b_base se ritenzione full-page | P-DT0 OVERLAP obbligatorio | P-DT20 NO-OVERLAP e net==cal AL BYTE | P-ORD surplus sul lato che spara PER PRIMO (ordine, non fixture)"

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

# A-PP46 (Council WP-90, Pedersen — KS-PP-90-1): the fail() that runs
# INSIDE a command-substitution (assert_single_server) cannot kill
# anything in the parent — the old `|| exit 1` left SRV and DPID ALIVE:
# the port-owner assert's own fail-path PRODUCED the orphan it exists to
# prevent. The parent-side handler kills the tree, verifies the server is
# GONE (best-effort, 5s), LEDGERS the outcome in-band, then exits.
subshell_failpath() { # <dpid> <label>
  local dpid="$1" label="$2" gone=no i
  kill_server_tree "$dpid"
  for i in 1 2 3 4 5 6 7 8 9 10; do
    pgrep -x php-server >/dev/null || { gone=yes; break; }
    sleep 0.5
  done
  ledger "attempt=$ATT phase=$label failpath=assert_single_server server_gone=$gone"
  exit 1
}
# A-PP46(2): body-mismatch teardown — a TERM alone can be ignored and the
# survivor would ambush the NEXT campaign's pre-flight. Kill the tree,
# verify server-gone best-effort, LEDGER it, then fail in-band.
teardown_fail() { # <srv> <dpid> <label> <msg...>
  local srv="$1" dpid="$2" label="$3"; shift 3
  kill -TERM "$srv" 2>/dev/null
  kill_server_tree "$dpid"
  local gone=no i
  for i in 1 2 3 4 5 6 7 8 9 10; do
    pgrep -x php-server >/dev/null || { gone=yes; break; }
    sleep 0.5
  done
  ledger "attempt=$ATT phase=$label failpath=teardown server_gone=$gone"
  fail "$*"
}
# A-BG51 (Council WP-90, Gregg): pid-echo — the FIRST request of every
# phase must return `x-phpr-pid` equal to the asserted LISTEN-owner pid
# (a stale or foreign server cannot echo the fresh pid). Uses the caller's
# DPID (bash dynamic scoping) for the teardown path.
assert_http_pid() { # <label> <srv>
  local hp
  hp=$(curl -s -o /dev/null -m 5 -D - "http://127.0.0.1:$PORT/__reqns" | tr -d '\r' | awk 'tolower($1)=="x-phpr-pid:"{print $2; exit}')
  [ "$hp" = "$2" ] || teardown_fail "$2" "$DPID" "$1" "m89.$1 x-phpr-pid='$hp' != srv=$2 (A-BG51 pid-echo)"
}

assert_single_server() { # <label> <owner-pid> -> echoes the server pid
  local label="$1" owner="$2" pids srv
  pids=$(pgrep -x php-server || true)
  [ -n "$pids" ] || { fail "m89.$label no php-server process after up (anti-orphan a)" 1>&2; }
  [ "$(echo "$pids" | grep -c .)" = 1 ] || { echo "$pids" 1>&2; fail "m89.$label MULTIPLE php-server processes (anti-orphan b)" 1>&2; }
  srv="$pids"
  if [ "$srv" != "$owner" ] && [ "$(ps -o ppid= -p "$srv" | tr -d ' ')" != "$owner" ]; then
    fail "m89.$label the live php-server ($srv) is not ours ($owner) — orphan serving (anti-orphan b)" 1>&2
  fi
  # A-PP43/KS-PP-89-4: the LISTEN-owner of the port must BE that pid — a
  # name census alone never proves ownership (renamed binary, third party).
  local lowner
  lowner=$(lsof -nP -iTCP:$PORT -sTCP:LISTEN -t 2>/dev/null | sort -u)
  [ "$lowner" = "$srv" ] || { echo "listen-owner=$lowner srv=$srv" 1>&2; fail "m89.$label port $PORT LISTEN-owner is not our server (A-PP43/KS-PP-89-4)" 1>&2; }
  echo "$srv"
}
assert_server_gone() { # <label>
  local i
  for i in 1 2 3 4 5 6 7 8 9 10; do
    pgrep -x php-server >/dev/null || return 0
    sleep 0.5
  done
  fail "m89.$1 php-server still alive after teardown (anti-orphan c)"
}

# identity v2 (A-BG47): srv_pid + srv_boot_epoch in-band, judged by VIDENT.
identity_row() { # <MC> <phase> <w> <nreq> <extra>
  echo "mem_hash=$MEM_HASH git=$GIT_REV campaign_sha=$CAMPAIGN_SHA arm=mem-census phase=$2 attempt=$ATT w=$3 nreq=$4 envtag=${ENVTAG:-none} srv_pid=$SRV srv_boot_epoch=$BOOT_EPOCH server_exit=$5 seq=$SEQ epoch=$(date +%s)${6:+ $6}" >> "$1"
}

# run_arm <label> <workers> <nreq> <extra-env...>
run_arm() {
  local label="$1" workers="$2" nreq="$3"; shift 3
  SEQ=$((SEQ+1))
  local MC="$OUT/m89.$label.a$ATT.memcensus"
  local LOG="$OUT/m89.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  env "$@" PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 \
      PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers "$workers" --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || { kill_server_tree $DPID; fail "m89.$label server not up"; }
  local SRV BOOT_EPOCH
  SRV=$(assert_single_server "$label" "$DPID") || subshell_failpath "$DPID" "$label"
  BOOT_EPOCH=$(date +%s)
  assert_http_pid "$label" "$SRV"
  local ok=0 i=0 B
  B=$(mktemp)
  while [ $i -lt "$nreq" ]; do
    curl -s -m 10 -o "$B" "http://127.0.0.1:$PORT/hello.php"
    [ "$(cat "$B")" = "$WANTH" ] && ok=$((ok+1))
    i=$((i+1))
  done
  rm -f "$B"
  [ "$ok" = "$nreq" ] || { teardown_fail "$SRV" "$DPID" "$label" "m89.$label bodies $ok/$nreq != oracle"; }
  sleep 2
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m89.$label panic in server log (KH88-3)"
  identity_row "$MC" "$label" "$workers" "$nreq" "$rc"
  ledger "attempt=$ATT phase=$label raw=m89.$label.a$ATT.memcensus esito=ok nreq=$nreq"
}

# run_pad <label> <fixture> <want> — W=1 single-request calibration
run_pad() {
  local label="$1" fixture="$2" want="$3"
  SEQ=$((SEQ+1))
  local MC="$OUT/m89.$label.a$ATT.memcensus"
  local LOG="$OUT/m89.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers 1 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || { kill_server_tree $DPID; fail "m89.$label server not up"; }
  local SRV BOOT_EPOCH
  SRV=$(assert_single_server "$label" "$DPID") || subshell_failpath "$DPID" "$label"
  BOOT_EPOCH=$(date +%s)
  assert_http_pid "$label" "$SRV"
  local B; B=$(mktemp)
  curl -s -m 10 -o "$B" "http://127.0.0.1:$PORT/$fixture"
  if [ "$(cat "$B")" != "$want" ]; then teardown_fail "$SRV" "$DPID" "$label" "m89.$label body != oracle"; fi
  rm -f "$B"
  sleep 1
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m89.$label panic in server log (KH88-3)"
  identity_row "$MC" "$label" 1 1 "$rc" "fixture=$fixture"
  ledger "attempt=$ATT phase=$label raw=m89.$label.a$ATT.memcensus esito=ok"
}

# run_sweep <label> <dt_ms> <first:a|b> — W=2 pair, A-BB59 stagger-sweep
run_sweep() {
  local label="$1" dt="$2" first="$3"
  SEQ=$((SEQ+1))
  local MC="$OUT/m89.$label.a$ATT.memcensus"
  local LOG="$OUT/m89.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers 2 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || { kill_server_tree $DPID; fail "m89.$label server not up"; }
  local SRV BOOT_EPOCH
  SRV=$(assert_single_server "$label" "$DPID") || subshell_failpath "$DPID" "$label"
  BOOT_EPOCH=$(date +%s)
  assert_http_pid "$label" "$SRV"
  # A-BB59 (Council WP-90, Bak): inverted asymmetric stagger-sweep. The
  # pair is always {pad87a, pad87b}; `first` names WHICH pad fires first
  # (afirst/bfirst = the swap-fixture control: flipping the order also
  # flips the fixture<->first-lane binding under round-robin), `dt` the
  # delay in ms before the second fire.
  local FF WF SS WS
  if [ "$first" = a ]; then FF=pad87a.php; WF="$WANTA"; SS=pad87b.php; WS="$WANTB"
  else FF=pad87b.php; WF="$WANTB"; SS=pad87a.php; WS="$WANTA"; fi
  local BA BB
  BA=$(mktemp); BB=$(mktemp)
  curl -s -m 10 -o "$BA" "http://127.0.0.1:$PORT/$FF" &
  local C1=$!
  if [ "$dt" != 0 ]; then sleep "$(awk -v d="$dt" 'BEGIN{printf "%.3f", d/1000}')"; fi
  curl -s -m 10 -o "$BB" "http://127.0.0.1:$PORT/$SS" &
  local C2=$!
  wait "$C1" "$C2"
  if [ "$(cat "$BA")" != "$WF" ]; then teardown_fail "$SRV" "$DPID" "$label" "m89.$label $FF body != oracle"; fi
  if [ "$(cat "$BB")" != "$WS" ]; then teardown_fail "$SRV" "$DPID" "$label" "m89.$label $SS body != oracle"; fi
  rm -f "$BA" "$BB"
  sleep 1
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m89.$label panic in server log (KH88-3)"
  identity_row "$MC" "$label" 2 2 "$rc" "fixture=$FF+$SS dt_ms=$dt first=$first"
  ledger "attempt=$ATT phase=$label raw=m89.$label.a$ATT.memcensus esito=ok dt_ms=$dt first=$first"
}

# (fase UCLOG RIMOSSA in measure89 — A-DS45 consumata in m88; DICHIARATO:
# nessun canale PHPR_UNIT_CACHE_LOG armato in questa campagna, il positivo
# >=1-coppia vive in F16b in battery. KS-DS-90-1 non innescabile.)

# --- Phase SLOPE-BASE (baseline arm, come m88) -------------------------------
echo "== Phase SLOPE-BASE (W in {4,8,12,16}, R=5, env default) =="
ENVTAG=none
for w in 4 8 12 16; do
  for r in 1 2 3 4 5; do
    run_arm "slope.w$w.r$r" "$w" $((100*w))
  done
done

# --- Phase SLOPE-RET0 (braccio discriminante KL-90-4) -------------------------
# MIMALLOC_PAGE_FULL_RETAIN=0 con read-back ord 36 atteso val=0 (default 2:
# il positivo morde da solo, classe A-DL41). P-RET0 ex-ante nel header.
echo "== Phase SLOPE-RET0 (MIMALLOC_PAGE_FULL_RETAIN=0, read-back ord 36) =="
ENVTAG=ret0
for w in 4 8 12 16; do
  for r in 1 2 3 4 5; do
    run_arm "slope0.w$w.r$r" "$w" $((100*w)) MIMALLOC_PAGE_FULL_RETAIN=0
  done
done

# --- Phase EAGER-POS (A-DL44: positivo dell'ordinale 4) -----------------------
echo "== Phase EAGER-POS (MIMALLOC_ARENA_EAGER_COMMIT=1, read-back ord 4) =="
ENVTAG=eager1
run_arm "eagerpos.w4.r1" 4 400 MIMALLOC_ARENA_EAGER_COMMIT=1
ENVTAG=none

# --- Phase CAL (calibrazioni per lo sweep) ------------------------------------
echo "== Phase CAL =="
for r in 1 2; do
  run_pad "cala.r$r" pad87a.php "$WANTA"
  run_pad "calb.r$r" pad87b.php "$WANTB"
done

# --- Phase SWEEP (A-BB59: stagger invertito + swap-fixture) -------------------
echo "== Phase SWEEP (dt in {0,1,2,5,10,20} ms x {afirst,bfirst}) =="
for dt in 0 1 2 5 10 20; do
  run_sweep "sweep.dt$dt.afirst" "$dt" a
  run_sweep "sweep.dt$dt.bfirst" "$dt" b
done

head_unmoved
ledger "attempt=$ATT phase=done esito=ok"
# A-SK54: per-attempt done (name NEVER reused) + guarded pointer; the
# verdict judges pointer<->suffixed coherence.
ADONE="$OUT/m89.a$ATT.done"
[ -e "$ADONE" ] && fail "m89.a$ATT.done already exists — name reuse refused (A-SK54/KG-88-1)"
echo "campaign=measure89 git=$GIT_REV attempt=$ATT mem_hash=$MEM_HASH campaign_sha=$CAMPAIGN_SHA" > "$ADONE"
cp "$ADONE" "$OUT/m89.done"
echo "== measure89 DONE (attempt=$ATT, raws under $OUT/m89.*) =="
