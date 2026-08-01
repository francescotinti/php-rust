#!/bin/bash
# dl28-supplement.sh — S-85.0 (precedent: wp82 phaseR-supplement): the W=2
# canary run RE-EXECUTED with the ONE-DISPATCH-PER-WORKER protocol. The
# campaign's m85.dl28 raw is VOID by protocol break (my memrun dispatched
# hello TWICE — up-probe + oracle check — so both workers' first request
# was hello and the pad landed as thr0's SECOND main): superseded, kept in
# place, NEVER rm'd (KS-AH-83-2). Fix: the up-probe's first SUCCESS body
# IS request 1 (captured and oracle-compared, no second hello dispatch);
# the single pad request is dispatch 2 -> the other worker (round-robin
# ADVISORY, A-PP32 — the verdict attributes BY PATH+thr in-band,
# KS-PP-86-3, and fail-closes if both fixtures land on one thread).
# Same rev, same mem-census binary as the campaign (ENFORCED).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$REPO/wp78-harness/measure-out"
OUTBIN="$HOME/Claude/php-rust-output/release"
ORACLE="/opt/homebrew/opt/php/bin/php"
FIXDIR="$REPO/wp78-harness/gate-axum/fixtures"
export PHPR_CAMPAIGN_SCRIPT="$HERE/dl28-supplement.sh"   # A-AH30
FAILS=0
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD)"
fail() { echo "FAIL: $*"; FAILS=$((FAILS+1)); }

# ENFORCE: same mem-census binary as the campaign's CAL runs (the
# calibration NET_H/NET_P is binary-bound).
CAL_HASH=$(sed -n 's/^mem_hash=\([0-9a-f]*\) .*/\1/p' "$OUT/m85.cal-h.memcensus" | tail -1)
( cd "$REPO" && cargo build --release -p php-server --features mem-census ) \
  > "$OUT/m85s.build.mem-census.log" 2>&1 || { echo "FAIL: build mem-census"; exit 1; }
MEM_HASH=$(shasum -a 256 "$OUTBIN/php-server" | cut -c1-16)
[ "$MEM_HASH" = "$CAL_HASH" ] || { echo "FAIL: mem-census hash $MEM_HASH != campaign CAL $CAL_HASH — calibration not transferable"; exit 1; }
MTX=$(/bin/ls "$REPO/wp78-harness/matrix-archive"/feature-matrix.$GIT_REV.*.log 2>/dev/null | tail -1)
[ -n "$MTX" ] || { echo "FAIL: no archived matrix at $GIT_REV"; exit 1; }
WANT=$(tr -d '\0' < "$MTX" | sed -n "s/^bin\[mem-census\] sha256\[0:16\]=\([0-9a-f]*\).*/\1/p")
bash "$HERE/gate-binary-noprobe.sh" "$OUTBIN/php-server" "$WANT" || { echo "FAIL: KH86-1"; exit 1; }

MC="$OUT/m85.dl28s.memcensus"
: > "$MC"
PORT=8298
PHPR_MEM_CENSUS="$MC" "$OUTBIN/php-server" --axum --workers 2 --port $PORT -t "$FIXDIR" \
  > /dev/null 2> "$OUT/m85.dl28s.log" &
DPID=$!
up=0
UPBODY=$(mktemp)
for _ in $(seq 1 100); do
  # the FIRST SUCCESSFUL probe IS request 1 (its body is captured — no
  # second hello dispatch happens in this run)
  if curl -s -m 1 -o "$UPBODY" "http://127.0.0.1:$PORT/hello.php"; then up=1; break; fi
  sleep 0.1
done
[ $up = 1 ] || { fail "m85.dl28s server not up"; kill -TERM $DPID 2>/dev/null; }
WANT_H=$(cd "$FIXDIR" && "$ORACLE" -n hello.php 2>/dev/null)
[ "$(cat "$UPBODY")" = "$WANT_H" ] || fail "m85.dl28s up-probe body != oracle (KS-AH-83-1)"
rm -f "$UPBODY"
WANT_P=$(cd "$FIXDIR" && "$ORACLE" -n hello_pad85.php 2>/dev/null)
GOT_P=$(curl -s -m 10 "http://127.0.0.1:$PORT/hello_pad85.php")
[ "$GOT_P" = "$WANT_P" ] || fail "m85.dl28s full-body pad != oracle (KS-AH-83-1)"
kill -TERM "$DPID" 2>/dev/null
wait "$DPID" 2>/dev/null
rc=$?
echo "mem_hash=$MEM_HASH git=$GIT_REV campaign=$PHPR_CAMPAIGN_SCRIPT arm=axum-worker w=2 phase=dl28s server_exit=$rc" >> "$MC"
if grep -qE "panicked|aborting" "$OUT/m85.dl28s.log"; then
  fail "m85.dl28s stderr carries panic/abort markers (KS-MS-86-1)"
fi

echo "== dl28-supplement DONE fails=$FAILS =="
exit $FAILS
