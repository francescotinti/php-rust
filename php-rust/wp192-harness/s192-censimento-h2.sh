#!/bin/bash
# s192-censimento-h2.sh <tag> — S-192: censimento h2 (PERCHÉ un Ret o una chiamata di metodo esce dal cammino veloce) su UNA
# gamba ORM intera (orm-work.tgz, phpunit, come il braccio C di s186-fetta0.sh). Strumento = binario del TREE con
# `--features census-time` costruito in una target a parte sulla bundle (`~/Claude/phpr-target/s192-armC`): soli CONTEGGI, mai
# tempi, mai un verdetto. Attende la CI locale quieta. Esiti: s192-censimento-h2-<tag>.out (righe census ordinate),
# out/censimento-<tag>.rc. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; O="$H/out"; BT="$HOME/Claude/phpr-target"
TAG="${1:?tag}"; GATES="/Volumes/Extreme Pro/Claude/wp9-harness/gates"; SP="$BT/s192-orm"; TESTS_ATTESI=3484
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"; VERD="$H/s192-censimento-h2-$TAG.out"
mkdir -p "$O" "$SP"; rm -f "$O/censimento-$TAG.rc"
fin(){ echo "$1" > "$O/censimento-$TAG.rc"; exit "$1"; }
[ -s "$GATES/orm-work.tgz" ] || fin 7
quiet(){ tail -1 "$FEED" | grep -q '^DONE' && ! pgrep -x cargo >/dev/null && ! pgrep -f 'ci-target/release' >/dev/null; }
until quiet && sleep 60 && quiet; do sleep 30; done
cd "$P" || fin 2
CARGO_TARGET_DIR="$BT/s192-armC" cargo build --release -p php-cli --features census-time > "$O/censimento-$TAG-build.log" 2>&1 || fin 7
CEN="$BT/s192-armC/release/phpr"
rm -rf "$SP/orm-work"; tar xzf "$GATES/orm-work.tgz" -C "$SP" || fin 7
( cd "$SP/orm-work" && "$CEN" vendor/bin/phpunit --no-coverage > "$O/censimento-$TAG.txt" 2> "$O/censimento-$TAG.err" ); xrc=$?
{
  echo "== s192 censimento h2 tag=$TAG git=$(git rev-parse --short HEAD) C=$(shasum -a 256 "$CEN" | cut -c1-16) phpunit rc=$xrc $(date '+%F %T') (soli conteggi) =="
  LC_ALL=C tr -d '\0' < "$O/censimento-$TAG.txt" | LC_ALL=C grep -aE '^(Tests|OK)' | tail -1
  LC_ALL=C grep -a '^census-time' "$O/censimento-$TAG.err" | awk -F'\t' '{print $1"\t"$2"\t"$3}'
  LC_ALL=C grep -a '^census-h2' "$O/censimento-$TAG.err" | sort -t "$(printf '\t')" -k2,2 -k4,4nr
} > "$VERD"
rm -rf "$SP/orm-work"
LC_ALL=C tr -d '\0' < "$O/censimento-$TAG.txt" | LC_ALL=C grep -aqE "^Tests: $TESTS_ATTESI," && fin 0
fin 1
