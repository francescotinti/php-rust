#!/bin/bash
# measure90-campaign.sh — S-90.0 p5: ordine Concilio WP-91 Sintesi p5 —
# ATTRIBUZIONE di b, ITERAZIONE 2, con metrica RIQUALIFICATA PEAK
# (A-DL48/KL-91-1: commit==peak_commit su macOS — b e' la pendenza del
# PICCO di commit, non del residente; i knob post-picco sono ritirati,
# KL-91-2; ord44 ritirato dalla lane commit, A-DL51).
#
# BRACCI (team-misura WP-91, ordine per potere discriminante):
#   SLOPE-PEAK + SCALA Δcommitted PER FASE (A-DL49 v2 + braccio 2 Leijen):
#     W in {4,8,12,16}, N=100*W hello, R=5 — protocollo m89 BASE con la
#     SCALA di checkpoint in-request NOMINATI (KG-91-2):
#       ckpt=peak_inreq_pboot  (post-wait_up/asserts, pre-workload)
#       ckpt=peak_inreq_pwork  (post-workload = PICCO, worker VIVI)
#       + W richieste /__census_self (census per-worker SELF, A-DL31)
#       + ckpt=peak_inreq      (picco finale, post-self)
#     Il contatore commit e' MONOTONO (A-DL48) ⇒ i Δ per fase sono ESATTI
#     (lezione WP-88): b si decompone in b_boot + b_work senza census.
#   ⚠️ RIQUALIFICA DAL CANALE REALE (smoke S-90.0, PRIMA della campagna):
#     (a) mi_subproc_visit_heaps NON vede i theap VIVI di thread estranei
#         (heaps_total=1 al picco, coverage ~40% del commit) — il pin
#         heaps_total==W+1 della lettera A-DL49 e' byte-impossibile su
#         questo canale; (b) mi_heap_of(probe) su mimalloc v3 restituisce
#         lo STESSO heap per tutti i worker (collisione rilevata dal
#         dente onesta' A-DL31 heap=<ptr>): lo split per-worker va
#         DICHIARATO REFUSED dal giudice, non benedetto. L'attribuzione
#         poggia sulla SCALA Δcommitted (esatta) + census globale
#         ADVISORY (KL-91-3, coverage <0,9 dichiarata).
#   CAL: pad87a/b x2 per lato (quinta campagna; il BINARIO census e'
#     CAMBIATO — A-MS50/ckpt/self — quindi la predizione byte contro
#     7.801.102 B e' ADVISORY: pari fra r1/r2 resta REQUIRED).
#   SWEEP-PD0: dt in {1,5,20} x {afirst,bfirst} con PHPR_MI_COLLECT_REQ=1
#     (dump per-request ⇒ purged cumulativo per finestra-richiesta).
#   SWEEP-PDBIG (A-BB63+A-DL50 unificati, Bak Q3+Leijen Q4): dt in {1,5}
#     x {afirst,bfirst} con MIMALLOC_PURGE_DELAY=1000 (>= finestra) e
#     read-back ord 15 atteso val=1000.
#
# PREDIZIONI EX-ANTE (nominate PRIMA dei run; soglie NEL HEADER, KB-91-2):
#   P-PEAK90:  commit==peak_commit su OGNI riga mi_proc di OGNI raw
#              (win=0 e win=9; KL-91-1: una violazione ⇒ cifre VOID).
#   P-LADDER:  commit(pboot) <= commit(pwork) <= commit(peak_inreq) ==
#              commit(exit) per ogni raw slope (monotonia A-DL48); la
#              scomposizione b_boot+b_work e' additiva rispetto a b
#              entro la fascia δ=0,15.
#   P-SELF:    le W righe self portano thr distinti 0..W-1; se i heap
#              pointer COLLIDONO (v3), lo split per-worker e' REFUSED
#              per A-DL31 — esito DICHIARATO, non FAIL di campagna.
#   P-CLAMP-PD: con purge_delay=1000 >= finestra, clamped==0 su TUTTI i
#              lati dt in {1,5} e purged==0 ad ogni dump per-request
#              (nessuna purge dentro le finestre); con pd=0, ogni raw
#              clamped porta purged>0 all'ultimo dump per-request
#              (forma operativa A-DL50 sulla scala per-richiesta).
#   P-DT20:    a dt=20 spans NO-OVERLAP e net==cal AL BYTE (ancora m88).
#   SOGLIE: fascia δ=0,15; coverage census 0,9 (sotto = ADVISORY,
#           KL-91-3); robustezza ratio 0,8; tie mode-census in-band
#           (tie=K tiebreak=min, A-BB61).
#
# Catena evidenza (KG-88-1 + WP-91): battery consumata SOLO via
# battery-equivalence --same-rev (v6 + A-AH50/A-AH54) e la consumazione e'
# LEDGERATA (A-AH57: phase=consume brev= checker_sha=); giudice committato
# PRE-run (A-BG53) con judge_sha risolvibile; ledger APPEND-only con VOID
# in-band (KG-89-1); done PER-ATTEMPT (A-SK54); identity v2 + srv_pid +
# srv_boot_epoch dal PROCESSO (ps lstart, A-BG55) + preflight=ok anche sul
# cammino pulito (A-BG55); pid-echo prima request ASSERITA post-wait_up
# (A-BG51/A-PP53); failpath ledgerati su OGNI ramo con escalation KILL
# (A-PP50/51, KS-PP-91-1).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$REPO/wp78-harness/measure-out"
OUTBIN="$HOME/Claude/php-rust-output/release"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
ORACLE="/opt/homebrew/opt/php/bin/php"
BATW="/Volumes/Extreme Pro/Claude/wp90-battery-out"
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

