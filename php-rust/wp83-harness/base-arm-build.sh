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
# Usage (v2, A-TH30 — the expected pruned-set is NAMED per campaign):
#   base-arm-build.sh <base_rev> <worktree_dir> <target_dir> \
#            <features> <header_out> <expected_pruned_pkgs>
# <expected_pruned_pkgs> = comma-list of package names the campaign EXPECTS
# the prune to touch (e.g. "mimalloc"); "-" declares NO prune expected.
# A deletion touching any package outside the named set = arm VOID.
# Exit 0 = base binary at <target_dir>/release/php-server, header written.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
BASE_REV="${1:?base rev}"; BASEWT="${2:?worktree dir}"; TGT="${3:?target dir}"
FEATURES="${4:?features}"; HDR="${5:?header out}"
EXPECTED_PRUNED="${6:?expected pruned pkgs (comma-list or - for none) — A-TH30}"

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

# A-AH32 (PRUNE-ONLY, judged and ACCEPTED WITH REINFORCEMENT by Council
# WP-85 — Hejlsberg A-AH35/A-AH37, Hoare A-TH30): the byte-exact tooth was
# structurally unsatisfiable across manifests that differ by a dev-dep
# (WP-72 class); the honest form distinguishes the legitimate prune from a
# re-resolve AND from a provenance-strip. Teeth, all machine:
#   (a) the byte-diff live->post-build may contain ONLY DELETED lines
#       (prune); ANY added/changed line = re-resolve = arm VOID;
#   (b) the (name, version) pairs of the post-build lock must ALL appear
#       IDENTICAL in the live lock (subset; drift/addition = VOID);
#   (c) A-AH35: deletions are legal ONLY as (c1) dependency-EDGE lines
#       (quoted list elements) inside a SURVIVING package's block, or (c2)
#       whole [[package]] blocks whose name@version is ABSENT from the
#       post-build lock. A deleted version=/source=/checksum= line with the
#       package SURVIVING = provenance-strip = arm VOID (KS-AH-85-1);
#   (d) A-TH30: resolver = "2" pinned in BOTH manifests (under resolver v1
#       dev-dep edges unify features into `cargo build` — the prune would
#       then be a CPU-bias vector; KH85-2), the expected pruned-set is
#       NAMED per campaign, and the diff is archived INTEGRAL on file.
LOCK_DIFF=$(diff "$REPO/Cargo.lock" "$BASECRATE/Cargo.lock" || true)
LOCK_ADDED=$(printf '%s\n' "$LOCK_DIFF" | grep -c '^>' || true)
LOCK_PRUNED=$(printf '%s\n' "$LOCK_DIFF" | grep -c '^<' || true)
if [ "$LOCK_ADDED" -gt 0 ]; then
  echo "FAIL: Cargo.lock gained/changed lines during the base build — re-resolve, arm VOID (A-AH32/KS-AH-84-1):"
  printf '%s\n' "$LOCK_DIFF" | grep '^>' | head -5
  exit 1
fi
VERS_DRIFT=$(awk '/^name = /{n=$3} /^version = /{if (n) {print n, $3; n=""}}' "$BASECRATE/Cargo.lock" | sort > /tmp/base-arm-lock-base.$$;
  awk '/^name = /{n=$3} /^version = /{if (n) {print n, $3; n=""}}' "$REPO/Cargo.lock" | sort > /tmp/base-arm-lock-live.$$;
  comm -23 /tmp/base-arm-lock-base.$$ /tmp/base-arm-lock-live.$$; rm -f /tmp/base-arm-lock-base.$$ /tmp/base-arm-lock-live.$$)
if [ -n "$VERS_DRIFT" ]; then
  echo "FAIL: base lock carries (name, version) pairs ABSENT from the live lock — version drift, arm VOID (A-AH32):"
  echo "$VERS_DRIFT" | head -5
  exit 1
fi

# (d) A-TH30/KH85-2: resolver pinned "2" in BOTH manifests, or the
# prune-only innocuousness argument does not hold.
RES_LIVE=$(grep -c '^resolver = "2"' "$REPO/Cargo.toml" || true)
RES_BASE=$(grep -c '^resolver = "2"' "$BASECRATE/Cargo.toml" || true)
if [ "$RES_LIVE" -ne 1 ] || [ "$RES_BASE" -ne 1 ]; then
  echo "FAIL: resolver=\"2\" not pinned in both manifests (live=$RES_LIVE base=$RES_BASE) — arm VOID (A-TH30/KH85-2)"
  exit 1
fi

