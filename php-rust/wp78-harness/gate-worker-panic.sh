#!/bin/bash
# gate-worker-panic.sh — A-PP9/KS-PP-6 (Council WP-79 S-78.1.2): panic policy
# is FAIL-FAST and it must be PROVEN on the real binary, not asserted in prose.
#
# Protocol:
#   1. Build union php-server (axum-server feature), record sha256[0:16].
#   2. POSITIVE control: server with PHPR_TEST_WORKER_PANIC=1 serves a normal
#      request correctly (the pool works before we kill it).
#   3. TRIGGER: GET /__phpr_panic → the armed worker panics → the process MUST
#      abort (SIGABRT) and the log MUST carry the A-PP9 fail-fast line.
#      A server that survives = silently dead worker = FAIL (KS-PP-6).
#   4. NEGATIVE control: same request UNARMED (no env var) is an ordinary
#      docroot lookup (body == file content), server stays alive; SIGTERM
#      teardown must log the KL-78-4 join line ("all N workers joined").
#
# S-79.0.5 hardening (Council WP-80):
#   - A-SK4: exit code != 134 (SIGABRT) is a FAIL, never a WARN; the join
#     line pins N ("all 2 workers joined" — "all 0 workers joined" passed
#     the old grep).
#   - A-PP4 Phase C: a panic in the DISPATCHER (axum task) must abort too —
#     tokio used to swallow it (dead task, hung request, live process = the
#     "silently dead" KS-PP-6 bans, other side). Distinct magic path
#     /__phpr_panic_dispatcher, same env arm; unarmed negative included.
# Exit: 0 = PASS, 1 = FAIL, 2 = harness error.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
BIN="${BIN:-$HOME/Claude/php-rust-output/release/php-server}"
OUTDIR="$HERE/gate-axum/out"
mkdir -p "$OUTDIR"
PORT="${PORT:-8198}"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
VERDICT="$OUTDIR/worker_panic.verdict"
[ -f "$VERDICT" ] && mv "$VERDICT" "$VERDICT.superseded-$(date +%Y%m%d%H%M%S)"
FAILS=0

echo "== build -p php-server --features axum-server =="
( cd "$REPO" && cargo build --release -p php-server --features axum-server ) \
  > "$OUTDIR/panic-build.log" 2>&1
if [ $? -ne 0 ]; then
  echo "FAIL: build failed (see $OUTDIR/panic-build.log)"
  echo "FAIL build [git $GIT_REV]" > "$VERDICT"; exit 2
fi
HASH=$(shasum -a 256 "$BIN" | cut -c1-16)
echo "php-server sha256[0:16]=$HASH"

DOCROOT="$(mktemp -d)"
trap 'rm -rf "$DOCROOT"' EXIT
printf '<?php echo "alive\n";' > "$DOCROOT/hello.php"
printf 'inert\n' > "$DOCROOT/__phpr_panic"
printf 'inert-dispatcher\n' > "$DOCROOT/__phpr_panic_dispatcher"

pkill -f "php-server --axum --port $PORT" 2>/dev/null && sleep 1

wait_up() { # wait_up <url>
  for _ in $(seq 1 100); do
    if curl -s -o /dev/null -m 1 "$1"; then return 0; fi
    sleep 0.1
  done
  return 1
}

# --- Phase A: ARMED — panic must abort the process ------------------------
PHPR_TEST_WORKER_PANIC=1 "$BIN" --axum --port "$PORT" --workers 2 -t "$DOCROOT" \
  > /dev/null 2> "$OUTDIR/panic-armed.log" &
SRV=$!
if ! wait_up "http://127.0.0.1:$PORT/hello.php"; then
  echo "FAIL: armed server did not come up"; kill "$SRV" 2>/dev/null
  echo "FAIL server-up [git $GIT_REV]" > "$VERDICT"; exit 2
fi
BODY=$(curl -s -m 5 "http://127.0.0.1:$PORT/hello.php")
if [ "$BODY" = "alive" ]; then
  echo "OK  positive control: pool serves requests while armed"
else
  echo "FAIL positive control: hello body '$BODY' != 'alive'"; FAILS=$((FAILS+1))
fi

curl -s -o /dev/null -m 5 "http://127.0.0.1:$PORT/__phpr_panic" || true
dead=1
for _ in $(seq 1 100); do
  if ! kill -0 "$SRV" 2>/dev/null; then dead=0; break; fi
  sleep 0.1
done
if [ "$dead" = 0 ]; then
  wait "$SRV"; RC=$?
  if [ "$RC" -eq 134 ]; then
    echo "OK  armed panic aborted the process (exit=134 SIGABRT)"
  else
    # A-SK4 (Council WP-80): a non-SIGABRT death is NOT the fail-fast
    # contract — the old WARN downgrade let any exit pass on the log line.
    echo "FAIL: exit code $RC != 134 (SIGABRT) — not the abort contract (A-SK4)"
    FAILS=$((FAILS+1))
  fi
