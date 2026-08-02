#!/bin/bash
# ds40-verify.sh — A-DS40 (Concilio WP-88, Stogov): ancore oracle-vive per il
# catalogo §3.3-ter/quater. Tre bracci per fixture:
#   plain   = oracle 8.5.7, opcache CARICATO ma enable_cli=Off (default brew)
#   persist = oracle 8.5.7, opcache.enable_cli=1 + file_cache_only=1 (branch PERSIST;
#             run1 basta — l'inversione appare già alla prima esecuzione, Stogov WP-88 Q2)
#   phpr    = binario di parità corrente
# Output: wp87-harness/ds40-verify.out (stdout troncato a 3 righe + exit code per braccio).
# KS-DS-88-2: queste fixture committate sono l'ANCORA delle entry §3.3-ter/quater.
set -eu

REPO="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
FIX="$REPO/wp87-harness/fixtures-ds40"
OUT="$REPO/wp87-harness/ds40-verify.out"
ORACLE="/opt/homebrew/opt/php/bin/php"
PHPR="$HOME/Claude/php-rust-output/release/phpr"
FC=$(mktemp -d /tmp/ds40-filecache.XXXXXX)

: > "$OUT"
say() { echo "$@" | tee -a "$OUT"; }

say "== ds40-verify — oracle $($ORACLE -v | head -1 | awk '{print $2}') · phpr $(shasum -a 256 "$PHPR" | cut -c1-16) · git $(cd "$REPO" && git rev-parse --short HEAD) =="
say "branches: plain=opcache-loaded/enable_cli=Off · persist=enable_cli=1+file_cache_only=1 (run1) · phpr"
say ""

run_branch() { # <label> <cmd...>
  local label="$1"; shift
  local rc=0 body
  body=$("$@" 2>&1) || rc=$?
  echo "  $label: exit=$rc stdout=$(echo "$body" | head -3 | tr '\n' '|')"
}

for f in t_hoist_plain t_hoist_parent_earlier t_hoist_parent_later t_hoist_const t_hoist_declared t_hoist_new_predecl t_hoist_conditional t_redeclare tC_lsp_covariance; do
  say "fixture=$f.php"
  run_branch "plain  " "$ORACLE" "$FIX/$f.php" | tee -a "$OUT"
  # fresh file-cache dir per fixture: run1 of the persist pass
  rm -rf "$FC"; mkdir -p "$FC"
  run_branch "persist" "$ORACLE" -d opcache.enable_cli=1 -d opcache.file_cache_only=1 -d "opcache.file_cache=$FC" "$FIX/$f.php" | tee -a "$OUT"
  run_branch "phpr   " "$PHPR" "$FIX/$f.php" | tee -a "$OUT"
  say ""
done
rm -rf "$FC"
say "== ds40-verify DONE =="