# (c) A-AH35: per-package legality of every deleted line.
# Split each lock into [[package]] blocks keyed name@version.
split_lock() { # <lockfile> <outdir>
  awk -v d="$2" '
    function flush() { if (key != "") printf "%s", buf > (d "/" key); key=""; buf="" }
    /^\[\[package\]\]/ { flush(); collecting=1; next }
    collecting {
      buf = buf $0 "\n"
      if ($1 == "name")    { n = $3; gsub(/"/, "", n) }
      if ($1 == "version") { v = $3; gsub(/"/, "", v); key = n "@" v }
    }
    END { flush() }
  ' "$1"
}
BD=$(mktemp -d); mkdir -p "$BD/live" "$BD/base"
split_lock "$REPO/Cargo.lock" "$BD/live"
split_lock "$BASECRATE/Cargo.lock" "$BD/base"
PRUNED_PKGS=""; VIOL=""
for f in "$BD/live"/*; do
  k=${f##*/}
  if [ ! -f "$BD/base/$k" ]; then
    # (c2) whole-block prune — legal; record the package name for (d).
    PRUNED_PKGS="$PRUNED_PKGS ${k%@*}"
    continue
  fi
  PKG_DIFF=$(diff "$f" "$BD/base/$k" || true)
  [ -z "$PKG_DIFF" ] && continue
  # (c1) surviving package: every deleted line must be a quoted dependency-
  # edge list element; anything else (version=/source=/checksum=/brackets)
  # is a provenance-strip or a structural edit — VOID.
  BAD=$(printf '%s\n' "$PKG_DIFF" | grep '^<' | grep -vE '^< *"[^"]*",?$' || true)
  if [ -n "$BAD" ]; then
    VIOL="$VIOL$k: $(printf '%s\n' "$BAD" | head -2 | tr '\n' ' ')"$'\n'
  else
    # legal edge deletions: record the EDGE TARGET name (the S-83.0 real
    # case names "mimalloc", the pruned dev-dep — not the surviving owner).
    EDGES=$(printf '%s\n' "$PKG_DIFF" | grep '^<' | sed -E 's/^< *"([A-Za-z0-9_.-]+).*/\1/')
    PRUNED_PKGS="$PRUNED_PKGS $EDGES"
  fi
done
rm -rf "$BD"
if [ -n "$VIOL" ]; then
  echo "FAIL: non-edge deletion inside a SURVIVING package — provenance-strip class, arm VOID (A-AH35/KS-AH-85-1):"
  printf '%s' "$VIOL" | head -5
  exit 1
fi
# (d) A-TH30: the pruned-set must be NAMED per campaign — nothing outside it.
PRUNED_PKGS=$(printf '%s\n' $PRUNED_PKGS | sort -u | tr '\n' ' ')
UNEXPECTED=""
for p in $PRUNED_PKGS; do
  case ",$EXPECTED_PRUNED," in
    *",$p,"*) ;;
    *) UNEXPECTED="$UNEXPECTED $p";;
  esac
done
if [ -n "$UNEXPECTED" ]; then
  echo "FAIL: prune touched packages OUTSIDE the campaign-named set '$EXPECTED_PRUNED':$UNEXPECTED — arm VOID (A-TH30)"
  exit 1
fi
# (d) diff archived INTEGRAL on file — never head-truncated evidence.
printf '%s\n' "$LOCK_DIFF" > "$HDR.lockdiff"

BASE_BIN="$TGT/release/php-server"
[ -x "$BASE_BIN" ] || { echo "FAIL: no binary at $BASE_BIN"; exit 1; }
BASE_HASH=$(shasum -a 256 "$BASE_BIN" | cut -c1-16)
LOCK_LIVE_SHA=$(shasum -a 256 "$REPO/Cargo.lock" | cut -c1-16)
LOCK_BASE_SHA=$(shasum -a 256 "$BASECRATE/Cargo.lock" | cut -c1-16)
RUSTC_V=$(rustc -V)
{
  echo "base_rev=$BASE_REV base_hash=$BASE_HASH features=$FEATURES"
  echo "rustc=$RUSTC_V resolver=2 resolver_base=2"
  echo "lock_live_sha=$LOCK_LIVE_SHA lock_base_sha=$LOCK_BASE_SHA lock_cmp=PRUNE-ONLY pruned_lines=$LOCK_PRUNED added_lines=0 version_drift=0 provenance_strip=0"
  echo "pruned_set='$PRUNED_PKGS' expected_pruned='$EXPECTED_PRUNED' lock_diff_file=$HDR.lockdiff"
} >> "$HDR"
echo "OK base-arm: $BASE_HASH ($BASE_REV, lock-cmp PRUNE-ONLY+A-AH35 pruned=$LOCK_PRUNED set='$PRUNED_PKGS', integral diff at $HDR.lockdiff)"
exit 0