else
  echo "FAIL: process SURVIVED a worker panic — silently dead worker (KS-PP-6)"
  kill -9 "$SRV" 2>/dev/null; FAILS=$((FAILS+1))
fi
if grep -q "fail-fast, KS-PP-6" "$OUTDIR/panic-armed.log"; then
  echo "OK  fail-fast abort line present in log"
else
  echo "FAIL: fail-fast line missing from panic-armed.log"; FAILS=$((FAILS+1))
fi

# --- Phase B: UNARMED — trigger path is inert, teardown joins -------------
"$BIN" --axum --port "$PORT" --workers 2 -t "$DOCROOT" \
  > /dev/null 2> "$OUTDIR/panic-unarmed.log" &
SRV=$!
if ! wait_up "http://127.0.0.1:$PORT/hello.php"; then
  echo "FAIL: unarmed server did not come up"; kill "$SRV" 2>/dev/null
  echo "FAIL server-up-B [git $GIT_REV]" > "$VERDICT"; exit 2
fi
BODY=$(curl -s -m 5 "http://127.0.0.1:$PORT/__phpr_panic")
if [ "$BODY" = "inert" ]; then
  echo "OK  negative control: unarmed worker trigger is an ordinary lookup"
else
  echo "FAIL negative control: body '$BODY' != 'inert'"; FAILS=$((FAILS+1))
fi
BODY=$(curl -s -m 5 "http://127.0.0.1:$PORT/__phpr_panic_dispatcher")
if [ "$BODY" = "inert-dispatcher" ]; then
  echo "OK  negative control: unarmed dispatcher trigger is an ordinary lookup"
else
  echo "FAIL negative control: dispatcher body '$BODY' != 'inert-dispatcher'"; FAILS=$((FAILS+1))
fi
if kill -0 "$SRV" 2>/dev/null; then
  echo "OK  negative control: server alive after unarmed triggers"
else
  echo "FAIL negative control: server died unarmed"; FAILS=$((FAILS+1))
fi
kill -TERM "$SRV" 2>/dev/null
wait "$SRV" 2>/dev/null
# A-SK4: the join line pins N — "all 0 workers joined" satisfied the old grep.
if grep -q "all 2 workers joined" "$OUTDIR/panic-unarmed.log"; then
  echo "OK  KL-78-4: SIGTERM teardown logged 'all 2 workers joined' (N pinned)"
else
  echo "FAIL KL-78-4: pinned join line 'all 2 workers joined' missing after SIGTERM"
  FAILS=$((FAILS+1))
fi

# --- Phase C (A-PP4, Council WP-80): DISPATCHER panic must abort too --------
PHPR_TEST_WORKER_PANIC=1 "$BIN" --axum --port "$PORT" --workers 2 -t "$DOCROOT" \
  > /dev/null 2> "$OUTDIR/panic-dispatcher.log" &
SRV=$!
if ! wait_up "http://127.0.0.1:$PORT/hello.php"; then
  echo "FAIL: phase-C server did not come up"; kill "$SRV" 2>/dev/null
  echo "FAIL server-up-C [git $GIT_REV]" > "$VERDICT"; exit 2
fi
curl -s -o /dev/null -m 5 "http://127.0.0.1:$PORT/__phpr_panic_dispatcher" || true
dead=1
for _ in $(seq 1 100); do
  if ! kill -0 "$SRV" 2>/dev/null; then dead=0; break; fi
  sleep 0.1
done
if [ "$dead" = 0 ]; then
  wait "$SRV"; RC=$?
  if [ "$RC" -eq 134 ]; then
    echo "OK  dispatcher panic aborted the process (exit=134, A-PP4)"
  else
    echo "FAIL: dispatcher panic exit $RC != 134 (A-PP4/A-SK4)"; FAILS=$((FAILS+1))
  fi
else
  echo "FAIL: process SURVIVED a dispatcher panic — tokio swallowed it (A-PP4/KS-PP-6)"
  kill -9 "$SRV" 2>/dev/null; FAILS=$((FAILS+1))
fi
if grep -q "fail-fast, KS-PP-6" "$OUTDIR/panic-dispatcher.log"; then
  echo "OK  fail-fast abort line present in dispatcher log"
else
  echo "FAIL: fail-fast line missing from panic-dispatcher.log"; FAILS=$((FAILS+1))
fi

if [ "$FAILS" = 0 ]; then
  echo "PASS fails=0 bin=$HASH git=$GIT_REV $(date +%F_%H:%M:%S) (gate-worker-panic.sh)" > "$VERDICT"
  echo "== gate-worker-panic PASS (bin=$HASH git=$GIT_REV) =="; exit 0
else
  echo "FAIL fails=$FAILS bin=$HASH git=$GIT_REV" > "$VERDICT"
  echo "== gate-worker-panic FAIL($FAILS) bin=$HASH git=$GIT_REV =="; exit 1
fi
