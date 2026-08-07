#!/bin/bash
# s108-list-diff.sh — azione-3 revisore S-107: diff PER NOME degli eseguiti di
# batteria fra pin S-106 (commit 6019890, worktree phpr-s106-worktree) e HEAD
# S-107 (repo principale). `cargo test --release -- --list` enumera gli stessi
# test che la batteria esegue (stesso profilo release, stesse feature default).
# Il worktree usa un TARGET DEDICATO (mai il target di parita', mai lo stash).
# NB: la lista a HEAD rilinka release/phpr (churn): il chiamante DEVE
# ripristinare il pin dallo stash e riverificare l'hash a valle.
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
H="$(cd -P "$(dirname -- "$0")" && pwd -P)"
REPO="$H/.."
WT="/Volumes/Extreme Pro/Claude/phpr-s106-worktree/php-rust"  # la radice git ha php-rust/ come sottodir
WT_TARGET="/Volumes/Extreme Pro/Claude/phpr-s106-listtarget"
OUT="$H/list-out"; mkdir -p "$OUT"
norm() { sed -n 's/^\(.*\): test$/\1/p' "$1" | sort; }

echo "== lista HEAD (S-107) =="
( cd "$REPO" && cargo test --release -- --list ) > "$OUT/head.raw" 2> "$OUT/head.err"
RC_HEAD=$?
echo "rc_head=$RC_HEAD"

echo "== lista S-106 (worktree @ 6019890, target dedicato) =="
( cd "$WT" && CARGO_TARGET_DIR="$WT_TARGET" cargo test --release -- --list ) > "$OUT/s106.raw" 2> "$OUT/s106.err"
RC_S106=$?
echo "rc_s106=$RC_S106"

norm "$OUT/head.raw" > "$OUT/head.names"
norm "$OUT/s106.raw" > "$OUT/s106.names"
echo "n_head=$(wc -l < "$OUT/head.names" | tr -d ' ') n_s106=$(wc -l < "$OUT/s106.names" | tr -d ' ')"
diff "$OUT/s106.names" "$OUT/head.names" > "$OUT/names.diff"
echo "diff (s106 -> head):"
cat "$OUT/names.diff"
echo "rc_head=$RC_HEAD rc_s106=$RC_S106 done=$(date +%T)" > "$OUT/list-diff.done"