# --- APPEND-only ledger; VOID rows IN-BAND (A-BG43/A-BG46/KG-89-1) ----------
LEDGER="$OUT/m90.campaign.ledger"
ledger() { echo "attempt_epoch=$(date +%s) git=$GIT_REV campaign_sha=$CAMPAIGN_SHA $*" >> "$LEDGER"; }
ATT=0
fail() {
  echo "FAIL: $*"
  ledger "abort: $*"
  if [ "$ATT" = 0 ]; then
    ledger "attempt=0 phase=preflight verdict=VOID reason=$(echo "$*" | tr ' ' '-')"
  else
    ledger "attempt=$ATT verdict=VOID reason=$(echo "$*" | tr ' ' '-')"
  fi
  exit 1
}

# --- Pre-flight -------------------------------------------------------------
echo "== measure90 pre-flight git=$GIT_REV campaign_sha=$CAMPAIGN_SHA =="
DIRTY="$(git -C "$REPO" status --porcelain 2>/dev/null)"
[ -z "$DIRTY" ] || { echo "$DIRTY" | head -5; fail "tree not porcelain at campaign start"; }
pgrep -x php-server >/dev/null && fail "php-server already running"
UNCOMMITTED=$(git -C "$REPO" status --porcelain -- "wp78-harness/measure-out" 2>/dev/null | grep "m90\." || true)
[ -z "$UNCOMMITTED" ] || { echo "$UNCOMMITTED" | head -5; fail "uncommitted m90.* raws in measure-out (A-SK49/KG-88-1)"; }
# A-BG53: the judge of this campaign must be COMMITTED before the run —
# a generation judged by an uncommitted judge is unrecoverable (g2 m89).
JUDGE="$HERE/verdict90.sh"
[ -f "$JUDGE" ] || fail "verdict90.sh missing (A-BG53: judge committed PRE-run)"
JW=$(shasum -a 256 "$JUDGE" | cut -c1-16)
JC=$(git -C "$REPO" show "HEAD:php-rust/wp90-harness/verdict90.sh" 2>/dev/null | shasum -a 256 | cut -c1-16)
[ "$JW" = "$JC" ] || fail "verdict90.sh working copy ($JW) != committed blob at HEAD ($JC) — judge not committed PRE-run (A-BG53/KS-AH-91-2)"
# A-BG55: preflight OK is a ROW, not silence.
ledger "attempt=0 phase=preflight verdict=ok judge_sha=$JW"

ATT=1
while ls "$OUT"/m90.slope.w4.r1.a$ATT.* >/dev/null 2>&1; do ATT=$((ATT+1)); done
echo "attempt=$ATT (filenames carry it; prior generations stay committed)"
ledger "attempt=$ATT phase=start judge_sha=$JW"

