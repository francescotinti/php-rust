#!/bin/bash
# gate-doc-purge.sh — KS-MS-2/KS-MS-4 (A-MS5/A-MS6, Council WP-79 S-78.1.1)
#
# The A-DS1 mine ("persistent Vm", "RetainSet is Send", "preserving persistent
# data", "object table, class cache") authorized a lethal refactor once already
# (WP-78 lesson: a FALSE doc is worse than no doc). This gate fails the build
# if any forbidden doc pattern reappears in the Rust sources.
#
# KS-MS-4: reappearance of a forbidden pattern => gate FAIL.
# A-MS5: the gate proves its own regex BITES via a positive decoy (esca) before
#        scanning the tree — a scan whose self-test cannot fail is void.
#
# Usage: wp78-harness/gate-doc-purge.sh   (from anywhere; repo-relative)
set -u
REPO="$(cd "$(dirname "$0")/.." && pwd)"
SCOPE="$REPO/crates"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"

# Forbidden patterns (fixed strings, case-insensitive). One per line.
PATTERNS=(
  "persistent Vm"
  "persistent-Vm"
  "RetainSet is Send"
  "preserving persistent data"
  "object table, class cache"
  "persistent RetainSet"
  "thread-persistent RetainSet"
)

scan() { # scan <dir> -> matches on stdout, rc 0 if any found
  local dir="$1" found=1 p
  for p in "${PATTERNS[@]}"; do
    if grep -RIni --include='*.rs' --exclude='._*' --exclude-dir=target \
         -F -e "$p" "$dir"; then
      found=0
    fi
  done
  return $found
}

# --- Positive control (esca): the regex MUST bite a decoy ------------------
DECOY_DIR="$(mktemp -d)"
trap 'rm -rf "$DECOY_DIR"' EXIT
fail_self=0
for p in "${PATTERNS[@]}"; do
  printf '// decoy: %s\n' "$p" > "$DECOY_DIR/decoy.rs"
  if ! scan "$DECOY_DIR" > /dev/null; then
    echo "SELF-TEST FAIL: pattern not caught on decoy: '$p'"
    fail_self=1
  fi
done
if [ "$fail_self" -ne 0 ]; then
  echo "gate-doc-purge: SELF-TEST FAILED (esca not bitten) — gate VOID [git $GIT_REV]"
  exit 2
fi
echo "gate-doc-purge: self-test PASS (${#PATTERNS[@]} decoy patterns all bitten)"

# --- Real scan -------------------------------------------------------------
MATCHES="$(scan "$SCOPE" 2>/dev/null)"
if [ -n "$MATCHES" ]; then
  echo "$MATCHES"
  echo "gate-doc-purge: FAIL — forbidden doc pattern reappeared (KS-MS-4) [git $GIT_REV]"
  exit 1
fi
echo "gate-doc-purge: PASS — 0 forbidden patterns in crates/**/*.rs [git $GIT_REV]"
