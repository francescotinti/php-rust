#!/bin/bash
# s194-passo-P1.sh <tag> — S-194 package P, one measured step (criterion s194-criterio-P.md p.4): the WHOLE primitive probe on the pin
# (base reading, only if s194-primitive-Pbase.out is missing), pin-recipe arm build of the current tree, the whole probe on the arm
# against that base. Same recipe as s194-passo-C3b.sh; differences: whole probe instead of a fixture list, base measured here.
# Outcome: s194-passo-<tag>.out, out/passo-<tag>.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")" && pwd)"; T="$1"; ARM="$HOME/Claude/phpr-target/s192-armD"; V="$W/s194-passo-$T.out"; : > "$V"; rm -f "$W/out/passo-$T.done"
if [ ! -s "$W/s194-primitive-Pbase.out" ]; then bash "$W/s194-primitive.sh" Pbase; echo "base: rc=$(cat "$W/s194-primitive-Pbase.rc")" >> "$V"; fi
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
for i in $(seq 1 240); do
  if [ "$(tail -1 "$FEED" | cut -d' ' -f1)" = "DONE" ] && ! pgrep -x cargo >/dev/null; then break; fi
  sleep 15
done
cd "$W/.." || exit 2
CARGO_TARGET_DIR="$ARM" CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed cargo build --release -p php-cli > "$W/out/passo-$T-braccio.log" 2>&1
b=$?; echo "braccio: rc=$b sha=$(shasum -a 256 "$ARM/release/phpr" | cut -c1-16)" >> "$V"
if [ "$b" != 0 ]; then echo 1 > "$W/out/passo-$T.done"; exit 1; fi
bash "$W/s194-primitive.sh" "$T" --bin "$ARM/release/phpr" --base s194-primitive-Pbase.out
echo "sonda: rc=$(cat "$W/s194-primitive-$T.rc")" >> "$V"
echo "fine $(date '+%F %T')" >> "$V"; echo 0 > "$W/out/passo-$T.done"