# --- Battery consumption via the toothed checker only (A-SK46/A-SK50) -------
BOUT="$BATW/battery-90pre.out"
[ -f "$BOUT" ] || fail "battery-90pre.out missing — run the battery first"
BREV=$(sed -n 's/^== battery-90pre git=\([0-9a-f]*\) ==$/\1/p' "$BOUT" | head -1)
CHECKER="$REPO/wp83-harness/battery-equivalence.sh"
bash "$CHECKER" --same-rev "$BOUT" "$BREV" \
  || fail "same-rev consumption REFUSED (A-SK46/A-SK50/KS-SK-89-1)"
# A-AH57 (Council WP-91, Hejlsberg): the consumption is LEDGERED — the
# SAME-REV verdict no longer lives only in campaign stdout.
CKSHA=$(shasum -a 256 "$CHECKER" | cut -c1-16)
ledger "attempt=$ATT phase=consume brev=$BREV checker=battery-equivalence.sh checker_sha=$CKSHA esito=LEGAL"
DONE="$BATW/.done"
MTX_NAME=$(sed -n 's/.* matrix=\([^ ]*\) .*/\1/p' "$DONE" | head -1)
MTX="$REPO/wp78-harness/matrix-archive/$MTX_NAME"
mtx_hash() { tr -d '\0' < "$MTX" | sed -n "s/^bin\[$1\] sha256\[0:16\]=\([0-9a-f]*\).*/\1/p"; }

# --- Build the mem-census arm, enforce identity -----------------------------
( cd "$REPO" && cargo build --release -p php-server --features mem-census ) \
  > "$OUT/m90.build.mem-census.a$ATT.log" 2>&1 || fail "build mem-census"
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

echo "MEASURED ex-ante: slope-peak W in {4,8,12,16} R=5 con scala pboot/pwork/self/peak | cal 1 req R=2/side | sweep-pd0 dt {1,5,20} x {afirst,bfirst} | sweep-pdbig dt {1,5} x {afirst,bfirst} pd=1000"
echo "PREDICTIONS ex-ante: P-PEAK90 commit==peak_commit OVUNQUE | P-LADDER monotonia+additivita' δ=0,15 | P-SELF thr distinti, split REFUSED se heap collidono (A-DL31) | P-CLAMP-PD pd1000⇒clamped==0 e purged==0 per-request, pd0⇒clamped⇒purged>0 | P-DT20 net==cal AL BYTE | soglie: δ=0,15 coverage=0,9 robustezza=0,8"

# --- A-PP51: gone-verifier with KILL escalation -----------------------------
ensure_gone() { # -> echoes yes|killed9|no
  local gone=no i p
  for i in 1 2 3 4 5 6 7 8 9 10; do
    pgrep -x php-server >/dev/null || { gone=yes; break; }
    sleep 0.5
  done
  if [ "$gone" = no ]; then
    for p in $(pgrep -x php-server 2>/dev/null); do kill -KILL "$p" 2>/dev/null; done
    for i in 1 2 3 4; do
      pgrep -x php-server >/dev/null || { gone=killed9; break; }
      sleep 0.5
    done
  fi
  echo "$gone"
}

wait_up() {
  local up=0
  for _ in $(seq 1 100); do
    curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/__reqns" && { up=1; break; }
    sleep 0.1
  done
  [ $up = 1 ]
}

kill_server_tree() { # <wrapper-pid>
  local wpid="$1" child
  child=$(pgrep -P "$wpid" -x php-server 2>/dev/null || true)
  [ -n "$child" ] && kill -TERM $child 2>/dev/null
  kill -TERM "$wpid" 2>/dev/null
}

subshell_failpath() { # <dpid> <label>
  local dpid="$1" label="$2" gone
  kill_server_tree "$dpid"
  gone=$(ensure_gone)
  ledger "attempt=$ATT phase=$label failpath=assert_single_server server_gone=$gone"
  exit 1
}
teardown_fail() { # <srv> <dpid> <label> <msg...>
  local srv="$1" dpid="$2" label="$3"; shift 3
  kill -TERM "$srv" 2>/dev/null
  kill_server_tree "$dpid"
  local gone
  gone=$(ensure_gone)
  ledger "attempt=$ATT phase=$label failpath=teardown server_gone=$gone"
  fail "$*"
}
wait_up_fail() { # <dpid> <label>  (A-PP50/KS-PP-91-1)
  local dpid="$1" label="$2" gone
  kill_server_tree "$dpid"
  gone=$(ensure_gone)
  ledger "attempt=$ATT phase=$label failpath=wait_up server_gone=$gone"
  fail "m90.$label server not up (A-PP50)"
}

