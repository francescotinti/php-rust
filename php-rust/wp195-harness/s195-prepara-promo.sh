#!/bin/bash
# s195-prepara-promo.sh — S-195, criterion s195-criterio-promozione.md p.5/p.7/p.8, everything that needs no disk: (1) candidate arm copied to
# out/phpr-cand, (2) chain generated, (3) dry run of the fixture gates, (4) cycle gate on mcalls-dq (s194g-cicli.py, R=9): stash s190, a second
# COPY of s190 (A-A round), candidate, stash s189, all on paths without spaces. Outcome: out/prepara-promo.out, .rc, .done.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; W="$P/wp195-harness"; W4="$P/wp194-harness"; O="$W/out"
ST="/Volumes/Extreme Pro/Claude/phpr-old-target/release"
V="$O/prepara-promo.out"; : > "$V"; rm -f "$O/prepara-promo.rc" "$O/prepara-promo.done"; RC=0
cd "$P" || exit 2
cp "$W4/out/phpr-orm-istr-J" "$O/phpr-cand" || exit 2
echo "candidato $(shasum -a 256 "$O/phpr-cand" | cut -c1-16)" >> "$V"
python3 "$W/gen-promozione-s195.py" abcbfd90 "$@" >> "$V" 2>&1 || RC=1
bash -n "$W/s195-promozione.sh" && bash -n "$W/s195-lancio-promo.sh" || RC=1
diff "$W4/out/e-dryrun.sh" "$W/s195-dryrun.sh" > "$W/s195-dryrun-copia.diff"
bash "$W/s195-dryrun.sh"; echo "prova a secco: gate che mordono = $(cat "$O/dryrun.rc")" >> "$V"; [ "$(cat "$O/dryrun.rc")" = 0 ] || RC=1
cp "$ST/phpr-s190" /private/tmp/phpr-s195-c190 && cp "$ST/phpr-s190" /private/tmp/phpr-s195-c190b && cp "$ST/phpr-s189" /private/tmp/phpr-s195-c189 && cp "$O/phpr-cand" /private/tmp/phpr-s195-ccand || exit 2
R=9 python3 "$W4/s194g-cicli.py" s195promo s190=/private/tmp/phpr-s195-c190 s190b=/private/tmp/phpr-s195-c190b cand=/private/tmp/phpr-s195-ccand s189=/private/tmp/phpr-s195-c189 >> "$V" 2>&1 || RC=1
cp "$W4/s194g-cicli-s195promo.out" "$W/s195-cicli-promo.out" && cat "$W/s195-cicli-promo.out" >> "$V"
echo "esito rc=$RC $(date '+%F %T')" >> "$V"; echo "$RC" > "$O/prepara-promo.rc"; : > "$O/prepara-promo.done"; exit "$RC"
