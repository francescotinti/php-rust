#!/bin/bash
# s194-catena-D.sh — S-194 package D verification chain (criterion s194-criterio-D.md), one heavy step at a time:
# day acceptance, pin-recipe arm build (line tables), primitive probe against the base, line profile of the target fixture.
# Outcomes: s194-catena-D.out (one line per step), out/catena-D.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; W="$P/wp194-harness"
ARM="$HOME/Claude/phpr-target/s192-armD"; V="$W/s194-catena-D.out"; mkdir -p "$W/out"; : > "$V"; rm -f "$W/out/catena-D.done"
cd "$P" || exit 2
fine() { echo "fine rc=$1 $(date '+%F %T')" >> "$V"; echo "$1" > "$W/out/catena-D.done"; exit "$1"; }
bash "$H/s192-accettazione.sh" s194D > "$W/out/catena-D-accettazione.log" 2>&1
a=$(cat "$H/out/accettazione-s194D.rc" 2>/dev/null || echo 99); echo "accettazione s194D: rc=$a" >> "$V"
[ "$a" = 0 ] || fine 1
CARGO_TARGET_DIR="$ARM" CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed cargo build --release -p php-cli > "$W/out/catena-D-braccio.log" 2>&1
b=$?; echo "braccio (ricetta del pin + tabelle di riga): rc=$b sha=$(shasum -a 256 "$ARM/release/phpr" | cut -c1-16)" >> "$V"
[ "$b" = 0 ] || fine 2
bash "$W/s194-primitive.sh" D --bin "$ARM/release/phpr" --base s194-primitive-base.out
echo "sonda primitive D: rc=$(cat "$W/s194-primitive-D.rc")" >> "$V"
S194_BIN="$ARM/release/phpr" S194_SUF=-D python3 "$W/s194-profilo-fixture.py" builtin_strlen 40000000 > "$W/out/catena-D-profilo.log" 2>&1
echo "profilo builtin_strlen: rc=$?" >> "$V"
fine 0
