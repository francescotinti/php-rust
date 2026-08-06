#!/bin/bash
# s103-collaudo-server.sh — S-103 punto 1 (Concilio WP-104): collaudo del
# pin php-server NUOVO (runtime S-102 con fix §3.13) — launcher EMENDATO.
#
# USO: s103-collaudo-server.sh <off|on>
#
# Emendamenti WP-104 rispetto a s102-collaudo-server.sh:
#   C) braccio WARNING-LINE cb2 (KS-PE-104-1: il collaudo S-102 era CIECO
#      al §3.13 che motiva il pin — senza questo braccio NON grada):
#      fixture servita che emette 2 warning undef-prop, riga giudicata al
#      confine HTTP (byte-id oracle + parità esplicita dei numeri
#      «on line N» + conteggio esatto 2).
#   D) INTERLEAVING CROSS-FIXTURE workers=2: cb1/cb2 alternate su due
#      worker, ogni risposta byte-id alla referenza del proprio braccio
#      (contaminazione cross-richiesta/cross-worker di diag_line_marks).
#   E) cella ERRORE-POI-SUCCESSO: cbE (warning accodato + fatal) POI cb2
#      sullo stesso worker byte-id alla referenza (lo stato pendente non
#      sopravvive al confine richiesta).
#   PIN_SRV_ATTESO letto da file (aggiornato DOPO la build con ricetta);
#   DOC a path FISSO (i warning embeddano il path: mktemp romperebbe il
#   cross-mode byte-id).
# Invariati da S-102: gamba A sentinella estesa (mode-probe A-PE-102-1),
# gamba B dente capture-boundary cb1 (KS-PE-103-3).
#
# AMBIENTE COSTRUITO, MAI SOTTRATTO (A-SK-93): re-exec env -i lista chiusa.
# Niente nuove build dentro il collaudo (A-PE-103-3): FAIL-CLOSED sul pin.
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
ALLOWED='PATH|HOME|TMPDIR|LC_ALL|MIMALLOC_PURGE_DELAY|PHPR_REG_LOWER|S103_COLLAUDO_REEXEC|PWD|SHLVL|_'
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
  if [ "${S103_COLLAUDO_REEXEC:-0}" != 0 ]; then
    echo "REFUSE s103-collaudo-server: ambiente ancora sporco dopo il re-exec — nomi extra: [${EXTRA:-none}]"; exit 65
  fi
  exec /usr/bin/env -i \
    PATH="$ENV_PATH" HOME="$ENV_HOME" TMPDIR="$ENV_TMPDIR" LC_ALL=C \
    MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER="$REG" S103_COLLAUDO_REEXEC=1 \
    /bin/bash "$0" ${@+"$@"}
fi
unset S103_COLLAUDO_REEXEC

# ---- pin e strumenti ----
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp103-harness"
PIN_SRV_ATTESO="$(cat "$H/PIN_SRV_ATTESO.txt" 2>/dev/null | tr -d '[:space:]')"
PHPSRV="$HOME/Claude/php-rust-output/release/php-server"
ORACLE=/opt/homebrew/opt/php/bin/php
SENTINEL="$REPO/wp100-harness/s100-sentinella-estesa.sh"
OUT="$H/collaudo-out-$MODE"
mkdir -p "$OUT"
step() { echo "== $(date +%H:%M:%S) [S103-COLLAUDO-$MODE] $1" | tee -a "$OUT/progress.txt"; }
FAILS=0

# ---- gamba 0: hash del pin, FAIL-CLOSED (KS-PE-100-3) ----
case "$PIN_SRV_ATTESO" in
  ""|PENDING*) echo "REFUSE: PIN_SRV_ATTESO.txt non valorizzato (build con ricetta prima)"; exit 65 ;;
