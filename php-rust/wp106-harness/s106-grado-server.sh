#!/bin/bash
# s106-grado-server.sh — S-106 punto 2 (A-PE-107-1, Concilio WP-107): grado
# PIENO del pin php-server de67cb64 — PRIMO ATTO di S-106, PRIMA di ogni
# cifra server e di ogni build (KS-PE-107-3: seconda proroga = pin decade).
#
# USO: s106-grado-server.sh <off|on>
#
# LETTERA A-PE-107-1 implementata:
#   - collaudo sul binario STASHATO `php-server-s105` (hash de67cb64
#     riverificato da file PIN_SRV_ATTESO.txt, FAIL-CLOSED);
#   - pin phpr d4d0fa52 riverificato (PIN_PHPR_ATTESO.txt, FAIL-CLOSED —
#     è il binario che esercita le gambe phpunit);
#   - option 413 + restapi 3508 per NOME (junit names diff, assert
#     conteggi↔nomi KS-KL-101-3), env -i, 2 modi espliciti;
#   - mode-probe: dentro la sentinella estesa (A-PE-102-1, dump unità).
# Derivato da s100-parity-server.sh; PIN da file come s103 (mai hardcoded).
# AMBIENTE COSTRUITO, MAI SOTTRATTO (A-SK-93): re-exec env -i lista chiusa.
# Niente build dentro il collaudo (A-PE-103-3): FAIL-CLOSED sui pin.
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
ALLOWED='PATH|HOME|TMPDIR|LC_ALL|MIMALLOC_PURGE_DELAY|PHPR_REG_LOWER|S106_GRADO_REEXEC|PWD|SHLVL|_'
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
  if [ "${S106_GRADO_REEXEC:-0}" != 0 ]; then
    echo "REFUSE s106-grado-server: ambiente ancora sporco dopo il re-exec — nomi extra: [${EXTRA:-none}]"; exit 65
  fi
  exec /usr/bin/env -i \
    PATH="$ENV_PATH" HOME="$ENV_HOME" TMPDIR="$ENV_TMPDIR" LC_ALL=C \
    MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER="$REG" S106_GRADO_REEXEC=1 \
    /bin/bash "$0" ${@+"$@"}
fi
unset S106_GRADO_REEXEC

# ---- pin e strumenti ----
REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
H="$REPO/wp106-harness"
PIN_SRV_ATTESO="$(cat "$H/PIN_SRV_ATTESO.txt" 2>/dev/null | tr -d '[:space:]')"
PIN_PHPR_ATTESO="$(cat "$H/PIN_PHPR_ATTESO.txt" 2>/dev/null | tr -d '[:space:]')"
# lettera A-PE-107-1: il collaudo gira sul binario STASHATO
PHPSRV="/Volumes/Extreme Pro/Claude/phpr-old-target/release/php-server-s105"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
ORACLE=/opt/homebrew/opt/php/bin/php
WPDEV="$HOME/Claude/wpdev"
XJ="/Volumes/Extreme Pro/Claude/wp16-harness/extract-junit.pl"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
SENTINEL="$REPO/wp100-harness/s100-sentinella-estesa.sh"
OUT="$H/grado-out-$MODE"
mkdir -p "$OUT"
step() { echo "== $(date +%H:%M:%S) [S106-GRADO-$MODE] $1" | tee -a "$OUT/progress.txt"; }
FAILS=0

# ---- gamba 0: hash dei DUE pin, FAIL-CLOSED (KS-PE-100-3 + A-PE-107-1) ----
case "$PIN_SRV_ATTESO" in ""|PENDING*) echo "REFUSE: PIN_SRV_ATTESO.txt non valorizzato"; exit 65 ;; esac
case "$PIN_PHPR_ATTESO" in ""|PENDING*) echo "REFUSE: PIN_PHPR_ATTESO.txt non valorizzato"; exit 65 ;; esac
H_SRV="$(shasum -a 256 "$PHPSRV" | cut -c1-16)"
H_PHPR="$(shasum -a 256 "$PHPR" | cut -c1-16)"
step "modo=$MODE (PHPR_REG_LOWER=$REG esplicito) php-server-stash=$H_SRV (atteso $PIN_SRV_ATTESO) phpr=$H_PHPR (atteso $PIN_PHPR_ATTESO)"
if [ "$H_SRV" != "$PIN_SRV_ATTESO" ]; then
  step "REFUSE: lo stash php-server-s105 NON è il pin da gradare"; echo "rc=65 modo=$MODE $(date +%T)" > "$OUT/grado.done"; exit 65
