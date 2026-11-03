#!/usr/bin/env python3
"""gen-battery61-axum.py — genera battery61-axum.sh come COPIA DICHIARATA di
wp94-harness/battery61.sh (regola shell-harness: ogni sostituzione asserita UNICA; manifest
battery61-axum-copia.diff prodotto dal chiamante). Adattamenti SOLI: il braccio phpr è
`php-server --axum --workers ${WORKERS:-4}` (binario PHPSERVER), pattern pkill/pgrep estesi alla
forma `--port N` dell'argv axum, OUT in wp188-harness, intestazione/etichette. L'oracle resta
`php -S` a 4 worker. Generato con assistenza AI (Claude Fable 5)."""
import os, stat

H = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(H, "..", "wp94-harness", "battery61.sh")
DST = os.path.join(H, "battery61-axum.sh")
s = open(SRC, encoding="utf-8").read()

SUBS = [
    ("# battery61.sh — S-94.0 OGGETTO #2: la batteria di accettazione WordPress di\n"
     "# WP-61, resa RIPRODUCIBILE sul MODO NATIVO (criterio 5 della chiusura del\n"
     "# fronte; debito di 31 sessioni: l'accettazione WP-61 era un verdetto\n"
     "# storico senza harness in repo).\n",
     "# battery61-axum.sh — S-188 Tappa 2 C5: COPIA DICHIARATA di wp94-harness/battery61.sh\n"
     "# (generatore gen-battery61-axum.py, manifest battery61-axum-copia.diff) coi SOLI adattamenti:\n"
     "# braccio phpr = `php-server --axum --workers ${WORKERS:-4}` (PHPSERVER), pkill/pgrep estesi alla\n"
     "# forma `--port N`, OUT in wp188-harness, etichette. Il resto INVARIATO (testo S-94.0 sotto).\n"),
    ('PHPR=${PHPR:-$HOME/Claude/php-rust-output/release/phpr}\n',
     'PHPSERVER=${PHPSERVER:-$HOME/Claude/php-rust-output/release/php-server}\nWORKERS=${WORKERS:-4}\n'),
    ('OUT="${OUT:-/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp94-harness/battery61-out}"\n',
     'OUT="${OUT:-/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp188-harness/battery61-axum-out}"\n'),
    ('  [ "$who" = oracle ] && bin="$ORACLE" || bin="$PHPR"\n'
     '  pkill -f "127.0.0.1:$PORT" 2>/dev/null; sleep 1\n'
     '  if pgrep -f "127.0.0.1:$PORT" >/dev/null; then echo "STALE SERVER on $PORT"; exit 3; fi\n',
     '  [ "$who" = oracle ] && bin="$ORACLE" || bin="$PHPSERVER"\n'
     '  pkill -f "127.0.0.1:$PORT" 2>/dev/null; pkill -f -- "--port $PORT" 2>/dev/null; sleep 1\n'
     '  if pgrep -f "127.0.0.1:$PORT" >/dev/null || pgrep -f -- "--port $PORT" >/dev/null; then echo "STALE SERVER on $PORT"; exit 3; fi\n'),
    ('  ( cd "$WPDEV" && PHP_CLI_SERVER_WORKERS=4 "$bin" -S "127.0.0.1:$PORT" -t "$WPDEV" >/dev/null 2>"$d/server.log" ) &\n',
     '  if [ "$who" = oracle ]; then\n'
     '    ( cd "$WPDEV" && PHP_CLI_SERVER_WORKERS=4 "$bin" -S "127.0.0.1:$PORT" -t "$WPDEV" >/dev/null 2>"$d/server.log" ) &\n'
     '  else\n'
     '    ( cd "$WPDEV" && "$bin" --axum --workers "$WORKERS" --host 127.0.0.1 --port "$PORT" -t "$WPDEV" >/dev/null 2>"$d/server.log" ) &\n'
     '  fi\n'),
    ('  kill $srv 2>/dev/null; pkill -f "127.0.0.1:$PORT" 2>/dev/null; sleep 1\n',
     '  kill $srv 2>/dev/null; pkill -f "127.0.0.1:$PORT" 2>/dev/null; pkill -f -- "--port $PORT" 2>/dev/null; sleep 1\n'),
    ('  echo "battery61 — accettazione WordPress sul MODO NATIVO (phpr -S vs php -S)"\n',
     '  echo "battery61-axum — accettazione WordPress sul canale AXUM (php-server --axum --workers $WORKERS vs php -S)"\n'),
    ('  echo "phpr=$(shasum -a 256 "$PHPR" | cut -c1-16)"\n',
     '  echo "php-server=$(shasum -a 256 "$PHPSERVER" | cut -c1-16) workers=$WORKERS"\n'),
    ('  echo "battery61 rc=$rc"\n} | tee "$OUT/battery61.out"\necho "rc=$rc $(date +%T)" > "$OUT/battery61.done"\n',
     '  echo "battery61-axum rc=$rc"\n} | tee "$OUT/battery61-axum.out"\necho "rc=$rc $(date +%T)" > "$OUT/battery61-axum.done"\n'),
    ('rm -f "$OUT/battery61.done"\n', 'rm -f "$OUT/battery61-axum.done"\n'),
]
for old, new in SUBS:
    n = s.count(old)
    assert n == 1, f"sostituzione non unica ({n}): {old[:70]!r}"
    s = s.replace(old, new)
open(DST, "w", encoding="utf-8").write(s)
os.chmod(DST, os.stat(DST).st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
print(f"scritto {DST} ({len(SUBS)} sostituzioni uniche)")
