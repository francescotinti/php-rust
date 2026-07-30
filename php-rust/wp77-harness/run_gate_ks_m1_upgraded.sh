#!/bin/bash
# KS-M1-Gate-Upgraded runner (WP-77.4.2): byte-snapshot parity on 3 workloads.
#
#   A. hello-world Axum endpoint  — 5 sequential + 2 parallel fetches, all
#      byte-identical AND non-empty (empty-body guard: an all-zero snapshot is
#      a failed positive control, not a PASS).
#   B. WordPress full site (phpr -S on ~/Claude/wpdev/src, DB wp) — front page,
#      post permalink, RSS feed; same 7-fetch parity + non-empty guard.
#      NOTE: use the REAL permalink for the post — /?p=1 is a 301 with empty
#      body and false-greens the hash check.
#   C. ORM suite — counts 3E/13F and fail-set BY NAME identical to the pinned
#      baseline gate-baseline/orm7742-fails.txt (16 names). The phpunit run is
#      heavy: run it separately (watchdog) and pass its output via ORM_OUT.
#
# Verdict: PASS iff A+B+C all pass. Emesso in $OUTDIR/ks_m1_upgraded.verdict.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
PHPSRV="${PHPSRV:-$HOME/Claude/php-rust-output/release/php-server}"
PHPR="${PHPR:-$HOME/Claude/php-rust-output/release/phpr}"
H77="$(cd "$(dirname "$0")" && pwd)"
OUTDIR="${OUTDIR:-/Volumes/Extreme Pro/Claude/wp77-harness/gate-out}"
WPDOC="${WPDOC:-$HOME/Claude/wpdev/src}"
ORM_OUT="${ORM_OUT:-}"
mkdir -p "$OUTDIR/snap"
VERDICT="$OUTDIR/ks_m1_upgraded.verdict"
[ -f "$VERDICT" ] && mv "$VERDICT" "$VERDICT.superseded-$(date +%Y%m%d%H%M%S)"
FAILS=0
ok(){ echo "  ✓ $1"; }
ko(){ echo "  ✗ $1"; FAILS=$((FAILS+1)); }

snap7() { # $1=name $2=url → distinct-hashes==1 and size>0
  local name="$1" url="$2" i n sz
  rm -f "$OUTDIR/snap/$name-"*.bin
  for i in 1 2 3 4 5; do curl -sg -m 60 "$url" -o "$OUTDIR/snap/$name-r$i.bin"; done
  # wait sui soli PID curl (un wait nudo aspetterebbe anche i server &)
  curl -sg -m 60 "$url" -o "$OUTDIR/snap/$name-p1.bin" & local w1=$!
  curl -sg -m 60 "$url" -o "$OUTDIR/snap/$name-p2.bin" & local w2=$!
  wait $w1 $w2
  n=$(shasum -a 256 "$OUTDIR/snap/$name-"*.bin | awk '{print $1}' | sort -u | wc -l | tr -d ' ')
  sz=$(wc -c < "$OUTDIR/snap/$name-r1.bin" | tr -d ' ')
  if [ "$n" = 1 ] && [ "$sz" -gt 0 ]; then ok "$name byte-id x7 (size=$sz)"; else ko "$name distinct=$n size=$sz"; fi
}

# --- Workload A: Axum hello-world (own server on 8197) ---
echo "== A. Axum hello-world =="
if pgrep -f "127.0.0.1:8197" >/dev/null 2>&1; then ko "stale server on 8197"; else
  ( "$PHPSRV" --axum --port 8197 >/dev/null 2>"$OUTDIR/axum-srv.log" ) &
  APID=$!
  sleep 1
  snap7 axum-hello "http://127.0.0.1:8197/hello-world"
  kill $APID 2>/dev/null; pkill -f "php-server --axum --port 8197" 2>/dev/null
fi

# --- Workload B: WordPress (server on 8080; reuse if already serving wpdev) ---
echo "== B. WordPress (docroot $WPDOC) =="
STARTED_WP=""
if ! pgrep -f "127.0.0.1:8080" >/dev/null 2>&1; then
  ( cd "$WPDOC" && "$PHPR" -S 127.0.0.1:8080 -t "$WPDOC" >/dev/null 2>"$OUTDIR/wp-srv.log" ) &
  STARTED_WP=$!
  sleep 2
fi
snap7 wp-front "http://127.0.0.1:8080/"
POST_URL=$(curl -sg -m 60 -o /dev/null -D - "http://127.0.0.1:8080/?p=1" | awk 'tolower($1)=="location:"{print $2}' | tr -d '\r')
if [ -n "$POST_URL" ]; then snap7 wp-post "$POST_URL"; else ko "post permalink not resolved"; fi
snap7 wp-feed "http://127.0.0.1:8080/?feed=rss2"
[ -n "$STARTED_WP" ] && { kill $STARTED_WP 2>/dev/null; pkill -f "phpr -S 127.0.0.1:8080" 2>/dev/null; }

# --- Workload C: ORM fail-set by name ---
echo "== C. ORM (baseline 3E/13F, 16 nomi) =="
if [ -z "$ORM_OUT" ] || [ ! -f "$ORM_OUT" ]; then
  ko "ORM_OUT non fornito (lancia phpunit sotto watchdog e passa il log)"
else
  SUMMARY=$(tr -d '\0' < "$ORM_OUT" | grep -E '^Tests: [0-9]+, Assertions:' | tail -1)
  echo "  summary: $SUMMARY"
  echo "$SUMMARY" | grep -q 'Errors: 3, Failures: 13' && ok "conteggio 3E/13F" || ko "conteggio diverso da 3E/13F"
  tr -d '\0' < "$ORM_OUT" | grep -E '^[0-9]+\) ' | sed 's/^[0-9]*) //' | sort > "$OUTDIR/orm-fails-current.txt"
  if diff -q "$H77/gate-baseline/orm7742-fails.txt" "$OUTDIR/orm-fails-current.txt" >/dev/null 2>&1; then
    ok "fail-set IDENTICO per nome (16)"
  else
    ko "fail-set DIVERSO per nome"; diff "$H77/gate-baseline/orm7742-fails.txt" "$OUTDIR/orm-fails-current.txt" | head -20
  fi
fi

if [ "$FAILS" = 0 ]; then
  echo "PASS fails=0 $(date +%F_%H:%M:%S) (emesso da run_gate_ks_m1_upgraded.sh)" > "$VERDICT"
  echo "== KS-M1-Gate-Upgraded PASS =="; exit 0
else
  echo "FAIL fails=$FAILS" > "$VERDICT"; echo "== KS-M1-Gate-Upgraded FAIL($FAILS) =="; exit 1
fi
