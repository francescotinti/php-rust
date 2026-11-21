#!/bin/bash
# s194-catena-dedicate.sh — S-194 dedicated strlen/count ops, verification chain (criterion s194-criterio-dedicate.md).
# Derived from s194-catena-dedicate.sh (manifest s194-catena-dedicate-copia.diff); the ONLY adaptations: tag, behaviour file,
# gate against the pin reference recorded BEFORE the code, bytecode check, base = the D-bis reading. Outcomes: s194-catena-dedicate.out, out/catena-dedicate.done.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; W="$P/wp194-harness"
ARM="$HOME/Claude/phpr-target/s192-armD"; V="$W/s194-catena-dedicate.out"; mkdir -p "$W/out"; : > "$V"; rm -f "$W/out/catena-dedicate.done"
cd "$P" || exit 2
fine() { echo "fine rc=$1 $(date '+%F %T')" >> "$V"; echo "$1" > "$W/out/catena-dedicate.done"; exit "$1"; }
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
for i in $(seq 1 240); do
  if [ "$(tail -1 "$FEED" | cut -d' ' -f1)" = "DONE" ] && ! pgrep -x cargo >/dev/null; then break; fi
  sleep 15
done
CARGO_TARGET_DIR="$ARM" CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed cargo build --release -p php-cli > "$W/out/catena-dedicate-braccio.log" 2>&1
b=$?; echo "braccio (ricetta del pin + tabelle di riga): rc=$b sha=$(shasum -a 256 "$ARM/release/phpr" | cut -c1-16)" >> "$V"
[ "$b" = 0 ] || fine 2
/opt/homebrew/opt/php/bin/php "$W/s194-dedicate-comportamento.php" > "$W/out/dedicate-comportamento.oracle" 2>&1
"$ARM/release/phpr" "$W/s194-dedicate-comportamento.php" > "$W/out/dedicate-comportamento.phpr" 2>&1
cmp -s "$W/out/dedicate-comportamento.pin" <("$HOME/Claude/php-rust-output/release/phpr" "$W/s194-dedicate-comportamento.php" 2>&1) || fine 4  # the reference recorded before the code still matches the pin
diff "$W/out/dedicate-comportamento.oracle" "$W/out/dedicate-comportamento.phpr" > "$W/out/dedicate-comportamento.diff"
echo "comportamento contro l'oracle (informativo: divergenze preesistenti se uguali al pin): diff $(wc -l < "$W/out/dedicate-comportamento.diff" | tr -d ' ') righe" >> "$V"
diff "$W/out/dedicate-comportamento.pin" "$W/out/dedicate-comportamento.phpr" > "$W/out/dedicate-comportamento-pin.diff"
c=$?; echo "comportamento contro il PIN s185 (gate: identico): rc=$c ($(wc -l < "$W/out/dedicate-comportamento.pin" | tr -d ' ') righe)" >> "$V"
[ "$c" = 0 ] || fine 3
n=0; for f in builtin_strlen builtin_count; do PHPR_DUMP_OPS=1 "$ARM/release/phpr" "$W/primitive/$f.php" 3 2> "$W/primitive/$f.ops" > /dev/null; n=$((n + $(awk '/^-- fn run /{p=1} p' "$W/primitive/$f.ops" | grep -c -E '^[0-9]+ (Strlen|Count) '))); done
echo "bytecode: op dedicati nei cicli strlen/count = $n (attesi >= 2)" >> "$V"; [ "$n" -ge 2 ] || fine 5
bash "$H/s192-accettazione.sh" s194ded > "$W/out/catena-dedicate-accettazione.log" 2>&1
a=$(cat "$H/out/accettazione-s194ded.rc" 2>/dev/null || echo 99); echo "accettazione s194ded: rc=$a" >> "$V"
[ "$a" = 0 ] || fine 1
bash "$W/s194-primitive.sh" ded --bin "$ARM/release/phpr" --base s194-primitive-Dbis.out
echo "sonda primitive ded: rc=$(cat "$W/s194-primitive-ded.rc")" >> "$V"
S194_BIN="$ARM/release/phpr" S194_SUF=-ded python3 "$W/s194-profilo-fixture.py" builtin_strlen 40000000 > "$W/out/catena-dedicate-profilo.log" 2>&1
echo "profilo builtin_strlen: rc=$?" >> "$V"
fine 0
