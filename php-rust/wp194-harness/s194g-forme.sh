#!/bin/bash
# s194g-forme.sh <tag> — S-194 C5 (criterion s194-criterio-C.md p.26): pin-recipe arm build of the current tree (same recipe as
# s194-passo.sh), then retired instructions per iteration on the call-shape fixtures s194g-forme/*.php, arm vs the N3 arm of the
# bisection. Outcome: s194g-forme-<tag>.out, out/s194g-forme-<tag>.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")" && pwd)"; T="$1"; ARM="$HOME/Claude/phpr-target/s192-armD"; V="$W/s194g-forme-$T.out"; : > "$V"; rm -f "$W/out/s194g-forme-$T.done"
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
for i in $(seq 1 240); do
  if [ "$(tail -1 "$FEED" | cut -d' ' -f1)" = "DONE" ] && ! pgrep -x cargo >/dev/null; then break; fi
  sleep 15
done
cd "$W/.." || exit 2
CARGO_TARGET_DIR="$ARM" CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed cargo build --release -p php-cli > "$W/out/s194g-forme-$T-braccio.log" 2>&1
b=$?; echo "braccio: rc=$b sha=$(shasum -a 256 "$ARM/release/phpr" | cut -c1-16)" >> "$V"
if [ "$b" != 0 ]; then grep -E -A12 '^(error|warning: unused)' "$W/out/s194g-forme-$T-braccio.log" | cut -c1-240 | sed -n '1,60p' >> "$V"; echo 1 > "$W/out/s194g-forme-$T.done"; exit 1; fi
# binaries on a path WITHOUT spaces (the pairs below are split by the shell)
B=/private/tmp/phpr-s194c-bin; cp "$ARM/release/phpr" "$B/phpr-forme-$T"; cp "$W/out/phpr-bis-f1-6b5c0095" "$B/phpr-bis-N3"; REF="$B/phpr-bis-N3"
for f in empty lit var1 var1x var2; do
  for p in "N3 $REF" "$T $B/phpr-forme-$T"; do
    set -- $p
    /usr/bin/time -l "$2" "$W/s194g-forme/$f.php" > /dev/null 2> "$W/out/s194g-t.err"
    [ -s "$W/out/s194g-t.err" ] && grep -q 'instructions retired' "$W/out/s194g-t.err" || { echo "conto mancante $f $1" >> "$V"; echo 7 > "$W/out/s194g-forme-$T.done"; exit 7; }
    i=$(grep 'instructions retired' "$W/out/s194g-t.err" | tr -s ' ' | cut -d' ' -f2); c=$(grep 'cycles elapsed' "$W/out/s194g-t.err" | tr -s ' ' | cut -d' ' -f2)
    echo "$f $1 istr/iter $(python3 -c "print('%.1f' % ($i/2e7))") cicli/iter $(python3 -c "print('%.1f' % ($c/2e7))")" >> "$V"
  done
done
echo "fine $(date '+%F %T')" >> "$V"; echo 0 > "$W/out/s194g-forme-$T.done"
