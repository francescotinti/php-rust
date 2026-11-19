#!/bin/bash
# s192-censimento-gc.sh <tag> — S-192 octies: conteggi del GC su UNA gamba ORM intera col binario del TREE costruito con la
# feature `gc-census` (strumento esistente, WP-39/48: note, sweep, raccolte dei cicli, radici esaminate, garbage trovato, tempo
# nel classificatore), in una target a parte sulla bundle. Soli CONTEGGI di uno strumento, mai un verdetto di tempo.
# Esito: s192-censimento-gc-<tag>.out, out/censimento-gc-<tag>.rc. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; O="$H/out"; BT="$HOME/Claude/phpr-target"
TAG="${1:?tag}"; GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"; SP="$BT/s192-orm"
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"; V="$H/s192-censimento-gc-$TAG.out"; CF="$O/censimento-gc-$TAG.census"
mkdir -p "$O" "$SP"; rm -f "$O/censimento-gc-$TAG.rc" "$CF"
fin(){ echo "$1" > "$O/censimento-gc-$TAG.rc"; exit "$1"; }
until tail -1 "$FEED" | grep -q '^DONE' && ! pgrep -x cargo >/dev/null; do sleep 30; done
cd "$P" || fin 2
CARGO_TARGET_DIR="$BT/s192-armC" cargo build --release -p php-cli --features php-runtime/gc-census > "$O/censimento-gc-$TAG-build.log" 2>&1 || fin 7
CEN="$BT/s192-armC/release/phpr"
rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || fin 7
( cd "$SP/orm-work" && PHPR_GC_CENSUS="$CF" "$CEN" vendor/bin/phpunit --no-coverage > "$O/censimento-gc-$TAG.txt" 2> "$O/censimento-gc-$TAG.err" ); xrc=$?
{
  echo "== s192 censimento GC tag=$TAG git=$(git rev-parse --short HEAD) C=$(shasum -a 256 "$CEN" | cut -c1-16) phpunit rc=$xrc $(date '+%F %T') (soli conteggi) =="
  LC_ALL=C tr -d '\0' < "$O/censimento-gc-$TAG.txt" | LC_ALL=C grep -aE '^Tests:' | tail -1
  LC_ALL=C tr -d '\0' < "$CF" 2>/dev/null | cut -c1-400
} > "$V"
rm -rf "$SP/orm-work"; fin 0