# A-BG51/A-PP53: pid-echo — first ASSERTED request, post-wait_up,
# pre-workload; tier0 excluded by NAME (never exercised here).
assert_http_pid() { # <label> <srv>
  local hp
  hp=$(curl -s -o /dev/null -m 5 -D - "http://127.0.0.1:$PORT/__reqns" | tr -d '\r' | awk 'tolower($1)=="x-phpr-pid:"{print $2; exit}')
  [ "$hp" = "$2" ] || teardown_fail "$2" "$DPID" "$1" "m90.$1 x-phpr-pid='$hp' != srv=$2 (A-BG51 pid-echo)"
}

assert_single_server() { # <label> <owner-pid> -> echoes the server pid
  local label="$1" owner="$2" pids srv
  pids=$(pgrep -x php-server || true)
  [ -n "$pids" ] || { fail "m90.$label no php-server process after up (anti-orphan a)" 1>&2; }
  [ "$(echo "$pids" | grep -c .)" = 1 ] || { echo "$pids" 1>&2; fail "m90.$label MULTIPLE php-server processes (anti-orphan b)" 1>&2; }
  srv="$pids"
  if [ "$srv" != "$owner" ] && [ "$(ps -o ppid= -p "$srv" | tr -d ' ')" != "$owner" ]; then
    fail "m90.$label the live php-server ($srv) is not ours ($owner) — orphan serving (anti-orphan b)" 1>&2
  fi
  local lowner
  lowner=$(lsof -nP -iTCP:$PORT -sTCP:LISTEN -t 2>/dev/null | sort -u)
  [ "$lowner" = "$srv" ] || { echo "listen-owner=$lowner srv=$srv" 1>&2; fail "m90.$label port $PORT LISTEN-owner is not our server (A-PP43/KS-PP-89-4)" 1>&2; }
  echo "$srv"
}
assert_server_gone() { # <label>
  local i
  for i in 1 2 3 4 5 6 7 8 9 10; do
    pgrep -x php-server >/dev/null || return 0
    sleep 0.5
  done
  fail "m90.$1 php-server still alive after teardown (anti-orphan c)"
}

# A-BG55 (Council WP-91, Gregg): srv_boot_epoch from the PROCESS
# (ps lstart), never the harness wall-clock — the boot<=epoch judgment
# stops being quasi-vacuous.
proc_boot_epoch() { # <pid> -> epoch
  local ls
  ls=$(ps -o lstart= -p "$1" 2>/dev/null | tr -s ' ' | sed 's/^ //;s/ $//')
  [ -n "$ls" ] || { echo ""; return 1; }
  date -j -f "%a %b %d %T %Y" "$ls" +%s 2>/dev/null
}

identity_row() { # <MC> <phase> <w> <nreq> <server_exit> [extra]
  echo "mem_hash=$MEM_HASH git=$GIT_REV campaign_sha=$CAMPAIGN_SHA arm=mem-census phase=$2 attempt=$ATT w=$3 nreq=$4 envtag=${ENVTAG:-none} srv_pid=$SRV srv_boot_epoch=$BOOT_EPOCH boot_src=ps-lstart server_exit=$5 seq=$SEQ epoch=$(date +%s)${6:+ $6}" >> "$1"
}

