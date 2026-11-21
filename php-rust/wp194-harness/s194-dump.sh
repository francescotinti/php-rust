#!/bin/bash
# dumps the bytecode of the fixtures given as arguments (PHPR_DUMP_OPS), pin binary; raw stderr, chunk headers listed first
H="$(cd "$(dirname "$0")" && pwd)"; B="$HOME/Claude/php-rust-output/release/phpr"
for f in "$@"; do echo "##### $f"; PHPR_DUMP_OPS=1 "$B" "$H/primitive/$f.php" 3 2>"$H/primitive/$f.ops" >/dev/null; grep -n '^--' "$H/primitive/$f.ops" | cut -c1-120; wc -l < "$H/primitive/$f.ops"; done
