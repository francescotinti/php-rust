#!/bin/bash
# KM-77-2 gate runner (WP-77.4.2): per-request $GLOBALS isolation over HTTP,
# 15 rounds x 2 parallel curls against php-server (cli-server mode) serving
# fixtures/gate_km77_2_http.php — the BITING fixture (unique stamp per request;
# the original gate_km77_2_concurrent.php false-PASSes on leaks and is kept
# only as history).
#
# Verdict: PASS iff 30/30 responses say PASS, 30 unique markers, and the
# positive control (stale $GLOBALS injected before include) says FAIL.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
PHPSRV="${PHPSRV:-$HOME/Claude/php-rust-output/release/php-server}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
H77="$(cd "$(dirname "$0")" && pwd)"
OUTDIR="${OUTDIR:-/Volumes/Extreme Pro/Claude/wp77-harness/gate-out}"
mkdir -p "$OUTDIR"
PORT=8198
VERDICT="$OUTDIR/km77_2.verdict"
[ -f "$VERDICT" ] && mv "$VERDICT" "$VERDICT.superseded-$(date +%Y%m%d%H%M%S)"
FAILS=0

if pgrep -f "127.0.0.1:$PORT" >/dev/null 2>&1; then echo "STALE SERVER on $PORT"; exit 3; fi
DOC=$(mktemp -d)
cp "$H77/fixtures/gate_km77_2_http.php" "$DOC/km77_2_http.php"
( "$PHPSRV" --port $PORT -t "$DOC" >/dev/null 2>"$OUTDIR/km77_2-srv.log" ) &
SRV=$!
sleep 2

OUT="$OUTDIR/km77_2.out"; : > "$OUT"
for i in $(seq 1 15); do
  # wait SOLO sui PID delle curl: un `wait` nudo aspetterebbe anche il
  # server backgrounded e non torna mai (bug morso al primo run).
  curl -s -m 5 "http://127.0.0.1:$PORT/km77_2_http.php" >> "$OUT" & C1=$!
  curl -s -m 5 "http://127.0.0.1:$PORT/km77_2_http.php" >> "$OUT" & C2=$!
  wait $C1 $C2
done
kill $SRV 2>/dev/null; pkill -f "127.0.0.1:$PORT" 2>/dev/null

total=$(grep -c . "$OUT"); pass=$(grep -c '^PASS' "$OUT"); uniq=$(awk '{print $2}' "$OUT" | sort -u | wc -l | tr -d ' ')
echo "total=$total pass=$pass uniq=$uniq"
[ "$total" = 30 ] && [ "$pass" = 30 ] && [ "$uniq" = 30 ] || FAILS=$((FAILS+1))

# Positive control: fixture MUST bite when state persists.
PC=$(printf '<?php\n$GLOBALS["km77_marker"]="stale";\ninclude "%s/km77_2_http.php";\n' "$DOC" | tee "$DOC/pc.php" >/dev/null; "$PHPR" "$DOC/pc.php")
echo "positive-control: $PC"
echo "$PC" | grep -q '^FAIL' || { echo "POSITIVE CONTROL DID NOT BITE"; FAILS=$((FAILS+1)); }

rm -rf "$DOC"
if [ "$FAILS" = 0 ]; then
  echo "PASS fails=0 $(date +%F_%H:%M:%S) (emesso da run_gate_km77_2.sh)" > "$VERDICT"
  echo "== KM-77-2 PASS =="; exit 0
else
  echo "FAIL fails=$FAILS" > "$VERDICT"; echo "== KM-77-2 FAIL($FAILS) =="; exit 1
fi
