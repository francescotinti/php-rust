#!/bin/bash
# s192-direzione-catena.sh <tag> — S-192 C2 (criterio p.5): attende la CI locale quieta, costruisce il binario RELEASE del tree
# su dev-output (ricetta `cargo build --release -p php-cli`, MAI la canonica) e lancia INVARIATO `wp190-harness/s190-direzione-k0.sh`
# con PIN = stash s183 e TREE = quel binario. DIREZIONE, non verdetto. Esiti: out/direzione-<tag>.rc (rc della direzione),
# out/build-rel-<tag>.rc, verdetto in wp190-harness/s190-direzione-k0-<tag>.out. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; O="$H/out"
TAG="${1:?tag}"; FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
mkdir -p "$O"; rm -f "$O/direzione-$TAG.rc" "$O/build-rel-$TAG.rc"
cd "$P" || exit 2
# CI quiet: last feed line is a DONE and no cargo / CI runner process is alive
until tail -1 "$FEED" | grep -q '^DONE' && ! pgrep -x cargo >/dev/null && ! pgrep -f 'ci-target/release/phpt-runner' >/dev/null; do sleep 30; done
echo "CI quieta $(date '+%F %T'): $(tail -1 "$FEED")" > "$O/direzione-$TAG.log"
cargo build --release -p php-cli >> "$O/build-rel-$TAG.log" 2>&1; r=$?; echo "$r" > "$O/build-rel-$TAG.rc"
[ "$r" = 0 ] || { echo 7 > "$O/direzione-$TAG.rc"; exit 7; }
TREE="$HOME/Claude/phpr-target/dev-output/release/phpr"
echo "tree release $(shasum -a 256 "$TREE" | cut -c1-16) git=$(git rev-parse --short HEAD) $(date '+%F %T')" >> "$O/direzione-$TAG.log"
PIN="/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s183" TREE="$TREE" R="${R:-3}" \
  bash "$P/wp190-harness/s190-direzione-k0.sh" "$TAG" >> "$O/direzione-$TAG.log" 2>&1; r=$?
echo "$r" > "$O/direzione-$TAG.rc"; exit "$r"
