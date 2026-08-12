#!/bin/bash
# ci-runner.sh — CI LOCALE phpr (decisione utente 2026-08-12, S-132).
# Lanciato da launchd (com.phpr.ci: WatchPaths su queue/ + StartInterval 600).
# Consuma la coda UN commit alla volta: clone/checkout in phpr-ci/work,
# build release + batteria (cargo test) + corpus-gate, target dir SEPARATA
# (phpr-ci/target: MAI toccare php-rust-output, che porta i binari pinnati).
# Esiti in phpr-ci/out/<sha12>/status + riga in CI_FEED.log + notifica macOS.
# La CI è rete di ALLARME PRECOCE, non gate di record: pin e promozione
# restano sugli script canonici (REGOLE §2/§5/§6).
# Mutex con le misure: quiet_wait aspetta se ci sono orchestratori di misura
# vivi (pattern via file di script, non bash -c: niente auto-match) o se
# esiste /private/tmp/phpr-measure.lock (convenzione per le ricette future).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:"$HOME/.cargo/bin"
CI="/Volumes/Extreme Pro/Claude/phpr-ci"
Q="$CI/queue"; OUT="$CI/out"; WORK="$CI/work"
export CARGO_TARGET_DIR="$CI/target"
LOCK="$CI/runner.lock"
FEED="$CI/CI_FEED.log"
MLOCK="/private/tmp/phpr-measure.lock"
mkdir -p "$Q" "$OUT"

# un solo runner alla volta (mkdir atomico); lock orfano >6h = stale, si rompe
if ! mkdir "$LOCK" 2>/dev/null; then
  if [ -n "$(find "$LOCK" -maxdepth 0 -mmin +360 2>/dev/null)" ]; then
    rmdir "$LOCK" 2>/dev/null; mkdir "$LOCK" 2>/dev/null || exit 0
  else
    exit 0
  fi
fi
trap 'rmdir "$LOCK" 2>/dev/null' EXIT

quiet_wait() { # aspetta le misure (check ogni 30 s, max 4 h; poi rinuncia)
  local i=0 busy
  while :; do
    busy=""
    [ -e "$MLOCK" ] && busy="measure-lock"
    pgrep -f "pair109" > /dev/null 2>&1 && busy="pair109"
    pgrep -f "harness/s1[0-9][0-9]-" > /dev/null 2>&1 && busy="harness"
    pgrep -f "run-with-watchdog" > /dev/null 2>&1 && busy="watchdog"
    [ -z "$busy" ] && return 0
    i=$((i+1)); [ "$i" -gt 480 ] && return 1
    sleep 30
  done
}

notify() { osascript -e "display notification \"$1\" with title \"phpr CI\"" 2>/dev/null || true; }

while :; do
  JOB=$(ls -1 "$Q" 2>/dev/null | sort | head -1)
  [ -n "${JOB:-}" ] || break
  SHA=$(cat "$Q/$JOB" 2>/dev/null); rm -f "$Q/$JOB"
  [ -n "$SHA" ] || continue
  S12=${SHA:0:12}
  O="$OUT/$S12"; mkdir -p "$O"
  if ! quiet_wait; then
    echo "skipped-busy" > "$O/status"
    echo "SKIP $S12 misure in corso da >4h $(date '+%F %T')" >> "$FEED"
    continue
  fi
  echo "START $S12 $(date '+%F %T')" >> "$FEED"
  if [ ! -d "$WORK/.git" ]; then
    git clone -q "$CI/repo.git" "$WORK" || { echo "clone-fail" > "$O/status"; continue; }
  fi
  git -C "$WORK" fetch -q origin || { echo "fetch-fail" > "$O/status"; continue; }
  git -C "$WORK" checkout -qf "$SHA" || { echo "checkout-fail" > "$O/status"; continue; }
  SRC="$WORK/php-rust"
  ST="OK"
  ( cd "$SRC" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release ) > "$O/build.log" 2>&1
  rc=$?; echo "$rc" > "$O/build.rc"
  if [ "$rc" != 0 ]; then
    ST="build-FAIL"
  else
    ( cd "$SRC" && CARGO_INCREMENTAL=0 cargo test --release ) > "$O/batteria.log" 2>&1
    rc=$?; echo "$rc" > "$O/batteria.rc"
    if [ "$rc" != 0 ]; then
      ST="batteria-FAIL"
    else
      # corpus-gate del CHECKOUT (i riferimenti congelati restano quelli del
      # repo canonico: path cablato dentro lo script — dichiarato in ci/README)
      "$SRC/scripts/corpus-gate.sh" "$CARGO_TARGET_DIR/release/phpt-runner" "$O/corpus" > "$O/corpus.stdout" 2>&1
      rc=$?; echo "$rc" > "$O/corpus.rc"
      [ "$rc" != 0 ] && ST="corpus-FAIL"
    fi
  fi
  echo "$ST" > "$O/status"
  echo "DONE $S12 $ST $(date '+%F %T')" >> "$FEED"
  notify "$S12: $ST"
done
