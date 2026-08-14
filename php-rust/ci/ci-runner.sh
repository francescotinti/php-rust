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
# il hook post-receive ci spawna con l'ambiente di git: senza unset, ogni
# comando git qui dentro vedrebbe GIT_DIR=. (morso al primo collaudo)
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE
CI="/Volumes/Extreme Pro/Claude/phpr-ci"
Q="$CI/queue"; OUT="$CI/out"; WORK="$CI/work"
# target su disco LOCALE APFS (cargo su ExFAT = terreno minato: niente
# symlink/permessi) e PRUNE a fine job: build a freddo, ma il disco locale
# (budget pre-flight ≥15G) non tiene cache CI residenti.
export CARGO_TARGET_DIR="/private/tmp/phpr-ci-target"
LOCK="$CI/runner.lock"
FEED="$CI/CI_FEED.log"
MLOCK="/private/tmp/phpr-measure.lock"
mkdir -p "$Q" "$OUT"

# un solo runner alla volta: lock a directory con PID dentro; stale = pid MORTO,
# MAI a tempo (S-138: una coda lunga tiene il lock >6h legittimamente e la
# staleness a orologio ha prodotto 5 runner concorrenti). Rottura ATOMICA via
# mv: un solo breaker vince la rename, i perdenti NON toccano il lock nuovo.
acquire_lock() {
  while :; do
    if mkdir "$LOCK" 2>/dev/null; then
      echo $$ > "$LOCK/pid"
      return 0
    fi
    local pid; pid=$(cat "$LOCK/pid" 2>/dev/null)
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
      return 1                    # runner vivo legittimo: ci si fa da parte
    fi
    if [ -z "$pid" ] && [ -z "$(find "$LOCK" -maxdepth 0 -mmin +10 2>/dev/null)" ]; then
      return 1                    # pid non ancora scritto: finestra di nascita, non stale
    fi
    if mv "$LOCK" "$LOCK.stale.$$" 2>/dev/null; then
      rm -rf "$LOCK.stale.$$"     # orfano rimosso; si ritenta il mkdir
    else
      return 1                    # un altro breaker ha vinto la mv
    fi
  done
}
acquire_lock || exit 0
# cleanup SOLO se il lock è ancora nostro (un breaker può averlo sostituito)
trap '[ "$(cat "$LOCK/pid" 2>/dev/null)" = "$$" ] && rm -rf "$LOCK"' EXIT

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
fail_job() { # esito negativo precoce: status + DONE nel feed + notifica (morso: START senza DONE)
  echo "$1" > "$O/status"
  echo "DONE $S12 $1 $(date '+%F %T')" >> "$FEED"
  notify "$S12: $1"
}
# AppleDouble: il volume exFAT semina ._* anche dentro .git (._pack-*.idx
# avvelena il pack reader: "non-monotonic index" — morso al primo collaudo);
# vale per il bare E per il clone di lavoro.
dot_clean_git() {
  find "$CI/repo.git/objects" -name '._*' -delete 2>/dev/null
  [ -d "$WORK/.git" ] && find "$WORK/.git" -name '._*' -delete 2>/dev/null
}

while :; do
  # AppleDouble in coda: l'exFAT semina ._* anche in queue/ e un ._job letto
  # come SHA produce raffiche di checkout-fail (morso S-138/S-139)
  find "$Q" -name '._*' -delete 2>/dev/null
  JOB=$(ls -1 "$Q" 2>/dev/null | grep -v '^\._' | sort | head -1)
  [ -n "${JOB:-}" ] || break
  SHA=$(cat "$Q/$JOB" 2>/dev/null); rm -f "$Q/$JOB"
  [ -n "$SHA" ] || continue
  S12=${SHA:0:12}
  O="$OUT/$S12"; mkdir -p "$O"
  if ! quiet_wait; then
    fail_job "skipped-busy"
    continue
  fi
  # guardia disco: picco misurato del job ~4G (smoke 76544e8); sotto 8G il
  # commit TORNA IN CODA (lo riprova il runner del prossimo push) e si esce.
  FREE=$(df -g /private/tmp | awk 'NR==2{print $4}')
  if [ "${FREE:-0}" -lt 8 ]; then
    echo "$SHA" > "$Q/$(date +%s)-${S12}"
    echo "REQUEUE $S12 disk-low(${FREE}G) $(date '+%F %T')" >> "$FEED"
    notify "$S12: requeue disk-low(${FREE}G)"
    break
  fi
  echo "START $S12 $(date '+%F %T')" >> "$FEED"
  dot_clean_git
  if [ ! -d "$WORK/.git" ]; then
    git clone -q "$CI/repo.git" "$WORK" || { fail_job "clone-fail"; continue; }
  fi
  dot_clean_git
  git -C "$WORK" fetch -q origin || { fail_job "fetch-fail"; continue; }
  dot_clean_git
  git -C "$WORK" checkout -qf "$SHA" || { fail_job "checkout-fail"; continue; }
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
  rm -rf "$CARGO_TARGET_DIR"   # prune: il disco locale non tiene cache CI
done
