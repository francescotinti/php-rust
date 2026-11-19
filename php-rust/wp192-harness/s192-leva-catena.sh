#!/bin/bash
# s192-leva-catena.sh <tag> <commit-Z> <commit-L> — S-192: i tre giudici di una leva in sequenza, un solo cargo alla volta:
# (1) accettazione di giorno (`s192-accettazione.sh`: build, catena S-190 con corpus-gate ×2, batteria), e SOLO se verde
# (2) gemelli ORM da archivio (`s192-gemelli-orm.py`, direzione sul tempo) e (3) profilo d'anatomia ORM del tree con la
# lettura dei chiamanti (giudice di meccanismo). Esito in out/leva-<tag>.rc (rc dell'accettazione; 0 = verde) e nei file
# dei tre strumenti. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; O="$H/out"
TAG="${1:?tag}"; CZ="${2:?commit Z}"; CL="${3:?commit L}"
rm -f "$O/leva-$TAG.rc"; cd "$P" || exit 2
bash "$H/s192-accettazione.sh" "$TAG" > "$O/leva-$TAG-acc.log" 2>&1; a=$?
if [ "$a" = 0 ]; then
  python3 "$H/s192-gemelli-orm.py" "$TAG" "$CZ" "$CL" > "$O/leva-$TAG-gemelli.log" 2>&1
  python3 "$H/s192-anatomia.py" "$TAG" orm > "$O/leva-$TAG-anatomia.log" 2>&1
  python3 "$H/s192-anatomia-chiamanti.py" "$TAG" >> "$O/leva-$TAG-anatomia.log" 2>&1
  python3 "$H/s192-anatomia-kernel.py" "$TAG" > "$H/s192-kernel-chiamanti-$TAG.out" 2>> "$O/leva-$TAG-anatomia.log"
fi
echo "$a" > "$O/leva-$TAG.rc"; exit "$a"
