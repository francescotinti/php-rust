#!/bin/bash
# G-APERTURA-2 executable gate (A-SK2, Council WP-78) — KS-SK-78.1 / KS-SK-78.2 / KS-DS-78-1.
#
# Contract (Klabnik A-SK2 + Stogov A-DS3):
#   1. Build php-server WITH the axum-server feature (the deployed binary must
#      contain the code under test — the WP-77.6.5.2.4 "PASS" was vacuous
#      precisely because cargo test never compiled worker_pool.rs).
#   2. Record shasum-256 of the php-server binary (A-SK5: the Axum baseline is
#      php-server, NEVER phpr).
#   3. Start the server in --axum mode, run sequential POST requests:
#        - hello.php          x2  → byte-identical bodies (cmp)
#        - gate_stateful.php  x3  → byte-identical bodies (cmp) AND each equal to
#          the EXPECTED line (positive control, WP-72 lesson: all-identical-but-
#          wrong must not pass). Statics/static-props/closure-statics/define()
#          must restart per request (FPM semantics) → KS-DS-78-1 on drift.
#   4. Deterministic exit code: 0 = PASS, 1 = FAIL.
#
# A PASS of G-APERTURA-2 may only be declared by citing THIS command and the
# binary hash it logs.
#
# S-78.1.3 hardening (Council WP-79 A-SK8/A-SK9/A-SK10/A-AH7):
# - BIN is the artifact cargo REPORTS in --message-format=json (KS-AH-78-5:
#   hashing a path at rest proved nothing about what the build wrote).
# - The PID listening on the port must BE the spawned $SRV (KS-SK-79.1:
#   otherwise the requests may be served by a stale process — run NULLA).
# - expected bodies are REGENERATED from the PHP 8.5.7 oracle with the command
#   logged, then cross-checked against the pinned strings (drift in fixture OR
#   oracle is loud, and self-equality never becomes the oracle again).
# - AppleDouble ._* files are purged from fixtures/ (A-SK10).
# - Every verdict line carries the git rev (A-SK9).
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
OUTDIR="$HERE/out"
mkdir -p "$OUTDIR"
PORT="${PORT:-8199}"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
VERDICT="$OUTDIR/g_apertura_2.verdict"
[ -f "$VERDICT" ] && mv "$VERDICT" "$VERDICT.superseded-$(date +%Y%m%d%H%M%S)"
FAILS=0

# A-SK10: AppleDouble droppings must never sit next to the fixtures a server
# is pointed at.
find "$HERE/fixtures" -name '._*' -delete

# 1. Build with the feature (union with default is fine for the dual-mode binary;
#    the feature identity of the DEFAULT binary is gated by gate-feature-matrix.sh).
#    A-AH7/KS-AH-78-5: capture cargo's own artifact report — BIN comes from it.
echo "== build -p php-server --features axum-server (json artifact capture) =="
( cd "$REPO" && cargo build --release -p php-server --features axum-server \
    --message-format=json ) > "$OUTDIR/build.json" 2> "$OUTDIR/build.log"
if [ $? -ne 0 ]; then
  echo "FAIL: build failed (see $OUTDIR/build.log)"
  echo "FAIL build [git $GIT_REV]" > "$VERDICT"; exit 1
fi
BIN="$(python3 - "$OUTDIR/build.json" <<'PY'
import json, sys
path = ""
for line in open(sys.argv[1]):
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("reason") == "compiler-artifact" \
       and m.get("target", {}).get("name") == "php-server" \
       and m.get("executable"):
        path = m["executable"]
print(path)
PY
)"
if [ -z "$BIN" ] || [ ! -x "$BIN" ]; then
  echo "FAIL: cargo did not report an executable artifact (KS-AH-78-5)"
  echo "FAIL artifact [git $GIT_REV]" > "$VERDICT"; exit 1
fi
echo "artifact (from cargo json): $BIN"

# 2. Binary identity (A-SK5) — hash of the artifact cargo reported, not a path
#    at rest.
HASH=$(shasum -a 256 "$BIN" | cut -c1-16)
echo "php-server sha256[0:16]=$HASH"

# Kill stale servers on our port
pkill -f "php-server --axum --port $PORT" 2>/dev/null && sleep 1

# 3. Start server
"$BIN" --axum --port "$PORT" -t "$HERE/fixtures" \
  > /dev/null 2> "$OUTDIR/server.log" &