fi
if [ "$H_PHPR" != "$PIN_PHPR_ATTESO" ]; then
  step "REFUSE: phpr NON è il pin firmatario PIN-106 (build intervenuta? KS-PE-107-1)"; echo "rc=65 modo=$MODE $(date +%T)" > "$OUT/grado.done"; exit 65
fi

# ---- gamba A: sentinella ESTESA sul pin stashato (con mode-probe A-PE-102-1) ----
step "gamba A: sentinella estesa (16 interleaved + 4 concorrenti, workers=2) sullo stash, modo=$MODE"
if PHPSRV="$PHPSRV" OUTDIR="$OUT/sentinella" "$SENTINEL" >> "$OUT/sentinella.txt" 2>&1; then
  step "sentinella: PASS (con mode-probe)"
else
  step "sentinella: FAIL — vedi sentinella.txt"; FAILS=$((FAILS+1))
fi

# ---- gamba B: option 413 + restapi 3508 per NOME (uploads sotto guard) ----
"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { step "ABORT: backup uploads fallito"; echo "rc=66 modo=$MODE $(date +%T)" > "$OUT/grado.done"; exit 66; }
restore_uploads() { "$GUARD" restore >> "$OUT/progress.txt" 2>&1; }
trap restore_uploads EXIT

run_group() { # $1=gruppo $2=timeout_phpr
  local g="$1" t="$2"
  step "gruppo $g: oracle"
  ( cd "$WPDEV" && mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" \
    && "$ORACLE" vendor/bin/phpunit --group "$g" --log-junit "$OUT/$g-oracle.junit.xml" ) > "$OUT/$g-oracle.txt" 2>&1
  tail -2 "$OUT/$g-oracle.txt" | tee -a "$OUT/progress.txt"
  step "gruppo $g: phpr (watchdog ${t}s, modo=$MODE)"
  ( cd "$WPDEV" && mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" \
    && "$WD" -t "$t" -o "$OUT" -- "$PHPR" vendor/bin/phpunit --group "$g" --log-junit "$OUT/$g-phpr.junit.xml" ) > "$OUT/$g-phpr.txt" 2>&1
  tail -2 "$OUT/$g-phpr.txt" | tee -a "$OUT/progress.txt"
  perl "$XJ" "$OUT/$g-oracle.junit.xml" | sort > "$OUT/$g-oracle.names"
  perl "$XJ" "$OUT/$g-phpr.junit.xml"   | sort > "$OUT/$g-phpr.names"
  # A-KL-101-3: assert conteggi↔nomi lato oracle (KS-KL-101-3)
  local n_decl n_extr
  n_decl="$(/usr/bin/sed -n 's/.*<testsuite[^>]*tests="\([0-9]*\)".*/\1/p' "$OUT/$g-oracle.junit.xml" | head -1)"
  n_extr="$(wc -l < "$OUT/$g-oracle.names" | tr -d ' ')"
  if [ -n "$n_decl" ] && [ "$n_decl" != "$n_extr" ]; then
    step "$g: GATE VOID — nomi estratti ($n_extr) != dichiarati ($n_decl) lato oracle (KS-KL-101-3)"; FAILS=$((FAILS+1))
  fi
  if diff "$OUT/$g-oracle.names" "$OUT/$g-phpr.names" > "$OUT/$g.diff"; then
    step "$g: IDENTICO per NOME ($n_extr nomi)"
  else
    step "$g: DIVERSO per NOME ($(wc -l < "$OUT/$g.diff" | tr -d ' ') righe di diff) — vedi $g.diff"; FAILS=$((FAILS+1))
  fi
}

run_group option 900
run_group restapi 2400

trap - EXIT
restore_uploads
step "S106-GRADO-$MODE DONE fails=$FAILS"
echo "rc=$FAILS modo=$MODE $(date +%T)" > "$OUT/grado.done"
exit "$FAILS"
