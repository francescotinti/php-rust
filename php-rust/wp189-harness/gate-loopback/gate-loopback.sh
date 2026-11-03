#!/bin/bash
# gate-loopback.sh — S-189 FETTA 0: deterministic tooth for the blocking-loopback stall of the
# axum worker pool (criterio: wp189-harness/s189-criterio-loopback.md; council COUNCIL_WP189_REVIEWS.md).
#
# One request (loop.php) performs L = W+1 SEQUENTIAL curl loopbacks to pong.php on the same server.
#   old pin (per-worker queues, blind round-robin), W=3, L=4: the 3rd inner request lands on the
#     worker that is waiting for it -> ERR errno=28 after CURLOPT_TIMEOUT 3 s; teardown map {0:2,1:2,2:1}
#   new (shared FIFO queue), W=3, L=4: 4/4 OK, orphan=0
#   W=1 (both binaries) L=2: ERR28 ERR28 — fidelity with php -S at 1 process (kept by name)
#   oracle php -S with PHP_CLI_SERVER_WORKERS=2 (= 3 acceptors: the master accepts too) L=4: 4/4 OK
#   oracle php -S 1 process L=2: ERR28 ERR28
#   negative: old pin W=3 L=1: OK (proves fixture + curl facade)
# Every arm: fresh server per repetition, REPS identical outcomes (any mismatch = FAIL, never 2/3),
# request log must hold exactly 1 + L loop/pong lines (extra line = repetition VOID, retried up to 3),
# port chosen by the kernel (never 8080/8199), readiness on a STATIC file (does not touch the pool),
# PID on port == spawned PID, SIGTERM then <=30 s wait (else SIGKILL + shutdown-hang), teardown rows
# tag=worker_dispatch read from stderr (printed on every non-mem-census build).
# Env: OLD_BIN NEW_BIN ORACLE PIN_ATTESO REPS ARMS. rc 0 = PASS, 1 = FAIL, 2 = usage/setup.
# Generato con assistenza AI (Claude Fable 5).
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
FIX="$HERE/fixtures"
OUT="$HERE/out"; mkdir -p "$OUT"
OLD_BIN="${OLD_BIN:-$HOME/Claude/php-rust-output/release/php-server}"
NEW_BIN="${NEW_BIN:-$HOME/Claude/phpr-target/dev-output/dev-release/php-server}"
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
PIN_ATTESO="${PIN_ATTESO:-97b58fd225ca7289}"
REPS="${REPS:-3}"
ARMS="${ARMS:-old3 new3 old1 new1 oracle3 oracle1 neg}"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo no-git)"
STAMP="$(date +%Y%m%d%H%M%S)"
VERDICT="$OUT/gate_loopback.verdict"
LOG="$OUT/gate_loopback.$STAMP.log"
find "$FIX" -name '._*' -delete 2>/dev/null
say(){ echo "$*" | tee -a "$LOG"; }

[ -x "$OLD_BIN" ] || { say "FAIL setup: OLD_BIN $OLD_BIN missing"; exit 2; }
[ -x "$NEW_BIN" ] || { say "FAIL setup: NEW_BIN $NEW_BIN missing"; exit 2; }
[ -x "$ORACLE" ] || { say "FAIL setup: ORACLE $ORACLE missing"; exit 2; }
OLD_HASH="$(shasum -a 256 "$OLD_BIN" | cut -c1-16)"
NEW_HASH="$(shasum -a 256 "$NEW_BIN" | cut -c1-16)"
if [ "$OLD_HASH" != "$PIN_ATTESO" ]; then
  say "VOID: OLD_BIN hash $OLD_HASH != PIN_ATTESO $PIN_ATTESO (KS-189-1: never skip)"; exit 2
fi
say "== gate-loopback S-189 git=$GIT_REV old=$OLD_HASH new=$NEW_HASH oracle=$("$ORACLE" -r 'echo PHP_VERSION;') reps=$REPS arms=[$ARMS] =="

