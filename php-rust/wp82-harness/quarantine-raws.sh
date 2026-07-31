#!/bin/bash
# quarantine-raws.sh — A-AH27/KS-AH-83-2 (Council WP-83): raw campaign files
# are NEVER `rm`-ed, VOID or not. VOID ≠ inesistente: a removal that is not
# diff-verifiable makes the surviving campaign non-falsifiable (the 13+13
# WP-81 raws are the standing counter-example, retro-annotated in MEASURE81).
#
# Usage: quarantine-raws.sh <motivo> <file>...
#   Moves the files into  wp82-harness/evidence/void/<stamp>-<rev>/  and
#   writes MANIFEST.txt (name, sha256, size, motivo, git rev, who) that MUST
#   be committed with the substitute campaign (KS-AH-83-2: senza manifest la
#   campagna sostitutiva è VOID).
#   KH83-2: the number of quarantined runs is printed for the campaign header
#   («run VOIDate contate»); the driver of the NEXT campaign cites it.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"

[ $# -ge 2 ] || { echo "usage: quarantine-raws.sh <motivo> <file>..."; exit 2; }
MOTIVO="$1"; shift

REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
DEST="$HERE/evidence/void/$STAMP-$REV"
mkdir -p "$DEST"

MANIFEST="$DEST/MANIFEST.txt"
{
  echo "# Quarantena raw (A-AH27/KS-AH-83-2) — mai rm, sempre manifest"
  echo "stamp: $STAMP"
  echo "git_rev: $REV"
  echo "motivo: $MOTIVO"
  echo "files:"
} > "$MANIFEST"

N=0
for f in "$@"; do
  [ -f "$f" ] || { echo "SKIP (not a file): $f"; continue; }
  SHA=$(shasum -a 256 "$f" | awk '{print $1}')
  SZ=$(stat -f %z "$f")
  BASE=$(basename "$f")
  mv "$f" "$DEST/$BASE"
  echo "  - $BASE sha256=$SHA size=$SZ" >> "$MANIFEST"
  N=$((N+1))
done

echo "quarantined: $N file(s) -> ${DEST#"$REPO"/}"
echo "KH83-2: campaign header must carry 'void_runs=$N (manifest ${DEST#"$REPO"/}/MANIFEST.txt)'"
echo "REMINDER: git add + commit the quarantine BEFORE the substitute campaign (KS-AH-83-2)"
