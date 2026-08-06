#!/bin/bash
# s102-collaudo-server.sh — S-102 punto 1 (Concilio WP-103, A-PE-103-1/2):
# collaudo del pin php-server 2c4242b6 — debito NON condizionato.
#
# USO: s102-collaudo-server.sh <off|on>
#
# Gambe (collaudo MINIMO che grada, A-PE-103-2):
#   A) sentinella ESTESA (16 interleaved su 3 endpoint + 4 concorrenti,
#      workers=2) CON mode-probe A-PE-102-1 — riusa s100-sentinella-estesa.sh;
#   B) dente CAPTURE-BOUNDARY nuovo: fixtures/cb1.php (output da
#      __destruct/shutdown alla request_end), servita 3 volte CONSECUTIVE
#      su workers=1 (stesso worker per costruzione), byte-id 1ª↔2ª↔3ª +
#      marker tutti presenti + corpo identico all'oracle (php -S) + suo
#      mode-probe dedicato (KS-PE-103-3: senza prova del modo NON grada).
#   Cross-mode: al secondo run, byte-id cb tra i due modi.
#
# AMBIENTE COSTRUITO, MAI SOTTRATTO (A-SK-93): re-exec env -i lista chiusa.
# Niente nuove build (A-PE-103-3): il pin si collauda com'è, FAIL-CLOSED.
set -u

case "${1:-}" in
  off) REG=0 ;;
  on)  REG=1 ;;
  *) echo "USO: $0 <off|on> — modo register-lowering ESPLICITO"; exit 64 ;;
esac
MODE="$1"

ENV_PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
ENV_HOME=/Users/francescotinti
ENV_TMPDIR=/tmp
ALLOWED='PATH|HOME|TMPDIR|LC_ALL|MIMALLOC_PURGE_DELAY|PHPR_REG_LOWER|S102_COLLAUDO_REEXEC|PWD|SHLVL|_'
EXTRA="$(/usr/bin/env | /usr/bin/sed 's/=.*//' | /usr/bin/grep -vxE "$ALLOWED" | /usr/bin/tr '\n' ' ')"
SANE=1
[ -z "$EXTRA" ] || SANE=0
[ "${PATH:-}" = "$ENV_PATH" ] || SANE=0
[ "${HOME:-}" = "$ENV_HOME" ] || SANE=0
[ "${TMPDIR:-}" = "$ENV_TMPDIR" ] || SANE=0
[ "${LC_ALL:-}" = C ] || SANE=0
[ "${MIMALLOC_PURGE_DELAY:-}" = 0 ] || SANE=0
[ "${PHPR_REG_LOWER:-}" = "$REG" ] || SANE=0
if [ "$SANE" != 1 ]; then
  if [ "${S102_COLLAUDO_REEXEC:-0}" != 0 ]; then
    echo "REFUSE s102-collaudo-server: ambiente ancora sporco dopo il re-exec — nomi extra: [${EXTRA:-none}]"; exit 65
  fi
  exec /usr/bin/env -i \
    PATH="$ENV_PATH" HOME="$ENV_HOME" TMPDIR="$ENV_TMPDIR" LC_ALL=C \
    MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER="$REG" S102_COLLAUDO_REEXEC=1 \
    /bin/bash "$0" ${@+"$@"}
fi
unset S102_COLLAUDO_REEXEC

# ---- pin e strumenti ----
PIN_SRV_ATTESO=2c4242b6c8120b8e
PHPSRV="$HOME/Claude/php-rust-output/release/php-server"
ORACLE=/opt/homebrew/opt/php/bin/php
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp102-harness"
SENTINEL="$REPO/wp100-harness/s100-sentinella-estesa.sh"
OUT="$H/collaudo-out-$MODE"
mkdir -p "$OUT"
step() { echo "== $(date +%H:%M:%S) [S102-COLLAUDO-$MODE] $1" | tee -a "$OUT/progress.txt"; }
FAILS=0

# ---- gamba 0: hash del pin, FAIL-CLOSED (KS-PE-100-3) ----
H_SRV="$(shasum -a 256 "$PHPSRV" | cut -c1-16)"
step "modo=$MODE (PHPR_REG_LOWER=$REG esplicito) hash php-server=$H_SRV (atteso $PIN_SRV_ATTESO)"
if [ "$H_SRV" != "$PIN_SRV_ATTESO" ]; then
  step "REFUSE: il binario php-server NON è il pin da collaudare"; echo "rc=65 $(date +%T)" > "$OUT/collaudo.done"; exit 65
fi

# ---- gamba A: sentinella ESTESA (con mode-probe interno) ----
step "gamba A: sentinella estesa sul pin, modo=$MODE"
if PHPSRV="$PHPSRV" OUTDIR="$OUT/sentinella" "$SENTINEL" >> "$OUT/sentinella.txt" 2>&1; then
  step "sentinella: PASS (con mode-probe)"
else
  step "sentinella: FAIL — vedi sentinella.txt"; FAILS=$((FAILS+1))
fi

# ---- gamba B: dente CAPTURE-BOUNDARY ----
CBPORT=8201
ORPORT=8202
for P in $CBPORT $ORPORT; do
  if pgrep -f "127.0.0.1:$P" >/dev/null 2>&1; then pkill -f "127.0.0.1:$P" 2>/dev/null || true; sleep 1; fi
