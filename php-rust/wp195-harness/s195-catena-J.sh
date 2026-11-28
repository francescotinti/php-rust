#!/bin/bash
# s195-catena-J.sh — S-195 valore all'invio, forma leggera J = abcbfd90 (criterio s195-criterio-invio.md p.8): in coda dietro la
# catena R: (1) accettazione di giorno s195J, (2) istruzioni ORM alternate S,J poi J,S (S = binario già costruito dalla catena I),
# (3) istruzioni WordPress S contro J. Esiti: out/catena-J.out, .rc, .done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; O="$P/wp195-harness/out"; W4="$P/wp194-harness"
V="$O/catena-J.out"; : > "$V"; rm -f "$O/catena-J.rc" "$O/catena-J.done"; RC=0
cd "$P" || exit 2
until [ -f "$O/catena-R.done" ]; do /bin/sleep 30; done
bash "$P/wp192-harness/s192-accettazione.sh" s195J > "$O/catena-J-acc.log" 2>&1; r=$?
echo "accettazione s195J: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1
cp "$W4/out/phpr-orm-istr-S" /private/tmp/phpr-s195-S || exit 2
REPS=2 python3 "$W4/s194-orm-istruzioni.py" S=/private/tmp/phpr-s195-S J=abcbfd9010e12079030856bca3750fec18e5f619 > "$O/catena-J-orm-a.log" 2>&1; r=$?
echo "ORM giro S,J: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1; tail -3 "$W4/s194-orm-istruzioni.out" >> "$V"
cp "$W4/out/phpr-orm-istr-J" /private/tmp/phpr-s195-J || exit 2
REPS=2 python3 "$W4/s194-orm-istruzioni.py" J=/private/tmp/phpr-s195-J S=/private/tmp/phpr-s195-S > "$O/catena-J-orm-b.log" 2>&1; r=$?
echo "ORM giro J,S: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1; tail -3 "$W4/s194-orm-istruzioni.out" >> "$V"
python3 "$W4/s194-wp-istruzioni.py" S=b9365351bd8da06977aa6e7c81420633a7150974 J=abcbfd9010e12079030856bca3750fec18e5f619 > "$O/catena-J-wp.log" 2>&1; r=$?
echo "WordPress S,J: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1; tail -4 "$W4/s194-wp-istruzioni.out" >> "$V"
echo "esito rc=$RC $(date '+%F %T')" >> "$V"; echo "$RC" > "$O/catena-J.rc"; : > "$O/catena-J.done"; exit "$RC"
