#!/bin/bash
# s195-catena-I2.sh — S-195 valore all'invio, istruttoria del +0,10 % ORM (criterio s195-criterio-invio.md p.7): rimisura in
# istruzioni dei DUE binari già costruiti dalla catena I (nessuna build), ordine alternato S,I poi I,S, REPS=2 per giro.
# In coda dietro la catena R. Esiti: out/catena-I2.out, .rc, .done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; O="$P/wp195-harness/out"; W4="$P/wp194-harness"
V="$O/catena-I2.out"; : > "$V"; rm -f "$O/catena-I2.rc" "$O/catena-I2.done"; RC=0
cd "$P" || exit 2
until [ -f "$O/catena-R.done" ]; do /bin/sleep 30; done
cp "$W4/out/phpr-orm-istr-S" /private/tmp/phpr-s195-S && cp "$W4/out/phpr-orm-istr-I" /private/tmp/phpr-s195-I || exit 2
echo "sha S $(shasum -a 256 /private/tmp/phpr-s195-S | cut -c1-16) I $(shasum -a 256 /private/tmp/phpr-s195-I | cut -c1-16)" >> "$V"
REPS=2 python3 "$W4/s194-orm-istruzioni.py" S=/private/tmp/phpr-s195-S I=/private/tmp/phpr-s195-I > "$O/catena-I2-a.log" 2>&1; r=$?
echo "giro S,I: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1; tail -3 "$W4/s194-orm-istruzioni.out" >> "$V"
REPS=2 python3 "$W4/s194-orm-istruzioni.py" I=/private/tmp/phpr-s195-I S=/private/tmp/phpr-s195-S > "$O/catena-I2-b.log" 2>&1; r=$?
echo "giro I,S: rc=$r" >> "$V"; [ "$r" = 0 ] || RC=1; tail -3 "$W4/s194-orm-istruzioni.out" >> "$V"
echo "esito rc=$RC $(date '+%F %T')" >> "$V"; echo "$RC" > "$O/catena-I2.rc"; : > "$O/catena-I2.done"; exit "$RC"