free_port(){
  python3 - <<'PY'
import socket
s = socket.socket(); s.bind(("127.0.0.1", 0)); p = s.getsockname()[1]; s.close(); print(p)
PY
}

SRV=""
stop_server(){ # $1 = kind ; returns 0 clean, 1 hang
  local pid="$SRV" i
  [ -n "$pid" ] || return 0
  kill -TERM "$pid" 2>/dev/null
  for i in $(seq 1 300); do kill -0 "$pid" 2>/dev/null || { wait "$pid" 2>/dev/null; SRV=""; return 0; }; sleep 0.1; done
  kill -KILL "$pid" 2>/dev/null; wait "$pid" 2>/dev/null; SRV=""
  # php -S children (shared socket) die with the master; sweep anything left on the port
  return 1
}
cleanup(){ [ -n "$SRV" ] && kill -TERM "$SRV" 2>/dev/null; }
trap cleanup EXIT

# boot <kind:phpr|oracle> <bin> <W> <port> <errfile>
boot(){
  local kind="$1" bin="$2" w="$3" port="$4" err="$5" i up=0 lp
  if [ "$kind" = phpr ]; then
    ( cd "$FIX" && exec "$bin" --axum --workers "$w" --host 127.0.0.1 --port "$port" -t "$FIX" ) >"$err.out" 2>"$err" &
  else
    if [ "$w" -gt 1 ]; then
      ( cd "$FIX" && PHP_CLI_SERVER_WORKERS=$((w-1)) exec "$bin" -S "127.0.0.1:$port" -t "$FIX" ) >"$err.out" 2>"$err" &
    else
      ( cd "$FIX" && exec "$bin" -S "127.0.0.1:$port" -t "$FIX" ) >"$err.out" 2>"$err" &
    fi
  fi
  SRV=$!
  for i in $(seq 1 100); do
    if [ "$(curl -s -m 1 "http://127.0.0.1:$port/ready.txt")" = "ready" ]; then up=1; break; fi
    sleep 0.1
  done
  [ "$up" = 1 ] || { say "   boot failed ($kind w=$w port=$port): $(head -3 "$err")"; return 1; }
  lp="$(lsof -nP -tiTCP:"$port" -sTCP:LISTEN | tr '\n' ' ')"
  case " $lp " in *" $SRV "*) ;; *) say "   PID on port $port is [$lp], spawned $SRV: run NULLA (KS-SK-79.1)"; return 1;; esac
  return 0
}

# expected outcome string per arm: tokens per i: OK | ERR28
expect_arm(){
  case "$1" in
    old3)   echo "OK OK ERR28 OK";;
    new3)   echo "OK OK OK OK";;
    old1|new1|oracle1) echo "ERR28 ERR28";;
    oracle3) echo "OK OK OK OK";;
    neg)    echo "OK";;
  esac
}
arm_kind(){ case "$1" in oracle*) echo oracle;; *) echo phpr;; esac; }
arm_bin(){ case "$1" in old*|neg) echo "$OLD_BIN";; new*) echo "$NEW_BIN";; *) echo "$ORACLE";; esac; }
arm_w(){ case "$1" in *3|neg) echo 3;; *1) echo 1;; esac; }
arm_L(){ case "$1" in neg) echo 1;; *3) echo 4;; *1) echo 2;; esac; }
arm_map(){ # expected teardown map (phpr arms only); "SUM=n" means set exact + sum only
  case "$1" in
    old3) echo "0:2 1:2 2:1";;
    new3) echo "SUM=5";;
    old1) echo "0:3";;
    new1) echo "0:3";;
    neg)  echo "0:1 1:1 2:0";;
    *) echo "";;
  esac
}