# run_slope <label> <workers> <nreq> — SLOPE-PEAK with the phase ladder
run_slope() {
  local label="$1" workers="$2" nreq="$3"
  SEQ=$((SEQ+1))
  local MC="$OUT/m90.$label.a$ATT.memcensus"
  local LOG="$OUT/m90.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers "$workers" --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || wait_up_fail "$DPID" "$label"
  local SRV BOOT_EPOCH
  SRV=$(assert_single_server "$label" "$DPID") || subshell_failpath "$DPID" "$label"
  BOOT_EPOCH=$(proc_boot_epoch "$SRV")
  [ -n "$BOOT_EPOCH" ] || teardown_fail "$SRV" "$DPID" "$label" "m90.$label ps-lstart boot epoch unreadable (A-BG55)"
  assert_http_pid "$label" "$SRV"
  # ladder checkpoint 1: post-asserts, pre-workload
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__census_pboot" || teardown_fail "$SRV" "$DPID" "$label" "m90.$label __census_pboot failed"
  local ok=0 i=0 B
  B=$(mktemp)
  while [ $i -lt "$nreq" ]; do
    curl -s -m 10 -o "$B" "http://127.0.0.1:$PORT/hello.php"
    [ "$(cat "$B")" = "$WANTH" ] && ok=$((ok+1))
    i=$((i+1))
  done
  rm -f "$B"
  [ "$ok" = "$nreq" ] || { teardown_fail "$SRV" "$DPID" "$label" "m90.$label bodies $ok/$nreq != oracle"; }
  # ladder checkpoint 2: the PEAK (workers alive, workload done; the
  # sequential curl above guarantees zero outstanding — quiescence
  # declared, KS-MS-91-2)
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__census_pwork" || teardown_fail "$SRV" "$DPID" "$label" "m90.$label __census_pwork failed"
  # per-worker SELF census: W sequential calls (round-robin; the judge
  # pins DISTINCT thr and judges heap-pointer collisions per A-DL31)
  local s=0 SB
  SB=$(mktemp)
  while [ $s -lt "$workers" ]; do
    curl -s -m 5 -o "$SB" "http://127.0.0.1:$PORT/__census_self" || teardown_fail "$SRV" "$DPID" "$label" "m90.$label __census_self #$s failed"
    grep -q "^census-self thr=" "$SB" || teardown_fail "$SRV" "$DPID" "$label" "m90.$label __census_self #$s bad body"
    s=$((s+1))
  done
  rm -f "$SB"
  # ladder checkpoint 3: peak proper (post-self — commit monotone)
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/__census" || teardown_fail "$SRV" "$DPID" "$label" "m90.$label __census failed"
  sleep 2
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m90.$label panic in server log (KH88-3)"
  identity_row "$MC" "$label" "$workers" "$nreq" "$rc"
  ledger "attempt=$ATT phase=$label raw=m90.$label.a$ATT.memcensus esito=ok nreq=$nreq"
}

# run_pad <label> <fixture> <want> — W=1 single-request calibration
run_pad() {
  local label="$1" fixture="$2" want="$3"
  SEQ=$((SEQ+1))
  local MC="$OUT/m90.$label.a$ATT.memcensus"
  local LOG="$OUT/m90.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 MIMALLOC_PURGE_DELAY=0 PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers 1 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || wait_up_fail "$DPID" "$label"
  local SRV BOOT_EPOCH
  SRV=$(assert_single_server "$label" "$DPID") || subshell_failpath "$DPID" "$label"
  BOOT_EPOCH=$(proc_boot_epoch "$SRV")
  [ -n "$BOOT_EPOCH" ] || teardown_fail "$SRV" "$DPID" "$label" "m90.$label ps-lstart boot epoch unreadable (A-BG55)"
  assert_http_pid "$label" "$SRV"
  local B; B=$(mktemp)
  curl -s -m 10 -o "$B" "http://127.0.0.1:$PORT/$fixture"
  if [ "$(cat "$B")" != "$want" ]; then teardown_fail "$SRV" "$DPID" "$label" "m90.$label body != oracle"; fi
  rm -f "$B"
  sleep 1
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m90.$label panic in server log (KH88-3)"
  identity_row "$MC" "$label" 1 1 "$rc" "fixture=$fixture"
  ledger "attempt=$ATT phase=$label raw=m90.$label.a$ATT.memcensus esito=ok"
}

