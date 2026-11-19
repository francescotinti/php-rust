#!/bin/bash
# s192-picco-memoria.sh <tag> <binario-Z> <binario-L> — S-192 septies: picco di memoria (`/usr/bin/time -l`, maximum resident
# set size e peak memory footprint) dei due gemelli della leva «registro a tabella», su UNA gamba ORM (×2 per braccio, ordine
# Z L L Z) e UNA gamba WordPress `--group media` per braccio (guardia uploads backup-wipe/restore, wptests ricreato, come
# s191-pair.sh). Soli byte, nessun tempo. Esito: s192-picco-memoria-<tag>.out, out/picco-<tag>.rc. Generato con assistenza AI
# (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; O="$H/out"
TAG="${1:?tag}"; Z="${2:?Z}"; L="${3:?L}"
GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"; SP="$HOME/Claude/phpr-target/s192-orm"; WPDEV="$HOME/Claude/wpdev"
GUARD="/Volumes/Extreme Pro/Claude/wp62-harness/uploads-guard.sh"
V="$H/s192-picco-memoria-$TAG.out"; : > "$V"; rm -f "$O/picco-$TAG.rc"; mkdir -p "$SP"
note(){ echo "$1" >> "$V"; }
peak(){ awk '/maximum resident set size/{r=$1} /peak memory footprint/{f=$1} END{printf "rss=%.1f MiB footprint=%.1f MiB", r/1048576, f/1048576}' "$1"; }
note "== s192 picco di memoria tag=$TAG Z=$(shasum -a 256 "$Z" | cut -c1-16) L=$(shasum -a 256 "$L" | cut -c1-16) $(date '+%F %T') =="
for arm in Z L L Z; do
  [ "$arm" = Z ] && B="$Z" || B="$L"
  rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || { echo 7 > "$O/picco-$TAG.rc"; exit 7; }
  ( cd "$SP/orm-work" && /usr/bin/time -l "$B" vendor/bin/phpunit --no-coverage > "$O/picco-$TAG-orm.txt" 2> "$O/picco-$TAG-orm.time" )
  note "ORM $arm: $(peak "$O/picco-$TAG-orm.time") · $(LC_ALL=C tr -d '\0' < "$O/picco-$TAG-orm.txt" | grep -aE '^Tests:' | tail -1 | cut -c1-40)"
done
rm -rf "$SP/orm-work"
"$GUARD" backup-wipe > "$O/picco-$TAG-guard.log" 2>&1 || { note "uploads-guard backup-wipe FALLITO: niente gamba WP"; echo 4 > "$O/picco-$TAG.rc"; exit 4; }
for arm in Z L; do
  [ "$arm" = Z ] && B="$Z" || B="$L"
  mysql -h 127.0.0.1 -u root -e "DROP DATABASE IF EXISTS wptests; CREATE DATABASE wptests; GRANT ALL ON wptests.* TO 'wp'@'%';" >> "$O/picco-$TAG-guard.log" 2>&1
  find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
  ( cd "$WPDEV" && MIMALLOC_PURGE_DELAY=0 /usr/bin/time -l "$B" vendor/bin/phpunit --group media > "$O/picco-$TAG-wp.txt" 2> "$O/picco-$TAG-wp.time" )
  note "WP  $arm: $(peak "$O/picco-$TAG-wp.time") · $(LC_ALL=C tr -d '\0' < "$O/picco-$TAG-wp.txt" | grep -aE '^(Tests:|OK)' | tail -1 | cut -c1-40)"
done
find "$WPDEV/src/wp-content/uploads" -mindepth 1 -delete 2>/dev/null
"$GUARD" restore >> "$O/picco-$TAG-guard.log" 2>&1; note "uploads-guard restore rc=$?"
echo 0 > "$O/picco-$TAG.rc"
