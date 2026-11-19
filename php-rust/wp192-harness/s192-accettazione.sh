#!/bin/bash
# s192-accettazione.sh <tag> — S-192 ter: accettazione di giorno di una leva (criterio s192-criterio-hasher.md p.4), in sequenza,
# un solo cargo alla volta: (1) build dev-release, (2) catena S-190 INVARIATA (gate di giorno, loc_dente, sentinelle, leave_dtor,
# corpus-gate ×2), (3) batteria `cargo test --release` (rc dal comando). Esiti: out/accettazione-<tag>.out (una riga per passo),
# out/accettazione-<tag>.rc (0 = tutto verde). Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; O="$P/wp192-harness/out"; TAG="${1:?tag}"
V="$O/accettazione-$TAG.out"; mkdir -p "$O"; : > "$V"; rm -f "$O/accettazione-$TAG.rc"; RC=0
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
cd "$P" || exit 2
until tail -1 "$FEED" | grep -q '^DONE' && ! pgrep -x cargo >/dev/null; do sleep 30; done
cargo build --profile dev-release > "$O/accettazione-$TAG-build.log" 2>&1; r=$?; echo "build dev-release: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
if [ "$RC" = 0 ]; then
  bash "$P/wp190-harness/s190-c2-catena.sh" "$TAG" > "$O/accettazione-$TAG-catena.log" 2>&1; r=$?
  echo "catena S-190: rc=$r (wp190-harness/s190-c2-catena-$TAG.out)" >> "$V"; [ "$r" = 0 ] || RC=1
  cargo test --release > "$O/accettazione-$TAG-batteria.log" 2>&1; r=$?
  P_=$(grep -E '^test result' "$O/accettazione-$TAG-batteria.log" | awk '{p+=$4; f+=$6; i+=$8} END{print p"/"f"/"i}')
  echo "batteria cargo test --release: rc=$r (passati/falliti/ignorati $P_)" >> "$V"; [ "$r" = 0 ] || RC=1
fi
echo "esito rc=$RC $(date '+%F %T')" >> "$V"; echo "$RC" > "$O/accettazione-$TAG.rc"; exit "$RC"