# run_sweep <label> <dt_ms> <first:a|b> <pd> — W=2 pair, stagger; pd = the
# MIMALLOC_PURGE_DELAY arm (0 = baseline, 1000 = A-BB63 discriminant with
# read-back ord 15). PHPR_MI_COLLECT_REQ=1: per-request dumps carry the
# cumulative purged per request-window (A-DL50 operative form).
run_sweep() {
  local label="$1" dt="$2" first="$3" pd="$4"
  SEQ=$((SEQ+1))
  local MC="$OUT/m90.$label.a$ATT.memcensus"
  local LOG="$OUT/m90.$label.a$ATT.log"
  [ -e "$MC" ] && fail "raw $MC already exists — name reuse refused (KG-88-1)"
  : > "$MC"
  head_unmoved
  PHPR_MEM_CENSUS="$MC" PHPR_MI_COLLECT_EXIT=1 PHPR_MI_COLLECT_REQ=1 \
    MIMALLOC_PURGE_DELAY="$pd" PHPR_CAMPAIGN_SEQ=$SEQ \
    /usr/bin/time -l "$OUTBIN/php-server" --axum --workers 2 --port $PORT -t "$FIXDIR" \
    > /dev/null 2> "$LOG" &
  local DPID=$!
  wait_up || wait_up_fail "$DPID" "$label"
  local SRV BOOT_EPOCH
  SRV=$(assert_single_server "$label" "$DPID") || subshell_failpath "$DPID" "$label"
  BOOT_EPOCH=$(proc_boot_epoch "$SRV")
  [ -n "$BOOT_EPOCH" ] || teardown_fail "$SRV" "$DPID" "$label" "m90.$label ps-lstart boot epoch unreadable (A-BG55)"
  assert_http_pid "$label" "$SRV"
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
  if [ "$(cat "$BA")" != "$WF" ]; then teardown_fail "$SRV" "$DPID" "$label" "m90.$label $FF body != oracle"; fi
  if [ "$(cat "$BB")" != "$WS" ]; then teardown_fail "$SRV" "$DPID" "$label" "m90.$label $SS body != oracle"; fi
  rm -f "$BA" "$BB"
  sleep 1
  kill -TERM "$SRV" 2>/dev/null
  wait "$DPID" 2>/dev/null
  local rc=$?
  assert_server_gone "$label"
  grep -qE "panicked|aborting" "$LOG" && fail "m90.$label panic in server log (KH88-3)"
  identity_row "$MC" "$label" 2 2 "$rc" "fixture=$FF+$SS dt_ms=$dt first=$first pd=$pd"
  ledger "attempt=$ATT phase=$label raw=m90.$label.a$ATT.memcensus esito=ok dt_ms=$dt first=$first pd=$pd"
}

# --- Phase SLOPE-PEAK (ladder + self, W in {4,8,12,16}, R=5) ----------------
echo "== Phase SLOPE-PEAK (ladder pboot/pwork/self/peak) =="
ENVTAG=none
for w in 4 8 12 16; do
  for r in 1 2 3 4 5; do
    run_slope "slope.w$w.r$r" "$w" $((100*w))
  done
done

# --- Phase CAL ---------------------------------------------------------------
echo "== Phase CAL =="
for r in 1 2; do
  run_pad "cala.r$r" pad87a.php "$WANTA"
  run_pad "calb.r$r" pad87b.php "$WANTB"
done

# --- Phase SWEEP-PD0 (baseline clamped regime + dt20 control) ----------------
echo "== Phase SWEEP-PD0 (dt in {1,5,20} x {afirst,bfirst}, pd=0) =="
for dt in 1 5 20; do
  run_sweep "sweep.dt$dt.afirst.pd0" "$dt" a 0
  run_sweep "sweep.dt$dt.bfirst.pd0" "$dt" b 0
done

# --- Phase SWEEP-PDBIG (A-BB63: purge_delay >= finestra, read-back ord 15) ---
echo "== Phase SWEEP-PDBIG (dt in {1,5} x {afirst,bfirst}, pd=1000) =="
for dt in 1 5; do
  run_sweep "sweep.dt$dt.afirst.pd1000" "$dt" a 1000
  run_sweep "sweep.dt$dt.bfirst.pd1000" "$dt" b 1000
done

head_unmoved
ledger "attempt=$ATT phase=done esito=ok"
ADONE="$OUT/m90.a$ATT.done"
[ -e "$ADONE" ] && fail "m90.a$ATT.done already exists — name reuse refused (A-SK54/KG-88-1)"
echo "campaign=measure90 git=$GIT_REV attempt=$ATT mem_hash=$MEM_HASH campaign_sha=$CAMPAIGN_SHA judge_sha=$JW" > "$ADONE"
cp "$ADONE" "$OUT/m90.done"
echo "== measure90 DONE (attempt=$ATT, raws under $OUT/m90.*) =="
