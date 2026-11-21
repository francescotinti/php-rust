#!/bin/bash
# s194-catena-Dbis.sh — S-194 package D-bis verification chain (criterion s194-criterio-Dbis.md), one heavy step at a time.
# Derived from s194-catena-D.sh (manifest s194-catena-Dbis-copia.diff); adaptations: tag Dbis, base = the D reading,
# arm built FIRST and a behaviour diff against the oracle before the acceptance. Outcomes: s194-catena-Dbis.out, out/catena-Dbis.done.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; W="$P/wp194-harness"
ARM="$HOME/Claude/phpr-target/s192-armD"; V="$W/s194-catena-Dbis.out"; mkdir -p "$W/out"; : > "$V"; rm -f "$W/out/catena-Dbis.done"
cd "$P" || exit 2
fine() { echo "fine rc=$1 $(date '+%F %T')" >> "$V"; echo "$1" > "$W/out/catena-Dbis.done"; exit "$1"; }
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
for i in $(seq 1 240); do
  if [ "$(tail -1 "$FEED" | cut -d' ' -f1)" = "DONE" ] && ! pgrep -x cargo >/dev/null; then break; fi
  sleep 15
done
CARGO_TARGET_DIR="$ARM" CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed cargo build --release -p php-cli > "$W/out/catena-Dbis-braccio.log" 2>&1
b=$?; echo "braccio (ricetta del pin + tabelle di riga): rc=$b sha=$(shasum -a 256 "$ARM/release/phpr" | cut -c1-16)" >> "$V"
[ "$b" = 0 ] || fine 2
/opt/homebrew/opt/php/bin/php "$W/s194-dbis-comportamento.php" > "$W/out/dbis-comportamento.oracle" 2>&1
"$ARM/release/phpr" "$W/s194-dbis-comportamento.php" > "$W/out/dbis-comportamento.phpr" 2>&1
"$HOME/Claude/php-rust-output/release/phpr" "$W/s194-dbis-comportamento.php" > "$W/out/dbis-comportamento.pin" 2>&1
diff "$W/out/dbis-comportamento.oracle" "$W/out/dbis-comportamento.phpr" > "$W/out/dbis-comportamento.diff"
echo "comportamento contro l'oracle (informativo: divergenze preesistenti se uguali al pin): diff $(wc -l < "$W/out/dbis-comportamento.diff" | tr -d ' ') righe" >> "$V"
diff "$W/out/dbis-comportamento.pin" "$W/out/dbis-comportamento.phpr" > "$W/out/dbis-comportamento-pin.diff"
c=$?; echo "comportamento contro il PIN s185 (gate: identico): rc=$c ($(wc -l < "$W/out/dbis-comportamento.pin" | tr -d ' ') righe)" >> "$V"
[ "$c" = 0 ] || fine 3
bash "$H/s192-accettazione.sh" s194Dbis > "$W/out/catena-Dbis-accettazione.log" 2>&1
a=$(cat "$H/out/accettazione-s194Dbis.rc" 2>/dev/null || echo 99); echo "accettazione s194Dbis: rc=$a" >> "$V"
[ "$a" = 0 ] || fine 1
bash "$W/s194-primitive.sh" Dbis --bin "$ARM/release/phpr" --base s194-primitive-D.out
echo "sonda primitive Dbis: rc=$(cat "$W/s194-primitive-Dbis.rc")" >> "$V"
S194_BIN="$ARM/release/phpr" S194_SUF=-Dbis python3 "$W/s194-profilo-fixture.py" builtin_strlen 40000000 > "$W/out/catena-Dbis-profilo.log" 2>&1
echo "profilo builtin_strlen: rc=$?" >> "$V"
fine 0
