#!/bin/bash
# prova.sh <tag> <ref> — S-194: quick reading of a candidate BEFORE paying the whole chain: LOC tooth, pin-recipe arm build of the
# tree, one ORM leg (instructions) against the already measured arm out/phpr-<ref>. No acceptance, no commit value: direction only.
# Outcome: out/prova-<tag>.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; T="$1"; R="$2"; D="$W/out/prova-$T.done"; : > "$D.tmp"; rm -f "$D"
lc() { echo "$1" | tr 'A-Z' 'a-z'; }
ARMD="$HOME/Claude/phpr-target/s192-armD"; FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
until tail -1 "$FEED" | grep -q '^DONE' && ! pgrep -x cargo >/dev/null; do sleep 20; done
cd "$W/.." || exit 2
cargo test --profile dev-release -p php-runtime --test loc_dente > "$W/out/prova-$T-loc.log" 2>&1; echo "loc_dente: rc=$?" >> "$D.tmp"
CARGO_TARGET_DIR="$ARMD" CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed cargo build --release -p php-cli > "$W/out/prova-$T-braccio.log" 2>&1
b=$?; echo "braccio: rc=$b" >> "$D.tmp"; [ "$b" = 0 ] || { mv "$D.tmp" "$D"; exit 1; }
cp "$ARMD/release/phpr" "$W/out/phpr-$(lc "$T")"
REPS=1 python3 "$W/s194-orm-istruzioni.py" "$R=$W/out/phpr-$(lc "$R")" "$T=$W/out/phpr-$(lc "$T")" > "$W/out/prova-$T-orm.log" 2>&1; echo "orm: rc=$?" >> "$D.tmp"
grep -E "^($R|$T) " "$W/s194-orm-istruzioni.out" | tail -2 | cut -c1-90 >> "$D.tmp"
mv "$D.tmp" "$D"