SRV=$!
# Readiness poll (max 10s) — no blind sleep
up=0
for _ in $(seq 1 100); do
  if curl -s -o /dev/null -m 1 "http://127.0.0.1:$PORT/hello.php"; then up=1; break; fi
  sleep 0.1
done
if [ "$up" != 1 ]; then
  echo "FAIL: server did not come up (see $OUTDIR/server.log)"
  kill "$SRV" 2>/dev/null; echo "FAIL server-up [git $GIT_REV]" > "$VERDICT"; exit 1
fi

# KS-SK-79.1 (A-SK8): the PID listening on the port must BE the process we
# spawned — otherwise a stale server answers and the run is NULL.
LPID="$(lsof -nP -tiTCP:"$PORT" -sTCP:LISTEN | head -1)"
if [ "$LPID" != "$SRV" ]; then
  echo "FAIL: PID on port $PORT is '$LPID', spawned SRV is '$SRV' — run NULLA (KS-SK-79.1)"
  kill "$SRV" 2>/dev/null; echo "FAIL pid-mismatch [git $GIT_REV]" > "$VERDICT"; exit 1
fi
echo "OK  port $PORT is served by spawned PID $SRV (KS-SK-79.1)"

req() { curl -s -m 5 -X POST "http://127.0.0.1:$PORT/$1"; }

# Expected bodies are compared IN FULL (cmp): a head -1 check was blind to the
# doubled-body bug (rendered+stdout concatenation) — self-equality plus a
# first-line probe is NOT a positive control (S-78.0 lesson).
#
# A-SK8 (Council WP-79): the expected bodies are REGENERATED from the oracle,
# command logged; the pinned strings remain as a cross-check so drift in either
# the fixture or the oracle output is loud.
ORACLE="${ORACLE:-/opt/homebrew/opt/php/bin/php}"
if [ ! -x "$ORACLE" ]; then
  echo "FAIL: oracle not found at $ORACLE — expected bodies cannot be regenerated (A-SK8)"
  kill "$SRV" 2>/dev/null; echo "FAIL oracle-absent [git $GIT_REV]" > "$VERDICT"; exit 1
fi
echo "oracle: $ORACLE ($("$ORACLE" -v | head -1))"
echo "cmd: $ORACLE fixtures/hello.php > hello.expected ; $ORACLE fixtures/gate_stateful.php > stateful.expected"
"$ORACLE" "$HERE/fixtures/hello.php"         > "$OUTDIR/hello.expected"
"$ORACLE" "$HERE/fixtures/gate_stateful.php" > "$OUTDIR/stateful.expected"
if ! printf 'hello axum\nsum=42\n' | cmp -s - "$OUTDIR/hello.expected"; then
  echo "FAIL: oracle hello output drifted from pinned string:"
  cat "$OUTDIR/hello.expected"
  FAILS=$((FAILS+1))
fi
if ! printf 'G3:fn=12;prop=12;closure=12;const=fresh;constval=1\n' | cmp -s - "$OUTDIR/stateful.expected"; then
  echo "FAIL: oracle stateful output drifted from pinned string:"
  cat "$OUTDIR/stateful.expected"
  FAILS=$((FAILS+1))
fi

# hello.php x2 — byte parity (KS-SK-78.2) + full-body oracle match
req hello.php > "$OUTDIR/hello.1"
req hello.php > "$OUTDIR/hello.2"
if cmp -s "$OUTDIR/hello.1" "$OUTDIR/hello.2"; then
  echo "OK  hello: 2 bodies byte-identical"
else
  echo "FAIL hello: bodies differ (KS-SK-78.2 → REJECT S-77.6.5.2.3)"
  FAILS=$((FAILS+1))
fi
if ! cmp -s "$OUTDIR/hello.1" "$OUTDIR/hello.expected"; then
  echo "FAIL hello: body != oracle body:"
  diff "$OUTDIR/hello.expected" "$OUTDIR/hello.1" | head -5
  FAILS=$((FAILS+1))
fi

# gate_stateful.php x3 — byte parity + full-body oracle match (KS-DS-78-1)
for i in 1 2 3; do req gate_stateful.php > "$OUTDIR/stateful.$i"; done
for i in 2 3; do
  if ! cmp -s "$OUTDIR/stateful.1" "$OUTDIR/stateful.$i"; then
    echo "FAIL stateful: request 1 vs $i differ (KS-DS-78-1)"
    FAILS=$((FAILS+1))
  fi
