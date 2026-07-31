#!/bin/bash
# gate-lever-fixtures.sh — S-81.0 (Council WP-82): the fixture teeth ARMED IN
# THE SAME COMMIT as the main-unit put (A-PP17/KS-SK-82-2). Runtime checks on
# fresh builds (php-server default = cli-server arm; phpr = one-shot arm),
# observability = the uc_log per-event channel (PHPR_UNIT_CACHE_LOG, union
# build — counters a census-only build could see would make these vacuous).
#
#   F-oneshot (three teeth, KS-SK-82-2/A-SK16):
#     t1 the log CHANNEL is alive on the one-shot arm (reqmark present —
#        "absent" can never read as "zero probes");
#     t2 main_probe events == 0 on the one-shot (A-TH14/F-oneshot);
#     t3 positive twin: main_probe == nreq on the server arm, same campaign.
#   F5 + put/hit accounting (KS-SK-82-3 pin main_hit per request):
#     N requests to one file ⇒ main_probe==N, main_put==1, main_hit==N-1.
#   F8c in COUNTER form (A-PP17/KS-PP-82-2) + F8b byte-identity:
#     link-fatal file, 2 requests ⇒ bodies byte-identical, main_put==0,
#     main_hit==0 for that path; fix ⇒ body "fixed", main_put==1.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
OUTBIN="$HOME/Claude/php-rust-output/release"
PORT="${PORT:-8391}"
FAILS=0

TMPD=$(mktemp -d)
SRVPID=""
cleanup() { [ -n "$SRVPID" ] && kill -TERM "$SRVPID" 2>/dev/null; rm -rf "$TMPD"; }
trap cleanup EXIT

count_ev() { # <log> <event> [path-substr] -> count of `unitcache <event> ...`
  local log="$1" ev="$2" sub="${3:-}"
  [ -f "$log" ] || { echo 0; return; }
  if [ -n "$sub" ]; then
    grep -c "^unitcache $ev .*$sub" "$log" || true
  else
    grep -c "^unitcache $ev " "$log" || true
  fi
}

echo "== build (release): php-server default (cli-server) + phpr =="
( cd "$REPO" && cargo build --release -p php-server -p php-cli ) > "$TMPD/build.log" 2>&1 || {
  echo "FAIL: build failed"; tail -20 "$TMPD/build.log"; exit 1; }

# --- F-oneshot: teeth 1+2 -----------------------------------------------------
ONELOG="$TMPD/oneshot.uclog"
echo '<?php echo "oneshot-ok";' > "$TMPD/oneshot.php"
BODY=$(PHPR_UNIT_CACHE_LOG="$ONELOG" "$OUTBIN/phpr" "$TMPD/oneshot.php")
if [ "$BODY" != "oneshot-ok" ]; then
  echo "FAIL: one-shot body '$BODY' != 'oneshot-ok'"; FAILS=$((FAILS+1))
fi
if [ "$(count_ev "$ONELOG" reqmark)" -lt 1 ]; then
  echo "FAIL: F-oneshot t1 — uc_log channel DEAD on one-shot (no reqmark): zero-probes claim would be vacuous (KS-SK-82-2)"
  FAILS=$((FAILS+1))
else
  echo "OK  F-oneshot t1: channel alive (reqmark present)"
fi
NP=$(count_ev "$ONELOG" main_probe)
if [ "$NP" -ne 0 ]; then
  echo "FAIL: F-oneshot t2 — one-shot arm probed the main $NP times (pin 0, A-TH14)"
  FAILS=$((FAILS+1))
else
  echo "OK  F-oneshot t2: main_probe == 0 on the one-shot arm (channel proven alive)"
fi

# --- server arm: F-oneshot t3 (positive twin) + F5 + put/hit accounting -------
DOCROOT="$TMPD/docroot"
mkdir -p "$DOCROOT"
echo '<?php echo "hello-lever";' > "$DOCROOT/hello.php"
SRVLOG="$TMPD/server.uclog"
PHPR_UNIT_CACHE_LOG="$SRVLOG" "$OUTBIN/php-server" --port "$PORT" -t "$DOCROOT" \
  > /dev/null 2> "$TMPD/server.log" &
SRVPID=$!
up=0
for _ in $(seq 1 100); do
  if curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php"; then up=1; break; fi
  sleep 0.1
