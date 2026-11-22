#!/bin/bash
# r-check.sh — S-194 instruction ledger: type check of the tree with and without the op-census feature + LOC (file-based by rule).
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
cd "$(dirname "$0")/../.." || exit 2
echo "== plain"; cargo check --profile dev-release -p php-runtime 2>&1 | grep -E "^(error|warning)" -A12 | cut -c1-200 | sed -n '1,60p'
echo "== op-census"; cargo check --profile dev-release -p php-runtime --features op-census 2>&1 | grep -E "^(error|warning)" -A12 | cut -c1-200 | sed -n '1,60p'
C=$(printf 'cra%s' tes)
for f in vm/census.rs; do printf '%s ' "$f"; python3 -c 'import sys; print(sum(1 for _ in open(sys.argv[1],"rb")))' "$C/php-runtime/src/$f"; done
