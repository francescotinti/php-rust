#!/bin/bash
# s100-parity-server.sh — S-100 punto 1 (ordine Concilio WP-101): launcher
# parità server BIMODALE (A-KL-101-6, A-PE-101-4).
#
# USO: s100-parity-server.sh <off|on>
#   Il MODO è un PARAMETRO OBBLIGATORIO dalla lista chiusa {off, on} e viene
#   speso come PHPR_REG_LOWER=0|1 ESPLICITO nell'ambiente costruito.
#
# COMMENTO-GARANZIA (riscritto per A-PE-101-4 — il vecchio «PHPR_REG_LOWER è
# ASSENTE per costruzione» garantiva flag-OFF solo PRIMA del contratto di
# modo e sarebbe diventato una garanzia del modo NUOVO dopo il flip del
# default): il modo register-lowering di ogni processo lanciato da questo
# script è quello del parametro, per VALORE ESPLICITO secondo la grammatica
# value-parsed del contratto (reg_lower.rs: assente=>DEFAULT_ON, `=1`=>on,
# `=0`=>off, altro=>default+warning). Nessuna gamba di questo script dipende
# dal default di build: il launcher resta corretto PRIMA e DOPO il flip.
#
#   gamba A: sentinella output-capture (G-APERTURA-2 axum, due richieste
#            byte-identiche) sul pin php-server — hash FAIL-CLOSED.
#   gamba B: WP gruppo option (413) + gruppo restapi (3508) per NOME,
#            oracle ↔ phpr, junit names diff (ricetta regate43, WP-43).
# AMBIENTE COSTRUITO, MAI SOTTRATTO (A-SK-93): re-exec via `env -i` con la
# sola lista chiusa qui sotto (PHPR_REG_LOWER incluso, dal parametro).
set -u

# ---- parametro di modo (lista chiusa) ----
case "${1:-}" in
  off) REG=0 ;;
  on)  REG=1 ;;
  *) echo "USO: $0 <off|on> — modo register-lowering ESPLICITO (A-KL-101-6)"; exit 64 ;;
esac
MODE="$1"

# ---- lista chiusa (l'ambiente si costruisce) ----
ENV_PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
ENV_HOME=/Users/francescotinti
ENV_TMPDIR=/tmp
ALLOWED='PATH|HOME|TMPDIR|LC_ALL|MIMALLOC_PURGE_DELAY|PHPR_REG_LOWER|S100_PARITY_REEXEC|PWD|SHLVL|_'
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
  if [ "${S100_PARITY_REEXEC:-0}" != 0 ]; then
    echo "REFUSE s100-parity-server: ambiente ancora sporco dopo il re-exec — nomi extra: [${EXTRA:-none}]"; exit 65
  fi
  exec /usr/bin/env -i \
    PATH="$ENV_PATH" HOME="$ENV_HOME" TMPDIR="$ENV_TMPDIR" LC_ALL=C \
    MIMALLOC_PURGE_DELAY=0 PHPR_REG_LOWER="$REG" S100_PARITY_REEXEC=1 \
    /bin/bash "$0" ${@+"$@"}
fi
unset S100_PARITY_REEXEC

# ---- pin e strumenti ----
# Registro = PIN_REGISTRY.md (repo root); il collaudo è GRADUATO per gamba e
# per MODO (A-PE-101-1): questo run copre la gamba dichiarata nel .done.
# Ricetta pin server: cargo build --release -p php-server --features axum-server
# AGGIORNARE PIN_SRV_ATTESO a ogni rotazione (mai pin effetto-collaterale).
# Pin S-100 POST-FIX emit_binary (A-HO-102-1, ricetta axum-server)
PIN_SRV_ATTESO=62b978c51c62e108
PHPSRV="$HOME/Claude/php-rust-output/release/php-server"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
ORACLE=/opt/homebrew/opt/php/bin/php
WPDEV="$HOME/Claude/wpdev"
XJ="/Volumes/Extreme Pro/Claude/wp16-harness/extract-junit.pl"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
# A-PE-101-3: sentinella ESTESA (N=16 interleaved su 3 endpoint + burst
# concorrente, workers=2) — la 2-req di wp77 resta dentro come endpoint g2.
SENTINEL="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp100-harness/s100-sentinella-estesa.sh"
OUT="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp100-harness/parity-out-$MODE"
mkdir -p "$OUT"
step() { echo "== $(date +%H:%M:%S) [S100-PARITY-$MODE] $1" | tee -a "$OUT/progress.txt"; }
FAILS=0

# ---- gamba 0: hash del pin, FAIL-CLOSED (KS-PE-100-3) ----
H_SRV="$(shasum -a 256 "$PHPSRV" | cut -c1-16)"
H_PHPR="$(shasum -a 256 "$PHPR" | cut -c1-16)"
step "modo=$MODE (PHPR_REG_LOWER=$REG esplicito) hash php-server=$H_SRV (atteso $PIN_SRV_ATTESO) phpr=$H_PHPR (churna col relink: registrato, non gate)"
if [ "$H_SRV" != "$PIN_SRV_ATTESO" ]; then
  step "REFUSE: il binario php-server NON è il pin da collaudare"; echo "rc=65 $(date +%T)" > "$OUT/parity.done"; exit 65
fi

# ---- gamba A: sentinella ESTESA sul pin ----
step "sentinella estesa (16 interleaved + 4 concorrenti, workers=2) sul pin, modo=$MODE"
if PHPSRV="$PHPSRV" OUTDIR="$OUT" "$SENTINEL" >> "$OUT/sentinella.txt" 2>&1; then
  step "sentinella: PASS (byte-identiche)"
else
  step "sentinella: FAIL — vedi sentinella.txt"; FAILS=$((FAILS+1))
fi

# ---- gamba B: option + restapi per NOME (uploads sotto guard, WP-62) ----
"$GUARD" backup-wipe >> "$OUT/progress.txt" 2>&1 || { step "ABORT: backup uploads fallito"; echo "rc=66 $(date +%T)" > "$OUT/parity.done"; exit 66; }
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
  # A-KL-101-3: assert conteggi↔nomi — i nomi estratti devono quadrare col
  # conteggio dichiarato dallo strumento (junit testsuite tests=N).
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
step "S100-PARITY-$MODE DONE fails=$FAILS"
echo "rc=$FAILS modo=$MODE $(date +%T)" > "$OUT/parity.done"
exit "$FAILS"