esac
H_SRV="$(shasum -a 256 "$PHPSRV" | cut -c1-16)"
step "modo=$MODE (PHPR_REG_LOWER=$REG esplicito) hash php-server=$H_SRV (atteso $PIN_SRV_ATTESO)"
if [ "$H_SRV" != "$PIN_SRV_ATTESO" ]; then
  step "REFUSE: il binario php-server NON è il pin da collaudare"; echo "rc=65 $(date +%T)" > "$OUT/collaudo.done"; exit 65
fi

# ---- DOC a path FISSO (deterministico cross-mode: i warning embeddano il path) ----
# CANONICO (/private/tmp, non /tmp): php -S canonicalizza il docroot
# attraverso il symlink macOS, phpr no — col path letterale i warning
# divergono per il SOLO prefisso (osservazione S-103, prima run off);
# il collaudo giudica il motore, non la scelta di symlink dell'harness.
DOC=/private/tmp/s103-cb-doc
rm -rf "$DOC"; mkdir -p "$DOC"
cp "$REPO/wp102-harness/fixtures/cb1.php" "$DOC/cb1.php"
cp "$H/fixtures/cb2.php" "$DOC/cb2.php"
cp "$H/fixtures/cbE.php" "$DOC/cbE.php"

# ---- gamba A: sentinella ESTESA (con mode-probe interno) ----
step "gamba A: sentinella estesa sul pin, modo=$MODE"
if PHPSRV="$PHPSRV" OUTDIR="$OUT/sentinella" "$SENTINEL" >> "$OUT/sentinella.txt" 2>&1; then
  step "sentinella: PASS (con mode-probe)"
else
  step "sentinella: FAIL — vedi sentinella.txt"; FAILS=$((FAILS+1))
fi

# ---- oracle di riferimento php -S: cb1 ×2, cb2 ×2, cbE ×2 (stesso DOC) ----
ORPORT=8212
pkill -f "127.0.0.1:$ORPORT" 2>/dev/null || true; sleep 0.5
( "$ORACLE" -S 127.0.0.1:$ORPORT -t "$DOC" >/dev/null 2>&1 ) &
ORPID=$!
sleep 1
for f in cb1 cb2 cbE; do
  curl -s -m 5 "http://127.0.0.1:$ORPORT/$f.php" > "$OUT/$f-oracle-1.out"
  curl -s -m 5 "http://127.0.0.1:$ORPORT/$f.php" > "$OUT/$f-oracle-2.out"
  if ! cmp -s "$OUT/$f-oracle-1.out" "$OUT/$f-oracle-2.out"; then
    step "$f: VOID — l'oracle stesso non è byte-id tra 2 richieste"; FAILS=$((FAILS+1))
  fi
done
kill $ORPID 2>/dev/null || true
pkill -f "127.0.0.1:$ORPORT" 2>/dev/null || true

# ---- gamba B: dente CAPTURE-BOUNDARY cb1 (invariato da S-102) ----
CBPORT=8211
pkill -f "127.0.0.1:$CBPORT" 2>/dev/null || true; sleep 0.5
( PHPR_DUMP_OPS=1 "$PHPSRV" --axum --workers 1 --port $CBPORT -t "$DOC" >/dev/null 2>"$OUT/cb-srv.log" ) &
SRV=$!
sleep 2
for r in 1 2 3; do
  curl -s -m 5 "http://127.0.0.1:$CBPORT/cb1.php" > "$OUT/cb1-$r.out"
done
# gamba C sulle STESSE assi (stesso server workers=1): cb2 ×2
for r in 1 2; do
  curl -s -m 5 "http://127.0.0.1:$CBPORT/cb2.php" > "$OUT/cb2-$r.out"
done
# gamba E sulle stesse assi: cbE poi cb2 (stesso worker per costruzione)
curl -s -m 5 "http://127.0.0.1:$CBPORT/cbE.php" > "$OUT/cbE-1.out"
curl -s -m 5 "http://127.0.0.1:$CBPORT/cb2.php" > "$OUT/cb2-postE.out"
kill $SRV 2>/dev/null || true
pkill -f "127.0.0.1:$CBPORT" 2>/dev/null || true
sleep 0.3