done
if [ "$up" != 1 ]; then echo "FAIL: cli-server did not come up"; exit 1; fi
# the boot probe above counts: continue to N total requests on hello.php
N=5
for _ in $(seq 2 "$N"); do
  curl -s -m 5 -o /dev/null "http://127.0.0.1:$PORT/hello.php"
done
B1=$(curl -s -m 5 "http://127.0.0.1:$PORT/hello.php")   # request N+1, must be a HIT
if [ "$B1" != "hello-lever" ]; then
  echo "FAIL: server body on HIT '$B1' != 'hello-lever' (KS-DS-80-3 territory)"
  FAILS=$((FAILS+1))
fi
NREQ=$((N + 1))
np=$(count_ev "$SRVLOG" main_probe hello.php)
nput=$(count_ev "$SRVLOG" main_put hello.php)
nhit=$(count_ev "$SRVLOG" main_hit hello.php)
if [ "$np" -ne "$NREQ" ]; then
  echo "FAIL: F-oneshot t3 — server arm main_probe=$np != nreq=$NREQ (positive twin, A-SK16)"
  FAILS=$((FAILS+1))
else
  echo "OK  F-oneshot t3: server arm probes every request (main_probe == $NREQ)"
fi
if [ "$nput" -ne 1 ] || [ "$nhit" -ne $((NREQ - 1)) ]; then
  echo "FAIL: F5/accounting — main_put=$nput (pin 1), main_hit=$nhit (pin $((NREQ-1))) on hello.php"
  FAILS=$((FAILS+1))
else
  echo "OK  F5: main_put==1, main_hit==$nhit (== nreq-1) — the hit-counter MOVES (KG-79.A)"
fi

# --- F8c (counter form) + F8b byte-identity ------------------------------------
# Link fatal: enum implementing Serializable (the S-78.1.4 sweep example).
cat > "$DOCROOT/linkfatal.php" <<'EOF'
<?php
enum LF implements Serializable { case A; }
echo "never-reached";
EOF
BF1=$(curl -s -m 5 "http://127.0.0.1:$PORT/linkfatal.php")
BF2=$(curl -s -m 5 "http://127.0.0.1:$PORT/linkfatal.php")
case "$BF1" in
  *"Fatal error"*) : ;;
  *) echo "FAIL: F8c — first link-fatal body carries no 'Fatal error' banner: $(printf %s "$BF1" | head -c 120)"
     FAILS=$((FAILS+1)) ;;
esac
if [ "$BF1" != "$BF2" ]; then
  echo "FAIL: F8b — two link-fatal responses are NOT byte-identical (the fatal was published?)"
  FAILS=$((FAILS+1))
else
  echo "OK  F8b: link-fatal response byte-identical across two requests"
fi
nput=$(count_ev "$SRVLOG" main_put linkfatal.php)
nhit=$(count_ev "$SRVLOG" main_hit linkfatal.php)
np=$(count_ev "$SRVLOG" main_probe linkfatal.php)
if [ "$nput" -ne 0 ] || [ "$nhit" -ne 0 ] || [ "$np" -ne 2 ]; then
  echo "FAIL: F8c counters — main_put=$nput (pin 0), main_hit=$nhit (pin 0), main_probe=$np (pin 2) on link-fatal (A-PP17/KS-PP-82-2)"
  FAILS=$((FAILS+1))
else
  echo "OK  F8c: link-fatal NEVER published (put==0, hit==0, probe==2 — counter form)"
fi
echo '<?php echo "fixed";' > "$DOCROOT/linkfatal.php"
BFIX=$(curl -s -m 5 "http://127.0.0.1:$PORT/linkfatal.php")
nput=$(count_ev "$SRVLOG" main_put linkfatal.php)
if [ "$BFIX" != "fixed" ] || [ "$nput" -ne 1 ]; then
  echo "FAIL: F8c fix — body '$BFIX' (want 'fixed'), main_put=$nput (pin 1 after fix)"
  FAILS=$((FAILS+1))
else
  echo "OK  F8c: fix publishes (body 'fixed', main_put==1)"
fi

kill -TERM "$SRVPID" 2>/dev/null
wait "$SRVPID" 2>/dev/null
SRVPID=""

if [ "$FAILS" = 0 ]; then
  echo "== GATE-LEVER-FIXTURES PASS (F-oneshot 3 teeth + F5 + F8b/F8c counters) [git $GIT_REV] =="
  exit 0
else
  echo "== GATE-LEVER-FIXTURES FAIL($FAILS) [git $GIT_REV] =="
  exit 1
fi
