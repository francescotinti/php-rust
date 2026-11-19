#!/bin/bash
# s192-gemelli.sh <tag> <commit-Z> <commit-L> — S-192: due bracci GEMELLI da ARCHIVIO (mai il tree), stessa ricetta del pin
# (`cargo build --release -p php-cli`, SOURCE_DATE_EPOCH=0, CARGO_INCREMENTAL=0), target SIMMETRICHE sulla bundle
# (`~/Claude/phpr-target/s192-arm{Z,L}`, sorgenti in `s192-src{Z,L}`: percorsi di pari lunghezza), poi direzione appaiata
# L−Z con `s192-direzione-zl.sh` (copia dichiarata di s190-direzione-k0.sh). Z = senza leva, L = con la leva: l'unica differenza
# fra i due binari è il commit. DIREZIONE, non verdetto. Esiti: out/gemelli-<tag>.rc, verdetto s192-direzione-zl-<tag>.out.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
P="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"; H="$P/wp192-harness"; O="$H/out"; BT="$HOME/Claude/phpr-target"
TAG="${1:?tag}"; CZ="${2:?commit Z}"; CL="${3:?commit L}"
mkdir -p "$O"; rm -f "$O/gemelli-$TAG.rc"
fail(){ echo "$1" > "$O/gemelli-$TAG.rc"; exit "$1"; }
FEED="/Volumes/Extreme Pro/Claude/phpr-ci/CI_FEED.log"
until tail -1 "$FEED" | grep -q '^DONE' && ! pgrep -x cargo >/dev/null; do sleep 30; done
build(){ # $1=arm letter $2=commit
  local SRC="$BT/s192-src$1" TGT="$BT/s192-arm$1"
  rm -rf "$SRC"; mkdir -p "$SRC" || return 1
  git -C "$P/.." archive "$2" php-rust/crates php-rust/Cargo.toml php-rust/Cargo.lock php-rust/rust-toolchain.toml php-rust/.cargo 2>/dev/null | tar -x -m -C "$SRC" || return 1  # -m: archive mtimes are commit times, cargo would keep stale rlibs
  ( cd "$SRC/php-rust" && SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TGT" cargo build --release -p php-cli ) > "$O/gemelli-$TAG-build$1.log" 2>&1 || return 1
  cp "$TGT/release/phpr" "$O/phpr-$TAG-$1" || return 1
  echo "arm $1 commit $2 hash $(shasum -a 256 "$O/phpr-$TAG-$1" | cut -c1-16) $(date '+%F %T')" >> "$O/gemelli-$TAG.log"
}
: > "$O/gemelli-$TAG.log"
build Z "$CZ" || fail 7
build L "$CL" || fail 7
A="$O/phpr-$TAG-Z" TREE="$O/phpr-$TAG-L" R="${R:-5}" bash "$H/s192-direzione-zl.sh" "$TAG" >> "$O/gemelli-$TAG.log" 2>&1; r=$?
echo "$r" > "$O/gemelli-$TAG.rc"; exit "$r"