# B1: marker cb1 tutti presenti nella PRIMA risposta
for m in 'CB1:begin' 'CB1:sum=4950' 'DTOR:mid:x=50' 'CB1:req=1' 'CB1:body-done' \
         'SHUTDOWN:1' 'DTOR:shut:x=7' 'DTOR:bag' 'DTOR:inbag:x=9' 'DTOR:end:x=3'; do
  if ! grep -qF "$m" "$OUT/cb1-1.out"; then
    step "cb1: FAIL marker assente [$m]"; FAILS=$((FAILS+1))
  fi
done
# B2: byte-id 1ª↔2ª↔3ª sullo stesso worker
for r in 2 3; do
  if ! cmp -s "$OUT/cb1-1.out" "$OUT/cb1-$r.out"; then
    step "cb1: FAIL richiesta $r NON byte-id alla prima"; diff "$OUT/cb1-1.out" "$OUT/cb1-$r.out" | head -5; FAILS=$((FAILS+1))
  fi
done
# B3: corpo identico all'oracle
if ! cmp -s "$OUT/cb1-1.out" "$OUT/cb1-oracle-1.out"; then
  step "cb1: FAIL corpo DIVERSO dall'oracle"; diff "$OUT/cb1-oracle-1.out" "$OUT/cb1-1.out" | head -10; FAILS=$((FAILS+1))
else
  step "cb1: corpo identico all'oracle"
fi

# ---- gamba C: WARNING-LINE cb2 (KS-PE-104-1) ----
# C1: byte-id tra le 2 richieste
if ! cmp -s "$OUT/cb2-1.out" "$OUT/cb2-2.out"; then
  step "cb2: FAIL 2ª richiesta NON byte-id"; FAILS=$((FAILS+1))
fi
# C2: corpo identico all'oracle (riga inclusa, per costruzione)
if ! cmp -s "$OUT/cb2-1.out" "$OUT/cb2-oracle-1.out"; then
  step "cb2: FAIL corpo DIVERSO dall'oracle"; diff "$OUT/cb2-oracle-1.out" "$OUT/cb2-1.out" | head -10; FAILS=$((FAILS+1))
else
  step "cb2: corpo identico all'oracle"
fi
# C3: la proprietà giudicata è NOMINATA — 2 warning esatti, righe uguali
#     all'oracle (il braccio morde anche se un giorno il byte-id passasse
#     per vie diverse: la RIGA è il giudice dichiarato del §3.13)
WL_PIN="$(grep -oE 'on line [0-9]+' "$OUT/cb2-1.out" | tr '\n' ';')"
WL_ORA="$(grep -oE 'on line [0-9]+' "$OUT/cb2-oracle-1.out" | tr '\n' ';')"
NW_PIN="$(grep -cE 'Warning: Undefined property' "$OUT/cb2-1.out")"
if [ "$NW_PIN" != 2 ] || [ -z "$WL_ORA" ] || [ "$WL_PIN" != "$WL_ORA" ]; then
  step "cb2: FAIL warning-line — pin [$WL_PIN] n=$NW_PIN vs oracle [$WL_ORA] n=2"; FAILS=$((FAILS+1))
else
  step "cb2: warning-line OK (2 warning, righe = oracle: $WL_ORA)"
fi
# C4: mode-probe dedicato sul dump dell'unità cb2.php (KS-PE-103-3)
case "$REG" in 0) EXPECT_ON=0 ;; *) EXPECT_ON=1 ;; esac
CBCHUNK="$(tr -d '\0' < "$OUT/cb-srv.log" | awk '/^== unit .*\/cb2\.php/{f=1;next} /^== unit /{f=0} f')"
if [ -z "$CBCHUNK" ]; then
  step "cb2: FAIL mode-probe — nessun dump dell'unità cb2.php nel log"; FAILS=$((FAILS+1))
