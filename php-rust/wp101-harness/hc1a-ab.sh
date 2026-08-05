#!/bin/bash
# hc1a-ab.sh — misura DA SOLA di H-C1a: A = binario H-C1a, B = pin S-100
# (stash). Bracci INTERLEAVED (ABAB...) per equalizzare i burst di rumore.
set -u
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
A="${A:-$HOME/Claude/php-rust-output/release/phpr}"
B="${B:-/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s100-fix}"
P="$H/../wp97-harness/micro/prop.php"
E="$H/../wp97-harness/micro/empty.php"
R="${R:-7}"
u() { { /usr/bin/time -p "$1" "$2" >/dev/null; } 2>&1 | awk '/^user/{print $2}'; }
echo "A=$(shasum -a 256 "$A" | cut -c1-16)  B=$(shasum -a 256 "$B" | cut -c1-16)  R=$R interleaved"
fa=(); fb=()
for i in $(seq 1 3); do fa+=("$(u "$A" "$E")"); fb+=("$(u "$B" "$E")"); done
echo "floorA: ${fa[*]}"; echo "floorB: ${fb[*]}"
ta=(); tb=()
for i in $(seq 1 "$R"); do
  ta+=("$(u "$A" "$P")")
  tb+=("$(u "$B" "$P")")
done
echo "A_prop: ${ta[*]}"
echo "B_prop: ${tb[*]}"
