#!/bin/bash
# s100-sentinella-estesa.sh — A-PE-101-3 (Concilio WP-101): la sentinella
# server da 2 richieste era troppo corta per il «sì» pieno. Questa:
#   - N=16 richieste INTERLEAVED su 3 endpoint (g2 reset-invarianti,
#     payload deterministico con static/oid/GLOBALS, json restapi-shaped
#     via HTTP con query alternata);
#   - byte-identita' per (endpoint,query) contro la PRIMA risposta (i leak
#     monotoni tipo WP-78 mordono dopo la seconda);
#   - workers=2 + burst di 4 richieste CONCORRENTI in coda;
#   - il modo register-lowering viene dall'ambiente allo spawn del server:
#     chiamare sotto launcher bimodale (PHPR_REG_LOWER esplicito).
# USO: PHPSRV=<bin> OUTDIR=<dir> s100-sentinella-estesa.sh
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
PHPSRV="${PHPSRV:-$HOME/Claude/php-rust-output/release/php-server}"
H="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp100-harness"
H77="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp77-harness"
OUTDIR="${OUTDIR:-$H/sentinella-out}"
mkdir -p "$OUTDIR"
PORT=8199
FAILS=0

if pgrep -f "127.0.0.1:$PORT" >/dev/null 2>&1; then
  pkill -f "127.0.0.1:$PORT" 2>/dev/null || true; sleep 1
fi
DOC=$(mktemp -d)
cp "$H77/fixtures/gate_two_reqs_same_vm.php" "$DOC/g2.php"
cp "$H/sentinella-fixtures/payload.php" "$DOC/p1.php"
cp "$H/sentinella-fixtures/json.php" "$DOC/j1.php"

( "$PHPSRV" --axum --workers 2 --port $PORT -t "$DOC" >/dev/null 2>"$OUTDIR/srv.log" ) &
SRV=$!
sleep 2

# 16 richieste interleaved: g2, p1, j1?A, j1?B ripetute 4 volte.
declare -a KEYS=(g2 p1 jA jB)
url() { case "$1" in
  g2) echo "http://127.0.0.1:$PORT/g2.php" ;;
  p1) echo "http://127.0.0.1:$PORT/p1.php" ;;
  jA) echo "http://127.0.0.1:$PORT/j1.php?rest_route=/wp/v2/posts" ;;
  jB) echo "http://127.0.0.1:$PORT/j1.php?rest_route=/wp/v2/users" ;;
esac; }
n=0
for round in 1 2 3 4; do
  for k in "${KEYS[@]}"; do
    n=$((n+1))
    curl -s -m 5 "$(url "$k")" > "$OUTDIR/$k-$round.out"
    if [ "$round" = 1 ]; then
      # controllo positivo: la prima risposta non e' vuota e ha la firma
      case "$k" in
        g2) grep -q '^G2:PASS' "$OUTDIR/$k-1.out" || { echo "FAIL $k: prima risposta senza G2:PASS"; FAILS=$((FAILS+1)); } ;;
        p1) grep -q '^P1:sum=' "$OUTDIR/$k-1.out" || { echo "FAIL $k: prima risposta senza P1:"; FAILS=$((FAILS+1)); } ;;
        jA|jB) grep -q '"route"' "$OUTDIR/$k-1.out" || { echo "FAIL $k: prima risposta senza JSON"; FAILS=$((FAILS+1)); } ;;
      esac
    elif ! cmp -s "$OUTDIR/$k-1.out" "$OUTDIR/$k-$round.out"; then
      echo "FAIL $k round $round: NON byte-identica alla prima (leak monotono?)"
      diff "$OUTDIR/$k-1.out" "$OUTDIR/$k-$round.out" | head -5
      FAILS=$((FAILS+1))
    fi
  done
done
echo "sequenza interleaved: $n richieste"

# burst concorrente (workers=2 esercitati davvero). `wait` NUDO aspetterebbe
# anche la subshell del SERVER (che non esce mai — morso in smoke S-100):
# si aspettano SOLO i pid dei curl.
CURL_PIDS=""
for c in 1 2 3 4; do
  curl -s -m 10 "$(url p1)" > "$OUTDIR/conc-$c.out" &
  CURL_PIDS="$CURL_PIDS $!"
done
wait $CURL_PIDS
for c in 1 2 3 4; do
  if ! cmp -s "$OUTDIR/p1-1.out" "$OUTDIR/conc-$c.out"; then
    echo "FAIL conc-$c: risposta concorrente diversa dalla riferimento"
    FAILS=$((FAILS+1))
  fi
done

kill $SRV 2>/dev/null || true
pkill -f "127.0.0.1:$PORT" 2>/dev/null || true
rm -rf "$DOC"
if [ "$FAILS" = 0 ]; then echo "SENTINELLA-ESTESA PASS (16 interleaved + 4 concorrenti, workers=2)"; else echo "SENTINELLA-ESTESA FAIL fails=$FAILS"; fi
exit "$FAILS"
