#!/bin/bash
# base-arm-build.sh — A-AH31 (Council WP-84): THE one base-arm builder.
# The two S-82.0 void-campaign traps (worktree-subdir, --locked vs newer
# dev-deps) lived as inline checks in ONE script; every future campaign
# reuses the same base arm — this helper is their single home.
#   - worktree add if missing; crate root = <worktree>/php-rust (campaign-1
#     died on Cargo.toml not at the git root — checked, named);
#   - Cargo.lock COPIED from the live repo (WP-65 same-versions intent);
#   - build --offline (DECLARED deviation: --locked refuses when the live
#     lock carries dev-deps the old manifest lacks; --offline resolves from
#     the copied lock + local cache — release deps identical);
#   - A-AH32 lock-cmp POST-BUILD: `--offline` forbids the NETWORK, not a
#     silent re-resolve from cache — the lock must be byte-identical to the
#     live one after the build, or the arm is VOID (KS-AH-84-1);
#   - A-TH26 header: base_rev, bin hash, rustc -V, sha256 of BOTH locks
#     (equal or the arm is VOID) — appended to <header_out>.
# Usage: base-arm-build.sh <base_rev> <worktree_dir> <target_dir> \
#            <features> <header_out>
# Exit 0 = base binary at <target_dir>/release/php-server, header written.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
BASE_REV="${1:?base rev}"; BASEWT="${2:?worktree dir}"; TGT="${3:?target dir}"
FEATURES="${4:?features}"; HDR="${5:?header out}"

if [ ! -d "$BASEWT" ]; then
  git -C "$REPO" worktree add "$BASEWT" "$BASE_REV" > /dev/null 2>&1 \
    || { echo "FAIL: worktree add $BASE_REV -> $BASEWT"; exit 1; }
fi
# worktree ≠ workspace (S-82.0 lesson): the git root is php-rust-experiment,
# the cargo workspace lives in the php-rust subdir.
BASECRATE="$BASEWT/php-rust"
[ -f "$BASECRATE/Cargo.toml" ] || { echo "FAIL: no Cargo.toml at $BASECRATE (worktree-subdir trap)"; exit 1; }

cp "$REPO/Cargo.lock" "$BASECRATE/Cargo.lock" || { echo "FAIL: lock copy"; exit 1; }
( cd "$BASECRATE" && CARGO_TARGET_DIR="$TGT" cargo build --release --offline \
    -p php-server --features "$FEATURES" ) > "$HDR.build.log" 2>&1 \
  || { echo "FAIL: base build (see $HDR.build.log)"; tail -5 "$HDR.build.log"; exit 1; }

# A-AH32: the tooth Hejlsberg ordered — one line, byte-exact.
if ! cmp -s "$REPO/Cargo.lock" "$BASECRATE/Cargo.lock"; then
  echo "FAIL: Cargo.lock re-resolved during the base build — arm VOID (A-AH32/KS-AH-84-1)"
  exit 1
fi

BASE_BIN="$TGT/release/php-server"
[ -x "$BASE_BIN" ] || { echo "FAIL: no binary at $BASE_BIN"; exit 1; }
BASE_HASH=$(shasum -a 256 "$BASE_BIN" | cut -c1-16)
LOCK_LIVE_SHA=$(shasum -a 256 "$REPO/Cargo.lock" | cut -c1-16)
LOCK_BASE_SHA=$(shasum -a 256 "$BASECRATE/Cargo.lock" | cut -c1-16)
RUSTC_V=$(rustc -V)
{
  echo "base_rev=$BASE_REV base_hash=$BASE_HASH features=$FEATURES"
  echo "rustc=$RUSTC_V"
  echo "lock_live_sha=$LOCK_LIVE_SHA lock_base_sha=$LOCK_BASE_SHA lock_cmp=IDENTICAL"
} >> "$HDR"
echo "OK base-arm: $BASE_HASH ($BASE_REV, lock-cmp IDENTICAL, rustc + lock shas in $HDR)"
exit 0
