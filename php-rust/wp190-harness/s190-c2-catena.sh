#!/bin/bash
# s190-c2-catena.sh <tag> — S-190 C2, catena di accettazione di GIORNO (criterio p.5; dev-release, NON verdetto) sul binario
# già costruito: (1) gate di giorno (5 test ×2 modi + 9 fixture), (2) loc_dente, (3) sentinelle drop-order + leave_dtor,
# (4) corpus-gate ×2 modi. Esito riga per riga in s190-c2-catena-<tag>.out, rc complessivo in .rc (0 = tutto verde).
# Un solo cargo alla volta: i test cargo girano DOPO il gate e PRIMA del corpus-gate. Generato con assistenza AI (Claude Fable 5).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp190-harness"
TAG="${1:?tag}"; VERD="$H/s190-c2-catena-$TAG.out"; RCF="$H/s190-c2-catena-$TAG.rc"; LOG="$H/corpus-out/$TAG-catena.log"
RUNNER="$HOME/Claude/phpr-target/dev-output/dev-release/phpt-runner"; PHPR="$HOME/Claude/phpr-target/dev-output/dev-release/phpr"
mkdir -p "$H/corpus-out"; : > "$VERD"; rm -f "$RCF"; RC=0
note(){ echo "$1" | tee -a "$VERD"; }
note "== s190 catena C2 tag=$TAG git=$(git -C "$P" rev-parse --short HEAD) phpr=$(shasum -a 256 "$PHPR" | cut -c1-16) runner=$(shasum -a 256 "$RUNNER" | cut -c1-16) $(date '+%F %T') =="
cd "$P" || exit 2
bash "$H/s190-gate-giorno.sh" "$TAG" > "$LOG" 2>&1; r=$?; note "gate di giorno: rc=$r ($H/s190-gate-giorno-$TAG.out)"; [ "$r" = 0 ] || RC=1
cargo test --profile dev-release -p php-runtime --test loc_dente >> "$LOG" 2>&1; r=$?; note "loc_dente: rc=$r"; [ "$r" = 0 ] || RC=1
cargo test --profile dev-release -p php-runtime --lib drop_order_sentinel > "$H/corpus-out/$TAG-sentinelle.log" 2>&1; r=$?
note "sentinelle drop-order: rc=$r $(grep -E '^test result' "$H/corpus-out/$TAG-sentinelle.log" | tail -1 | cut -c1-80)"; [ "$r" = 0 ] || RC=1
cargo test --profile dev-release -p php-runtime --test leave_dtor_cmp_recursion > "$H/corpus-out/$TAG-leave.log" 2>&1; r=$?
note "leave_dtor_cmp_recursion: rc=$r $(grep -E '^test result' "$H/corpus-out/$TAG-leave.log" | tail -1 | cut -c1-80)"; [ "$r" = 0 ] || RC=1
# the cargo test runs above rebuild nothing of the runner: same hash required before the corpus-gate
[ "$(shasum -a 256 "$RUNNER" | cut -c1-16)" = "$(sed -n 's/.*runner=\([0-9a-f]*\).*/\1/p' "$VERD" | head -1)" ] || { note "FAIL: runner cambiato hash dopo i test"; RC=1; }
bash "$P/scripts/corpus-gate.sh" "$RUNNER" "$H/corpus-out/$TAG" > "$H/corpus-out/$TAG.log" 2>&1; r=$?
note "corpus-gate ×2: rc=$r"; sed 's/^/  /' "$H/corpus-out/$TAG/corpus-gate.out" | cut -c1-200 | tee -a "$VERD"; [ "$r" = 0 ] || RC=1
note "== esito catena rc=$RC $(date '+%F %T') =="; echo "$RC" > "$RCF"; exit "$RC"
