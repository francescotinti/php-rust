#!/bin/bash
# s99-parity-server.sh — S-99.0 punto 1a (ordine Concilio WP-100, team-ordine §1a)
# Soddisfa KS-PE-99-1 / KS-PE-100-1/3: parità server PRIMA di ogni uso del server.
#   gamba A: sentinella output-capture (G-APERTURA-2 axum, due richieste
#            byte-identiche) sul pin php-server 365f4d4069513de3 — hash
#            verificato FAIL-CLOSED prima di servire una sola richiesta.
#   gamba B: WP gruppo option (413) + gruppo restapi (3508) per NOME,
#            oracle ↔ phpr, junit names diff (ricetta regate43, WP-43).
# AMBIENTE COSTRUITO, MAI SOTTRATTO (A-SK-93, lezione WP-96): re-exec via
# `env -i` con la sola lista chiusa qui sotto. PHPR_REG_LOWER è ASSENTE
# per costruzione (KS-PE-99-1), come ogni altro nome non elencato.
# La ricetta è uno SCRIPT perché servirà di nuovo al passo 4 (ri-parità
# dopo il sigillo eager — team-ordine §4).
set -u

# ---- lista chiusa (l'ambiente si costruisce) ----
ENV_PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
ENV_HOME=/Users/francescotinti
ENV_TMPDIR=/tmp
ALLOWED='PATH|HOME|TMPDIR|LC_ALL|MIMALLOC_PURGE_DELAY|S99_PARITY_REEXEC|PWD|SHLVL|_'
EXTRA="$(/usr/bin/env | /usr/bin/sed 's/=.*//' | /usr/bin/grep -vxE "$ALLOWED" | /usr/bin/tr '\n' ' ')"
SANE=1
[ -z "$EXTRA" ] || SANE=0
[ "${PATH:-}" = "$ENV_PATH" ] || SANE=0
[ "${HOME:-}" = "$ENV_HOME" ] || SANE=0
[ "${TMPDIR:-}" = "$ENV_TMPDIR" ] || SANE=0
[ "${LC_ALL:-}" = C ] || SANE=0
[ "${MIMALLOC_PURGE_DELAY:-}" = 0 ] || SANE=0
if [ "$SANE" != 1 ]; then
  if [ "${S99_PARITY_REEXEC:-0}" != 0 ]; then
    echo "REFUSE s99-parity-server: ambiente ancora sporco dopo il re-exec — nomi extra: [${EXTRA:-none}]"; exit 65
  fi
  exec /usr/bin/env -i \
    PATH="$ENV_PATH" HOME="$ENV_HOME" TMPDIR="$ENV_TMPDIR" LC_ALL=C \
    MIMALLOC_PURGE_DELAY=0 S99_PARITY_REEXEC=1 \
    /bin/bash "$0" ${@+"$@"}
fi
unset S99_PARITY_REEXEC

# ---- pin e strumenti ----
PIN_SRV_ATTESO=365f4d4069513de3
PHPSRV="$HOME/Claude/php-rust-output/release/php-server"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
ORACLE=/opt/homebrew/opt/php/bin/php
WPDEV="$HOME/Claude/wpdev"
XJ="/Volumes/Extreme Pro/Claude/wp16-harness/extract-junit.pl"
WD="/Volumes/Extreme Pro/Claude/wp13-harness/run-with-watchdog.sh"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
SENTINEL="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/wp77-harness/run_gate_g_apertura_2_axum.sh"
OUT="/Volumes/Extreme Pro/Claude/wp99-harness/parity-out"
mkdir -p "$OUT"
step() { echo "== $(date +%H:%M:%S) [S99-PARITY] $1" | tee -a "$OUT/progress.txt"; }
FAILS=0

# ---- gamba 0: hash del pin, FAIL-CLOSED (KS-PE-100-3) ----
H_SRV="$(shasum -a 256 "$PHPSRV" | cut -c1-16)"
H_PHPR="$(shasum -a 256 "$PHPR" | cut -c1-16)"
step "hash php-server=$H_SRV (atteso $PIN_SRV_ATTESO) phpr=$H_PHPR (churna col relink: registrato, non gate)"
if [ "$H_SRV" != "$PIN_SRV_ATTESO" ]; then
  step "REFUSE: il binario php-server NON è il pin da collaudare"; echo "rc=65 $(date +%T)" > "$OUT/parity.done"; exit 65
fi

# ---- gamba A: sentinella output-capture sul pin ----
step "sentinella output-capture (G-APERTURA-2 axum) sul pin"
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
  step "gruppo $g: phpr (watchdog ${t}s)"
  ( cd "$WPDEV" && mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" \
    && "$WD" -t "$t" -o "$OUT" -- "$PHPR" vendor/bin/phpunit --group "$g" --log-junit "$OUT/$g-phpr.junit.xml" ) > "$OUT/$g-phpr.txt" 2>&1
  tail -2 "$OUT/$g-phpr.txt" | tee -a "$OUT/progress.txt"
  perl "$XJ" "$OUT/$g-oracle.junit.xml" | sort > "$OUT/$g-oracle.names"
  perl "$XJ" "$OUT/$g-phpr.junit.xml"   | sort > "$OUT/$g-phpr.names"
  if diff "$OUT/$g-oracle.names" "$OUT/$g-phpr.names" > "$OUT/$g.diff"; then
    step "$g: IDENTICO per NOME ($(wc -l < "$OUT/$g-oracle.names" | tr -d ' ') nomi)"
  else
    step "$g: DIVERSO per NOME ($(wc -l < "$OUT/$g.diff" | tr -d ' ') righe di diff) — vedi $g.diff"; FAILS=$((FAILS+1))
  fi
}

run_group option 900
run_group restapi 2400

trap - EXIT
restore_uploads
step "S99-PARITY DONE fails=$FAILS"
echo "rc=$FAILS $(date +%T)" > "$OUT/parity.done"
exit "$FAILS"
