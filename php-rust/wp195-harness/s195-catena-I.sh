#!/bin/bash
# s195-catena-I.sh — S-195 valore all'invio (criterio s195-criterio-invio.md p.3, p.6); COPIA DICHIARATA di s195-catena-S.sh (manifest s195-catena-I-copia.diff): soli adattamenti = nomi, tag, bracci, attesa della catena S: in sequenza, un solo cargo alla volta:
# (1) accettazione di giorno (wp192-harness/s192-accettazione.sh s195I), (2) istruzioni ORM, (3) istruzioni WordPress,
# braccio S = b9365351 (pacchetto S) contro I = e04ce6e4 (valore all'invio). Esiti: out/catena-I.out, out/catena-I.rc, out/catena-I.done.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; O="$P/wp195-harness/out"; mkdir -p "$O"
V="$O/catena-I.out"; : > "$V"; rm -f "$O/catena-I.rc" "$O/catena-I.done"; RC=0
cd "$P" || exit 2
until [ -f "$O/catena-S.done" ]; do /bin/sleep 30; done
bash "$P/wp192-harness/s192-accettazione.sh" s195I > "$O/catena-I-acc.log" 2>&1; r=$?
echo "accettazione s195I: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
REPS=2 python3 "$P/wp194-harness/s194-orm-istruzioni.py" S=b9365351bd8da06977aa6e7c81420633a7150974 I=e04ce6e43b3ef042b1297fdf09ccab5463b61193 > "$O/catena-I-orm.log" 2>&1; r=$?
echo "istruzioni ORM: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
tail -6 "$P/wp194-harness/s194-orm-istruzioni.out" >> "$V"
python3 "$P/wp194-harness/s194-wp-istruzioni.py" S=b9365351bd8da06977aa6e7c81420633a7150974 I=e04ce6e43b3ef042b1297fdf09ccab5463b61193 > "$O/catena-I-wp.log" 2>&1; r=$?
echo "istruzioni WordPress: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
tail -6 "$P/wp194-harness/s194-wp-istruzioni.out" >> "$V"
echo "esito rc=$RC $(date '+%F %T')" >> "$V"; echo "$RC" > "$O/catena-I.rc"; : > "$O/catena-I.done"; exit "$RC"