done
for i in 1 2 3; do
  if ! cmp -s "$OUTDIR/stateful.$i" "$OUTDIR/stateful.expected"; then
    echo "FAIL stateful req $i: body != oracle body (KS-DS-78-1):"
    diff "$OUTDIR/stateful.expected" "$OUTDIR/stateful.$i" | head -5
    FAILS=$((FAILS+1))
  fi
done
[ "$FAILS" = 0 ] && echo "OK  stateful: 3 bodies byte-identical and == oracle"

# --- A-TH8 fatal contract (S-78.1.4, Council WP-79) — KH79-2/KS-DS-78-5 ------
# Every fatal (compile AND runtime) → HTTP 500 with the FULL oracle-CLI stdout
# as body; exit() → 200 with captured output; parse error → 500 with the
# "\nParse error: " envelope (message text is a REGISTERED divergence, no
# oracle cmp — PHPR_DIVERGENCES_FROM_PHP.md 3.9). Each fatal runs TWICE:
# byte-parity across requests AND worker survival.
req_st() { # req_st <fixture> <bodyfile> -> echoes http status
  curl -s -m 5 -o "$2" -w '%{http_code}' -X POST "http://127.0.0.1:$PORT/$1"
}

fatal_check() { # fatal_check <name> <want_status> <oracle_cmp:yes|no>
  local name="$1" want="$2" cmp_oracle="$3" st1 st2
  st1=$(req_st "$name.php" "$OUTDIR/$name.1")
  st2=$(req_st "$name.php" "$OUTDIR/$name.2")
  if [ "$st1" != "$want" ] || [ "$st2" != "$want" ]; then
    echo "FAIL $name: status $st1/$st2 != $want (A-TH8)"
    FAILS=$((FAILS+1))
  fi
  if ! cmp -s "$OUTDIR/$name.1" "$OUTDIR/$name.2"; then
    echo "FAIL $name: bodies differ across requests (KS-SK-78.2)"
    FAILS=$((FAILS+1))
  fi
  if [ "$cmp_oracle" = yes ]; then
    "$ORACLE" "$HERE/fixtures/$name.php" > "$OUTDIR/$name.expected" 2>/dev/null
    if ! cmp -s "$OUTDIR/$name.1" "$OUTDIR/$name.expected"; then
      echo "FAIL $name: body != oracle CLI stdout (KH79-2 full-body cmp):"
      diff "$OUTDIR/$name.expected" "$OUTDIR/$name.1" | head -8
      FAILS=$((FAILS+1))
    fi
  fi
}

fatal_check fatal_runtime 500 yes
fatal_check fatal_compile 500 yes
fatal_check exit_code     200 yes
fatal_check parse_err     500 no
# parse envelope form (the only assert parity allows here — KS-DS-78-5)
if ! head -c 14 "$OUTDIR/parse_err.1" | cmp -s - <(printf '\nParse error: '); then
  echo "FAIL parse_err: body does not start with the '\\nParse error: ' envelope:"
  head -2 "$OUTDIR/parse_err.1"
  FAILS=$((FAILS+1))
fi
# Worker survival after the fatal batch (boundary discipline)
req hello.php > "$OUTDIR/hello.after-fatals"
if cmp -s "$OUTDIR/hello.after-fatals" "$OUTDIR/hello.expected"; then
  echo "OK  fatal contract: 4 fixtures conform (500/500/200/500), workers alive"
else
  echo "FAIL: hello diverges after fatal batch — worker state polluted"
  FAILS=$((FAILS+1))
fi

# Teardown
kill "$SRV" 2>/dev/null
wait "$SRV" 2>/dev/null

if [ "$FAILS" = 0 ]; then
  echo "PASS fails=0 bin=$HASH git=$GIT_REV $(date +%F_%H:%M:%S) (run-gate.sh)" > "$VERDICT"
  echo "== G-APERTURA-2 PASS (bin=$HASH git=$GIT_REV) =="; exit 0
else
  echo "FAIL fails=$FAILS bin=$HASH git=$GIT_REV" > "$VERDICT"
  echo "== G-APERTURA-2 FAIL($FAILS) bin=$HASH git=$GIT_REV =="; exit 1
fi
