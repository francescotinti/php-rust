#!/bin/bash
# s195-catena-S.sh — S-195 pacchetto S (criterio s195-criterio-sweep.md p.5-6): in sequenza, un solo cargo alla volta:
# (1) accettazione di giorno (wp192-harness/s192-accettazione.sh s195S), (2) istruzioni ORM, (3) istruzioni WordPress,
# braccio A = 3b78c61c (sorgente del pin s190) contro S = b9365351. Esiti: out/catena-S.out, out/catena-S.rc, out/catena-S.done.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; O="$P/wp195-harness/out"; mkdir -p "$O"
V="$O/catena-S.out"; : > "$V"; rm -f "$O/catena-S.rc" "$O/catena-S.done"; RC=0
cd "$P" || exit 2
bash "$P/wp192-harness/s192-accettazione.sh" s195S > "$O/catena-S-acc.log" 2>&1; r=$?
echo "accettazione s195S: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
REPS=2 python3 "$P/wp194-harness/s194-orm-istruzioni.py" A=3b78c61c5b1d34e2acc8844abb8cdf279aa5f269 S=b9365351bd8da06977aa6e7c81420633a7150974 > "$O/catena-S-orm.log" 2>&1; r=$?
echo "istruzioni ORM: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
tail -6 "$P/wp194-harness/s194-orm-istruzioni.out" >> "$V"
python3 "$P/wp194-harness/s194-wp-istruzioni.py" A=3b78c61c5b1d34e2acc8844abb8cdf279aa5f269 S=b9365351bd8da06977aa6e7c81420633a7150974 > "$O/catena-S-wp.log" 2>&1; r=$?
echo "istruzioni WordPress: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
tail -6 "$P/wp194-harness/s194-wp-istruzioni.out" >> "$V"
echo "esito rc=$RC $(date '+%F %T')" >> "$V"; echo "$RC" > "$O/catena-S.rc"; : > "$O/catena-S.done"; exit "$RC"
