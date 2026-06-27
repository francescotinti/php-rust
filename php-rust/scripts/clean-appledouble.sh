#!/usr/bin/env bash
# Remove macOS AppleDouble (`._*`) and `.DS_Store` sidecars that accumulate on the
# external work volume. These confuse Serena's indexer (`._mod.rs` looks like a
# source file) and the copies git writes under `.git/objects/pack` trigger
# "non-monotonic index" errors on every `git` invocation.
#
# Usage:
#   scripts/clean-appledouble.sh         # clean the working tree (outside .git)
#   scripts/clean-appledouble.sh --git   # also clean inside .git (fixes pack errors)
#   scripts/clean-appledouble.sh --check  # exit 1 if any exist in the working tree
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

case "${1:-}" in
  --check)
    found="$(find . -name '._*' -not -path './.git/*' 2>/dev/null || true)"
    if [ -n "$found" ]; then
      echo "AppleDouble files present in the working tree:" >&2
      echo "$found" >&2
      exit 1
    fi
    echo "clean: no AppleDouble files in the working tree"
    ;;
  --git)
    n=$(find . -name '._*' -delete -print 2>/dev/null | wc -l | tr -d ' ')
    find . -name '.DS_Store' -delete 2>/dev/null || true
    echo "removed $n AppleDouble file(s) (working tree + .git)"
    ;;
  *)
    n=$(find . -name '._*' -not -path './.git/*' -delete -print 2>/dev/null | wc -l | tr -d ' ')
    find . -name '.DS_Store' -not -path './.git/*' -delete 2>/dev/null || true
    echo "removed $n AppleDouble file(s) from the working tree (.git left intact; use --git to include it)"
    ;;
esac
