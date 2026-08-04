#!/bin/bash
# battery61.sh — S-94.0 OGGETTO #2: la batteria di accettazione WordPress di
# WP-61, resa RIPRODUCIBILE sul MODO NATIVO (criterio 5 della chiusura del
# fronte; debito di 31 sessioni: l'accettazione WP-61 era un verdetto
# storico senza harness in repo).
#
# Modo nativo = il server HTTP integrato: oracle `php -S` contro `phpr -S`,
# STESSO albero (wpdev/src), STESSO DB, STESSA porta, IN SEQUENZA — pattern
# run-http.sh (WP-9), tabella di accettazione WP_SESSION_61 §Accettazione.
#
# Sei probe. Cinque pretendono BYTE-ID; la dashboard e NORM-ID sul solo
# `_wpnonce` (le due sessioni di login sono distinte per costruzione, quindi
# il nonce differisce per ragione NOMINATA, non per divergenza).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
WPDEV="/Users/francescotinti/Claude/wpdev/src"
ORACLE=${ORACLE:-/opt/homebrew/opt/php/bin/php}
PHPR=${PHPR:-$HOME/Claude/php-rust-output/release/phpr}
PORT=${PORT:-8080}   # = la porta di siteurl (http://127.0.0.1:8080): servire altrove fa redirigere WP altrove
BASE="http://127.0.0.1:$PORT"
OUT="${OUT:-/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp94-harness/battery61-out}"
WPUSER=${WPUSER:-admin}
WPPASS=${WPPASS:-wp-secret-Pass1}   # credenziale dell installazione viva wpdev (WP_SESSION_61)
mkdir -p "$OUT"
rm -f "$OUT/battery61.done"

serve_and_capture() { # $1 = oracle|phpr, $2 = outdir
  local who="$1" d="$2" bin
  mkdir -p "$d"; rm -f "$d"/*
  [ "$who" = oracle ] && bin="$ORACLE" || bin="$PHPR"
  pkill -f "127.0.0.1:$PORT" 2>/dev/null; sleep 1
  if pgrep -f "127.0.0.1:$PORT" >/dev/null; then echo "STALE SERVER on $PORT"; exit 3; fi
  # `php -S` e MONO-PROCESSO e WordPress fa richieste HTTP verso SE STESSO
  # (Requests/Curl): con un solo worker il server si auto-blocca fino al
  # "Maximum execution time exceeded" — osservato, non temuto. PHP_CLI_SERVER_WORKERS
  # e la sola ragione per cui questa batteria puo esistere sul modo nativo.
  ( cd "$WPDEV" && PHP_CLI_SERVER_WORKERS=4 "$bin" -S "127.0.0.1:$PORT" -t "$WPDEV" >/dev/null 2>"$d/server.log" ) &
  local srv=$!
  # attesa ATTIVA del servizio: un sleep fisso e una scommessa, non un gate
  local i=0
  until curl -s -o /dev/null --max-time 5 "$BASE/" 2>/dev/null; do
    i=$((i+1)); [ "$i" -gt 40 ] && { echo "SERVER $who non risponde su $PORT"; kill $srv 2>/dev/null; exit 4; }
    sleep 0.5
  done
  local jar="$d/cookies.txt"; rm -f "$jar"
  curl -sg --max-time 120 -D "$d/1-front.hdr"     -o "$d/1-front.body"     "$BASE/"
  curl -sg --max-time 120 -D "$d/2-asset.hdr"     -o "$d/2-asset.body"     "$BASE/wp-includes/css/buttons.css"
  curl -sg --max-time 120 -D "$d/3-permalink.hdr" -o "$d/3-permalink.body" "$BASE/2026/07/hello-world/"
  curl -sg --max-time 120 -c "$jar" -D "$d/4-login.hdr" -o "$d/4-login.body" "$BASE/wp-login.php"
  curl -sg --max-time 120 -b "$jar" -c "$jar" -D "$d/5-loginpost.hdr" -o "$d/5-loginpost.body" \
    --data-urlencode "log=$WPUSER" --data-urlencode "pwd=$WPPASS" \
    --data "wp-submit=Log+In&redirect_to=$BASE/wp-admin/&testcookie=1" "$BASE/wp-login.php"
  curl -sg --max-time 300 -b "$jar" -D "$d/6-admin.hdr" -o "$d/6-admin.body" "$BASE/wp-admin/"
  kill $srv 2>/dev/null; pkill -f "127.0.0.1:$PORT" 2>/dev/null; sleep 1
}

serve_and_capture oracle "$OUT/o"
serve_and_capture phpr   "$OUT/p"

# NORM: i volatili per NOME, nessun normalizzatore generico. Il nonce di WP
# dipende dalla sessione di login, che e distinta per costruzione fra le due
# catture: normalizzarlo e legittimo SOLO sulla dashboard, e solo lui.
norm() { perl -pe 's/[0-9a-f]{10}\b/NONCE10/g' "$1"; }

rc=0
{
  echo "battery61 — accettazione WordPress sul MODO NATIVO (phpr -S vs php -S)"
  echo "porta=$PORT docroot=$WPDEV"
  echo "oracle=$("$ORACLE" -v | head -1)"
  echo "phpr=$(shasum -a 256 "$PHPR" | cut -c1-16)"
  echo "epoch=$(date +%s)"
  echo
  for probe in 1-front 2-asset 3-permalink 4-login 5-loginpost; do
    os=$(head -1 "$OUT/o/$probe.hdr" | tr -d '\r'); ps=$(head -1 "$OUT/p/$probe.hdr" | tr -d '\r')
    ob=$(wc -c < "$OUT/o/$probe.body" | tr -d ' '); pb=$(wc -c < "$OUT/p/$probe.body" | tr -d ' ')
    if [ "$os" != "$ps" ]; then
      echo "$probe STATUS-DIFF oracle='$os' phpr='$ps'"; rc=1
    elif cmp -s "$OUT/o/$probe.body" "$OUT/p/$probe.body"; then
      echo "$probe BYTE-ID status='$os' bytes=$ob"
    else
      echo "$probe BYTE-DIFF status='$os' oracle_bytes=$ob phpr_bytes=$pb"; rc=1
    fi
  done
  os=$(head -1 "$OUT/o/6-admin.hdr" | tr -d '\r'); ps=$(head -1 "$OUT/p/6-admin.hdr" | tr -d '\r')
  ob=$(wc -c < "$OUT/o/6-admin.body" | tr -d ' '); pb=$(wc -c < "$OUT/p/6-admin.body" | tr -d ' ')
  if [ "$os" != "$ps" ]; then
    echo "6-admin STATUS-DIFF oracle='$os' phpr='$ps'"; rc=1
  elif cmp -s "$OUT/o/6-admin.body" "$OUT/p/6-admin.body"; then
    echo "6-admin BYTE-ID status='$os' bytes=$ob   # piu forte del richiesto (NORM-ID bastava)"
  elif diff -q <(norm "$OUT/o/6-admin.body") <(norm "$OUT/p/6-admin.body") >/dev/null; then
    echo "6-admin NORM-ID status='$os' bytes=$ob/$pb (differisce SOLO nei nonce a 10 hex — sessioni di login distinte per costruzione)"
  else
    echo "6-admin NORM-DIFF status='$os' oracle_bytes=$ob phpr_bytes=$pb"; rc=1
  fi
  echo
  echo "battery61 rc=$rc"
} | tee "$OUT/battery61.out"
echo "rc=$rc $(date +%T)" > "$OUT/battery61.done"
exit $rc