FAILS=0
ARM_SUMMARY=""
for arm in $ARMS; do
  kind="$(arm_kind "$arm")"; bin="$(arm_bin "$arm")"; W="$(arm_w "$arm")"; L="$(arm_L "$arm")"
  EXP="$(expect_arm "$arm")"; EXPMAP="$(arm_map "$arm")"
  say "-- arm $arm kind=$kind W=$W L=$L expect=[$EXP] map=[$EXPMAP]"
  armfail=0; outcomes=""
  rep=1; tries=0
  while [ "$rep" -le "$REPS" ]; do
    tries=$((tries+1)); [ "$tries" -le $((REPS+3)) ] || { say "   too many VOID repetitions"; armfail=1; break; }
    PORT="$(free_port)"
    case "$PORT" in 8080|8199) continue;; esac
    ERR="$OUT/$arm.r$rep.t$tries.server.log"
    if ! boot "$kind" "$bin" "$W" "$PORT" "$ERR"; then stop_server; armfail=1; break; fi
    t_start=$(date +%s)
    BODY="$(curl -s -m 30 -w '\nHTTP=%{http_code}' "http://127.0.0.1:$PORT/loop.php?L=$L&port=$PORT")"
    t_wall=$(( $(date +%s) - t_start ))
    HTTP="$(printf '%s' "$BODY" | sed -n 's/^HTTP=//p')"
    FIXOUT="$(printf '%s' "$BODY" | sed '/^HTTP=/d')"
    if stop_server; then hang=0; else hang=1; say "   shutdown-hang (SIGKILL after 30 s)"; fi
    # request lines seen by the server ("[code]: METHOD target", stdout+stderr): exactly 1 loop;
    # pong lines = L minus the stalled ones (an orphan executed after its client timed out has no
    # handler left to log it: it is counted by the teardown map, not by the log); nothing else.
    nloop=$(awk '/\[[0-9][0-9][0-9]\]: [A-Z]+ \/loop\.php/{n++} END{print n+0}' "$ERR" "$ERR.out")
    npong=$(awk '/\[[0-9][0-9][0-9]\]: [A-Z]+ \/pong\.php/{n++} END{print n+0}' "$ERR" "$ERR.out")
    nextra=$(awk '/\[[0-9][0-9][0-9]\]: [A-Z]+ \// && !/\/loop\.php/ && !/\/pong\.php/ && !/\/ready\.txt/{n++} END{print n+0}' "$ERR" "$ERR.out")
    # the floor follows the stalls the FIXTURE actually reports (not the expectation): a stall on
    # an arm expected green must surface as a MISMATCH below, never as a VOID
    nerr_got=$(printf '%s\n' "$FIXOUT" | awk '/ ERR /{n++} END{print n+0}')
    pong_min=$((L - nerr_got))
    # fixture outcome tokens + ms band checks
    got=""; bandfail=0; i=0
    while IFS= read -r line; do
      [ -n "$line" ] || continue
      i=$((i+1))
      ms=$(printf '%s' "$line" | sed -n 's/.* ms=\([0-9]*\).*/\1/p')
      case "$line" in
        *" OK "*)   tok=OK;  [ "${ms:-99999}" -lt 1000 ] || bandfail=1;;
        *"errno=28"*) tok=ERR28; { [ "${ms:-0}" -ge 2900 ] && [ "${ms:-0}" -le 3400 ]; } || bandfail=1;;
        *) tok="ERR$(printf '%s' "$line" | sed -n 's/.*errno=\([0-9]*\).*/\1/p')";;
      esac
      got="${got:+$got }$tok"
    done <<EOF