else
  HAS_REG=0
  printf '%s' "$CBCHUNK" | grep -qE "BinarySS|BinarySC|BinaryDst|CmpJmpSC|CmpJmpSS" && HAS_REG=1
  if [ "$HAS_REG" != "$EXPECT_ON" ]; then
    step "cb2: FAIL mode-probe — forme registro=$HAS_REG ma modo atteso on=$EXPECT_ON"; FAILS=$((FAILS+1))
  else
    step "cb2: mode-probe OK (on=$EXPECT_ON provato dal dump dell'unità)"
  fi
fi

# ---- gamba E: ERRORE-POI-SUCCESSO ----
if ! cmp -s "$OUT/cbE-1.out" "$OUT/cbE-oracle-1.out"; then
  step "cbE: FAIL corpo DIVERSO dall'oracle"; diff "$OUT/cbE-oracle-1.out" "$OUT/cbE-1.out" | head -10; FAILS=$((FAILS+1))
else
  step "cbE: corpo identico all'oracle"
fi
if ! cmp -s "$OUT/cb2-postE.out" "$OUT/cb2-1.out"; then
  step "cbE: FAIL — cb2 DOPO l'errore diverge dalla referenza (stato pendente sopravvissuto?)"; diff "$OUT/cb2-1.out" "$OUT/cb2-postE.out" | head -10; FAILS=$((FAILS+1))
else
  step "cbE: errore-poi-successo OK (cb2 post-errore byte-id alla referenza)"
fi

# ---- gamba D: INTERLEAVING CROSS-FIXTURE workers=2 ----
ILPORT=8213
pkill -f "127.0.0.1:$ILPORT" 2>/dev/null || true; sleep 0.5
( "$PHPSRV" --axum --workers 2 --port $ILPORT -t "$DOC" >/dev/null 2>"$OUT/il-srv.log" ) &
ILSRV=$!
sleep 2
ILFAIL=0
for r in 1 2 3 4 5 6; do
  curl -s -m 5 "http://127.0.0.1:$ILPORT/cb1.php" > "$OUT/il-cb1-$r.out"
  curl -s -m 5 "http://127.0.0.1:$ILPORT/cb2.php" > "$OUT/il-cb2-$r.out"
  cmp -s "$OUT/il-cb1-$r.out" "$OUT/cb1-1.out" || { step "interleave: FAIL cb1 round $r diverge"; ILFAIL=1; FAILS=$((FAILS+1)); }
  cmp -s "$OUT/il-cb2-$r.out" "$OUT/cb2-1.out" || { step "interleave: FAIL cb2 round $r diverge"; ILFAIL=1; FAILS=$((FAILS+1)); }
done
[ "$ILFAIL" = 0 ] && step "interleave workers=2: OK (6×cb1 + 6×cb2 byte-id alle referenze)"
kill $ILSRV 2>/dev/null || true
pkill -f "127.0.0.1:$ILPORT" 2>/dev/null || true
sleep 0.3
rm -rf "$DOC"

# ---- cross-mode (quando esiste l'altro braccio) ----
OTHER=off; [ "$MODE" = off ] && OTHER=on
if [ -f "$H/collaudo-out-$OTHER/cb1-1.out" ]; then
  for f in cb1-1 cb2-1 cbE-1; do
    if cmp -s "$OUT/$f.out" "$H/collaudo-out-$OTHER/$f.out"; then
      step "cross-mode: $f byte-id tra i due modi"
    else
      step "cross-mode: FAIL — $f DIVERSA tra i due modi"; FAILS=$((FAILS+1))
    fi
  done
else
  step "cross-mode: altro braccio non ancora eseguito (verrà giudicato lì)"
fi

step "S103-COLLAUDO-$MODE DONE fails=$FAILS"
echo "rc=$FAILS modo=$MODE $(date +%T)" > "$OUT/collaudo.done"
exit "$FAILS"
