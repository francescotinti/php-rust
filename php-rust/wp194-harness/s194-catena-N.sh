#!/bin/bash
# s194-catena-N.sh — S-194 package N (namespace fallback resolved once), verification chain (criterion s194-criterio-N.md).
# Derived from the dedicated-ops chain (manifest s194-catena-N-copia.diff); the ONLY adaptations: tag, behaviour gates
# (N teeth identical to the ORACLE reference recorded before the code; D-bis and dedicated teeth identical to the pin),
# bytecode check, base = the dedicated-ops reading, profile of ns_strlen. Outcomes: s194-catena-N.out, out/catena-N.done.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; W="$P/wp194-harness"
ARM="$HOME/Claude/phpr-target/s192-armD"; V="$W/s194-catena-N.out"; mkdir -p "$W/out"; : > "$V"; rm -f "$W/out/catena-N.done"
cd "$P" || exit 2
fine() { echo "fine rc=$1 $(date '+%F %T')" >> "$V"; echo "$1" > "$W/out/catena-N.done"; exit "$1"; }
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
for i in $(seq 1 240); do
  if [ "$(tail -1 "$FEED" | cut -d' ' -f1)" = "DONE" ] && ! pgrep -x cargo >/dev/null; then break; fi
  sleep 15
done
CARGO_TARGET_DIR="$ARM" CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed cargo build --release -p php-cli > "$W/out/catena-N-braccio.log" 2>&1
b=$?; echo "braccio (ricetta del pin + tabelle di riga): rc=$b sha=$(shasum -a 256 "$ARM/release/phpr" | cut -c1-16)" >> "$V"
[ "$b" = 0 ] || fine 2
: > "$W/out/n-gate.txt"
/opt/homebrew/opt/php/bin/php "$W/s194-n-comportamento.php" > "$W/out/n-comportamento.oracle2" 2> /dev/null
"$ARM/release/phpr" "$W/s194-n-comportamento.php" > "$W/out/n-comportamento.phpr" 2> /dev/null
cmp -s "$W/out/n-comportamento.oracle" "$W/out/n-comportamento.oracle2" || fine 4  # the oracle reference recorded before the code is stable
diff "$W/out/n-comportamento.oracle" "$W/out/n-comportamento.phpr" > "$W/out/n-comportamento-oracle.diff"
c=$?; echo "comportamento N contro l'ORACLE (gate: identico): rc=$c ($(wc -l < "$W/out/n-comportamento.oracle" | tr -d ' ') righe, diff $(wc -l < "$W/out/n-comportamento-oracle.diff" | tr -d ' '))" >> "$V"
[ "$c" = 0 ] || fine 3
for t in dbis dedicate; do
  "$ARM/release/phpr" "$W/s194-$t-comportamento.php" > "$W/out/$t-comportamento.phprN" 2>&1
  diff "$W/out/$t-comportamento.pin" "$W/out/$t-comportamento.phprN" > "$W/out/$t-comportamento-pinN.diff"
  c=$?; echo "comportamento $t contro il PIN s185 (gate: identico): rc=$c" >> "$V"
  [ "$c" = 0 ] || fine 3
done
PHPR_DUMP_OPS=1 "$ARM/release/phpr" "$W/primitive/ns_strlen.php" 3 2> "$W/primitive/ns_strlen.ops" > /dev/null
n=$(awk '/^-- fn /{p=0} /^-- fn App.run /{p=1} p' "$W/primitive/ns_strlen.ops" | grep -c -E '^[0-9]+ CallNsBuiltin ')
echo "bytecode: CallNsBuiltin nella funzione di prova = $n (attesi >= 1)" >> "$V"; [ "$n" -ge 1 ] || fine 5
bash "$H/s192-accettazione.sh" s194N > "$W/out/catena-N-accettazione.log" 2>&1
a=$(cat "$H/out/accettazione-s194N.rc" 2>/dev/null || echo 99); echo "accettazione s194N: rc=$a" >> "$V"
[ "$a" = 0 ] || fine 1
bash "$W/s194-primitive.sh" N --bin "$ARM/release/phpr" --base s194-primitive-ded.out
echo "sonda primitive N: rc=$(cat "$W/s194-primitive-N.rc")" >> "$V"
S194_BIN="$ARM/release/phpr" S194_SUF=-N python3 "$W/s194-profilo-fixture.py" ns_strlen 40000000 > "$W/out/catena-N-profilo.log" 2>&1
echo "profilo ns_strlen: rc=$?" >> "$V"
fine 0