$FIXOUT
EOF
    # teardown map (phpr only)
    mapgot=""; orphan="n/a"; sum=0; set_ok=1
    if [ "$kind" = phpr ]; then
      # rows are printed in join order (not thr order): normalise by thr before comparing
      mapgot="$(awk '/tag=worker_dispatch/{for(k=1;k<=NF;k++){if($k~/^thr=/)t=substr($k,5); if($k~/^count=/)c=substr($k,7)}; printf "%s:%s\n", t, c}' "$ERR" | sort -n | tr '\n' ' ' | sed 's/ $//')"
      sum=$(awk '/tag=worker_dispatch/{for(k=1;k<=NF;k++) if($k~/^count=/) s+=substr($k,7)} END{print s+0}' "$ERR")
      o="$(awk '/tag=worker_dispatch/{for(k=1;k<=NF;k++) if($k~/^orphan=/) s+=substr($k,8); seen=1} END{if(seen) print s+0}' "$ERR")"
      orphanrows=$(awk '/orphan=/{n++} END{print n+0}' "$ERR")
      [ "$orphanrows" -gt 0 ] && orphan="$o"
      # set must be exactly {0..W-1}
      thrs="$(printf '%s\n' $mapgot | sed 's/:.*//' | sort -n | tr '\n' ' ' | sed 's/ $//')"
      want="$(seq 0 $((W-1)) | tr '\n' ' ' | sed 's/ $//')"
      [ "$thrs" = "$want" ] || set_ok=0
    fi
    # judge this repetition
    void=0; why=""
    if [ "$nloop" != 1 ] || [ "$npong" -lt "$pong_min" ] || [ "$npong" -gt "$L" ] || [ "$nextra" != 0 ]; then void=1; why="log loop=$nloop pong=$npong extra=$nextra (expected 1/[$pong_min..$L]/0)"; fi
    if [ "$void" = 1 ]; then
      say "   rep $rep try $tries: VOID — $why"; continue
    fi
    ok=1
    [ "$HTTP" = 200 ] || { ok=0; why="$why http=$HTTP"; }
    [ "$got" = "$EXP" ] || { ok=0; why="$why got=[$got]"; }
    [ "$bandfail" = 0 ] || { ok=0; why="$why ms-band"; }
    if [ "$kind" = phpr ]; then
      case "$EXPMAP" in
        SUM=*) { [ "$sum" = "${EXPMAP#SUM=}" ] && [ "$set_ok" = 1 ]; } || { ok=0; why="$why map=[$mapgot] sum=$sum"; }
               [ "$orphan" = "n/a" ] || [ "$orphan" = 0 ] || { ok=0; why="$why orphan=$orphan"; };;
        *)     [ "$mapgot" = "$EXPMAP" ] || { ok=0; why="$why map=[$mapgot]"; };;
      esac
    fi
    [ "$hang" = 0 ] || why="$why shutdown-hang"
    printf '%s\n' "arm=$arm rep=$rep try=$tries port=$PORT http=$HTTP wall=${t_wall}s got=[$got] map=[$mapgot] sum=$sum orphan=$orphan log=$nloop/$npong/$nextra hang=$hang ok=$ok" >> "$OUT/$arm.evidence"
    printf '%s\n' "$FIXOUT" >> "$OUT/$arm.evidence"
    say "   rep $rep: http=$HTTP got=[$got] map=[$mapgot] sum=$sum orphan=$orphan wall=${t_wall}s → $([ "$ok" = 1 ] && echo ok || echo "MISMATCH:$why")"
    [ "$ok" = 1 ] || armfail=1
    rep=$((rep+1))
  done
  if [ "$armfail" = 0 ]; then say "   ARM $arm: PASS ($REPS/$REPS identical, as expected)"; ARM_SUMMARY="$ARM_SUMMARY $arm=PASS"
  else say "   ARM $arm: FAIL"; FAILS=$((FAILS+1)); ARM_SUMMARY="$ARM_SUMMARY $arm=FAIL"; fi
done

if [ "$FAILS" = 0 ]; then
  echo "PASS old=$OLD_HASH new=$NEW_HASH git=$GIT_REV $(date +%F_%H:%M:%S) arms=[$ARM_SUMMARY ] (gate-loopback.sh)" > "$VERDICT"
  say "== gate-loopback PASS old=$OLD_HASH new=$NEW_HASH git=$GIT_REV [$ARM_SUMMARY ] =="; exit 0
else
  echo "FAIL fails=$FAILS old=$OLD_HASH new=$NEW_HASH git=$GIT_REV $(date +%F_%H:%M:%S) arms=[$ARM_SUMMARY ] (gate-loopback.sh)" > "$VERDICT"
  say "== gate-loopback FAIL($FAILS) old=$OLD_HASH new=$NEW_HASH git=$GIT_REV [$ARM_SUMMARY ] =="; exit 1
fi