done
DOC=$(mktemp -d)
cp "$H/fixtures/cb1.php" "$DOC/cb1.php"

# oracle di riferimento: php -S, 2 richieste (sanity byte-id anche lato oracle)
( "$ORACLE" -S 127.0.0.1:$ORPORT -t "$DOC" >/dev/null 2>&1 ) &
ORPID=$!
sleep 1
curl -s -m 5 "http://127.0.0.1:$ORPORT/cb1.php" > "$OUT/cb-oracle-1.out"
curl -s -m 5 "http://127.0.0.1:$ORPORT/cb1.php" > "$OUT/cb-oracle-2.out"
kill $ORPID 2>/dev/null || true
pkill -f "127.0.0.1:$ORPORT" 2>/dev/null || true
if ! cmp -s "$OUT/cb-oracle-1.out" "$OUT/cb-oracle-2.out"; then
  step "cb: VOID — l'oracle stesso non è byte-id tra 2 richieste (fixture non deterministica?)"; FAILS=$((FAILS+1))
fi

# server pin, workers=1 (STESSO worker per costruzione), dump per il mode-probe
( PHPR_DUMP_OPS=1 "$PHPSRV" --axum --workers 1 --port $CBPORT -t "$DOC" >/dev/null 2>"$OUT/cb-srv.log" ) &
SRV=$!
sleep 2
for r in 1 2 3; do
  curl -s -m 5 "http://127.0.0.1:$CBPORT/cb1.php" > "$OUT/cb-$r.out"
done
kill $SRV 2>/dev/null || true
pkill -f "127.0.0.1:$CBPORT" 2>/dev/null || true
sleep 0.3

# B1: marker tutti presenti nella PRIMA risposta (capture window intera)
for m in 'CB1:begin' 'CB1:sum=4950' 'DTOR:mid:x=50' 'CB1:req=1' 'CB1:body-done' \
         'SHUTDOWN:1' 'DTOR:shut:x=7' 'DTOR:bag' 'DTOR:inbag:x=9' 'DTOR:end:x=3'; do
  if ! grep -qF "$m" "$OUT/cb-1.out"; then
    step "cb: FAIL marker assente [$m] — output fuori dalla finestra di capture?"; FAILS=$((FAILS+1))
  fi
done
# B2: byte-id 1ª↔2ª↔3ª sullo stesso worker
for r in 2 3; do
  if ! cmp -s "$OUT/cb-1.out" "$OUT/cb-$r.out"; then
    step "cb: FAIL richiesta $r NON byte-id alla prima (stesso worker)"; diff "$OUT/cb-1.out" "$OUT/cb-$r.out" | head -5; FAILS=$((FAILS+1))
  fi
done
# B3: corpo identico all'oracle
if ! cmp -s "$OUT/cb-1.out" "$OUT/cb-oracle-1.out"; then
  step "cb: FAIL corpo DIVERSO dall'oracle php -S"; diff "$OUT/cb-oracle-1.out" "$OUT/cb-1.out" | head -10; FAILS=$((FAILS+1))
else
  step "cb: corpo identico all'oracle"
fi
# B4: mode-probe dedicato sul dump dell'unità cb1.php (KS-PE-103-3)
case "$REG" in 0) EXPECT_ON=0 ;; *) EXPECT_ON=1 ;; esac
CBCHUNK="$(tr -d '\0' < "$OUT/cb-srv.log" | awk '/^== unit .*\/cb1\.php/{f=1;next} /^== unit /{f=0} f')"
if [ -z "$CBCHUNK" ]; then
  step "cb: FAIL mode-probe — nessun dump dell'unità cb1.php nel log del server"; FAILS=$((FAILS+1))
else
  HAS_REG=0
  printf '%s' "$CBCHUNK" | grep -qE "BinarySS|BinarySC|BinaryDst|CmpJmpSC|CmpJmpSS" && HAS_REG=1
  if [ "$HAS_REG" != "$EXPECT_ON" ]; then
    step "cb: FAIL mode-probe — forme registro=$HAS_REG ma modo atteso on=$EXPECT_ON"; FAILS=$((FAILS+1))
  else
    step "cb: mode-probe OK (on=$EXPECT_ON provato dal dump dell'unità)"
  fi
fi
rm -rf "$DOC"

# ---- cross-mode (quando esiste l'altro braccio) ----
OTHER=off; [ "$MODE" = off ] && OTHER=on
if [ -f "$H/collaudo-out-$OTHER/cb-1.out" ]; then
  if cmp -s "$OUT/cb-1.out" "$H/collaudo-out-$OTHER/cb-1.out"; then
    step "cross-mode: cb byte-id tra i due modi"
  else
    step "cross-mode: FAIL — cb DIVERSA tra i due modi"; FAILS=$((FAILS+1))
  fi
else
  step "cross-mode: altro braccio non ancora eseguito (verrà giudicato lì)"
fi

step "S102-COLLAUDO-$MODE DONE fails=$FAILS"
echo "rc=$FAILS modo=$MODE $(date +%T)" > "$OUT/collaudo.done"
exit "$FAILS"
