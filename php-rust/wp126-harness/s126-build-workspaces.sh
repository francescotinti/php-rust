#!/bin/bash
# s126-build-workspaces.sh — APPARATO mappa2: costruisce e CONGELA i workspace
# (dbal, http-foundation, collections, compoff) in wp9-harness/gates/ PRIMA del
# run di misura. Build con l'ORACLE; smoke bilaterale col lettore proprio
# (phpunit --version rc=0 su ENTRAMBI i motori) — collaudo-nell'atto.
# Rev del clone registrata in gates/s126-workspaces.identity.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"
ORACLE=/opt/homebrew/opt/php/bin/php
PHPR="$HOME/Claude/php-rust-output/release/phpr"
BUILD="${BUILD_SP:?BUILD_SP (workdir build) richiesto}"
LOG="$BUILD/build.log"; : > "$LOG"
IDF="$GATES/s126-workspaces.identity"; : > "$IDF"
fail(){ echo "BUILD FALLITA: $1" | tee -a "$LOG"; exit 1; }

mk(){ # DIR URL BRANCH(o '-' per default)
  local DIR="$1" URL="$2" BR="$3"
  rm -rf "$BUILD/$DIR"
  if [ "$BR" = "-" ]; then git clone --depth 1 "$URL" "$BUILD/$DIR" >> "$LOG" 2>&1 || fail "clone $DIR"
  else git clone --depth 1 -b "$BR" "$URL" "$BUILD/$DIR" >> "$LOG" 2>&1 || fail "clone $DIR ($BR)"; fi
  ( cd "$BUILD/$DIR" && COMPOSER_CACHE_DIR="$BUILD/$DIR/ccache" COMPOSER_HOME="$BUILD/$DIR/chome" \
      "$ORACLE" "$GATES/composer.phar" install --no-interaction --no-audit --no-progress >> "$LOG" 2>&1 ) || fail "composer $DIR"
  # smoke bilaterale (esito ESATTO rc=0, feedback forge-silent-failure)
  ( cd "$BUILD/$DIR" && "$ORACLE" vendor/bin/phpunit --version > /dev/null 2>&1 ) || fail "smoke oracle $DIR"
  ( cd "$BUILD/$DIR" && "$PHPR" vendor/bin/phpunit --version > /dev/null 2>&1 ) || fail "smoke phpr $DIR"
  echo "$DIR rev=$(git -C "$BUILD/$DIR" rev-parse --short=12 HEAD) branch=$BR" >> "$IDF"
  rm -rf "$BUILD/$DIR/.git"
  tar czf "$GATES/$DIR.tgz" -C "$BUILD" "$DIR" || fail "tar $DIR"
  echo "OK $DIR" | tee -a "$LOG"
}

mk dbal-work https://github.com/doctrine/dbal.git -
mk hf-work   https://github.com/symfony/http-foundation.git 7.4
mk coll-work https://github.com/doctrine/collections.git -

# compoff-work: composer.json+lock del clone dbal, cache CALDA, senza vendor.
rm -rf "$BUILD/compoff-work"; mkdir -p "$BUILD/compoff-work"
cp "$BUILD/dbal-work/composer.json" "$BUILD/dbal-work/composer.lock" "$BUILD/compoff-work/" 2>>"$LOG" || fail "compoff copia json/lock"
( cd "$BUILD/compoff-work" && COMPOSER_CACHE_DIR="$BUILD/compoff-work/ccache" COMPOSER_HOME="$BUILD/compoff-work/chome" \
    "$ORACLE" "$GATES/composer.phar" install --no-interaction --no-audit --no-progress >> "$LOG" 2>&1 ) || fail "compoff warm install"
# smoke OFFLINE col lettore proprio: reinstall a rete spenta deve riuscire (oracle)
( cd "$BUILD/compoff-work" && rm -rf vendor && \
  COMPOSER_DISABLE_NETWORK=1 COMPOSER_CACHE_DIR="$BUILD/compoff-work/ccache" COMPOSER_HOME="$BUILD/compoff-work/chome" \
  "$ORACLE" "$GATES/composer.phar" install --no-interaction --no-audit >> "$LOG" 2>&1 && [ -f vendor/autoload.php ] ) || fail "compoff smoke offline"
rm -rf "$BUILD/compoff-work/vendor"
echo "compoff-work base=dbal composer.lock $(shasum -a 256 "$BUILD/compoff-work/composer.lock" | cut -c1-16)" >> "$IDF"
tar czf "$GATES/compoff-work.tgz" -C "$BUILD" compoff-work || fail "tar compoff"
echo "OK compoff-work" | tee -a "$LOG"
cat "$IDF"
echo "BUILD COMPLETA"
