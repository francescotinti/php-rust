#!/bin/bash
# gate-dr1-module-immut.sh — DR-1 (design79 §5; Council WP-81 A-TH13/A-PP13,
# KH81-3): MACHINE check of interior mutability in the graphs a cached main
# module would share cross-request — Module (bytecode.rs) AND Program (hir.rs,
# the A-PP13 extension: the HIR is reused cross-request too via main_program).
#
# AUDIT VERDICT the gate encodes (S-80.0.8; header REWRITTEN S-82.0 p3,
# A-TH19 — the old header still declared u32 after the KH82-1 widening:
# gate doc that lies is the M-68.5 class, WP-78 lesson):
#   - The Program graph (hir.rs) has NO interior mutability: pinned 0 tokens.
#   - The Module graph has EXACTLY the WP-29/30 inline caches — PropIc and
#     MethodIc, `Rc<Cell<(u64,u32,u32,u32)>>` — plus their thread-local
#     IC_EPOCH (`Cell<u64>`). Every fill embeds the epoch; `Vm::new` calls
#     bump_ic_epoch() so ANY cached entry from a previous request is stale by
#     construction: per-request O(1) invalidation at the Vm boundary. This is
#     the "reset-on-hit con contatore" family of KH81-3 (the reset is
#     stronger: per-request, observable via the epoch tests below) — the
#     IC-out-of-Module (run_time_cache) form was not needed.
#   - Epoch WIDTH residual: CLOSED at u64 (KH82-1, lever commit 57ec7dc) and
#     GATED by tooth 1w below (A-TH19): the epoch is u64 — wrap is
#     machine-impossible (~5.8e11 years at 1k req/s). A silent u64→u32
#     revert now FAILS tooth 1w; the KH83-1 mutation self-test proves the
#     tooth bites on exactly that revert.
#   - Declared boundary: Op payloads embed Zval CONSTANTS, and the Zval TYPE
#     (php-types) admits Ref(Rc<RefCell<..>>)/shared arrays — outside this
#     token scan. The invariant relied on is that lower/compile only ever
#     produce LITERAL consts (Ref cells are made at runtime) and PHP array
#     value-semantics clones on write. That invariant is exercised
#     BEHAVIORALLY by G-APERTURA-2/gate_stateful (sequential byte-parity)
#     and, on the cache-HIT path, by fixtures F1-F9 (design79 §9) — the F7
#     stale-state fixture is the tripwire for exactly this class.
#
# Teeth (never prose, KH81-3):
#   1. Token census: TYPE-position interior-mutability tokens (Cell< etc.;
#      constructors like Cell::new necessarily follow a typed field) in
#      bytecode.rs == 3, every one naming IC_EPOCH/PropIc/MethodIc (names,
#      not line numbers — structural lock, WP-72); hir.rs == 0.
#      A new Cell/RefCell/atomic in either graph FAILS until DR-1 reruns.
#   1w. WIDTH pin (A-TH19): the IC_EPOCH declaration line carries `Cell<u64>`
#      and bytecode.rs has EXACTLY 2 non-comment `Cell<(u64,` (PropIc +
#      MethodIc tuples) and ZERO `Cell<(u32`/`Cell<u32>` on IC lines — a
#      silent u64→u32 revert fails HERE (mutation-tested, KH83-1).
#   2. Mechanism runs: the two epoch-invalidation tests (PropIc + MethodIc,
#      both asserting "epoch bump invalidates") pass under cargo.
#   3. Wiring: bump_ic_epoch() is CALLED in vm/mod.rs non-test code
#      (the Vm constructor) exactly once — defined-but-never-called would
#      leave every entry valid forever.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
BYTECODE="$REPO/crates/php-runtime/src/bytecode.rs"
HIR="$REPO/crates/php-runtime/src/hir.rs"
VMMOD="$REPO/crates/php-runtime/src/vm/mod.rs"
FAILS=0
TOKEN_RE='(Cell<|RefCell<|UnsafeCell|OnceCell|OnceLock|Mutex<|RwLock<|Atomic(U|I|Bool|Usize)|FrozenVec)'

count_tokens() { # <file> -> "count" (comment lines excluded)
  # A-TH17 (Council WP-82): count OCCURRENCES via gsub, not matching lines —
  # a second mutability token smuggled onto an already-allowlisted line
  # passed both the line count ==3 and the per-line allowlist.
  awk -v re="$TOKEN_RE" '
    /^[[:space:]]*\/\// { next }
    { n += gsub(re, "&") }
    END { print n + 0 }
  ' "$1"
}

# --- positive control: the token scan must BITE a decoy ----------------------
TMPD=$(mktemp -d)
trap 'rm -rf "$TMPD"' EXIT
printf 'pub struct Bad { c: std::cell::Cell<u32> }\n// Cell< in a comment does not count\n' > "$TMPD/decoy.rs"
n=$(count_tokens "$TMPD/decoy.rs")
if [ "$n" -ne 1 ]; then
  echo "SELF-TEST BROKEN: decoy counted $n != 1 (comment must not count)"; exit 2
fi
# A-TH17 decoy: TWO tokens on ONE line must count 2 (the smuggled-second-token
# case the line-based count blessed).
printf 'pub struct Bad2 { a: Cell<u32>, b: RefCell<u32> } // PropIc\n' > "$TMPD/decoy2.rs"
n=$(count_tokens "$TMPD/decoy2.rs")
if [ "$n" -ne 2 ]; then
  echo "SELF-TEST BROKEN: two-token line counted $n != 2 (A-TH17 gsub)"; exit 2
fi
echo "OK  self-test: token scan bites (decoy=1 comment excluded; two-token line=2, A-TH17)"

# --- 1. token census ---------------------------------------------------------
n=$(count_tokens "$BYTECODE")
if [ "$n" -ne 3 ]; then
  echo "FAIL: bytecode.rs has $n interior-mutability tokens, pinned 3 (DR-1)"
  echo "      new mutability in the Module graph requires a DR-1 re-audit:"
  awk -v re="$TOKEN_RE" '/^[[:space:]]*\/\//{next} $0 ~ re {print FNR": "$0}' "$BYTECODE" | head -10
  FAILS=$((FAILS+1))
else
  # every counted line must belong to the allowlisted IC machinery
  BADLINES=$(awk -v re="$TOKEN_RE" '
    /^[[:space:]]*\/\// { next }
    $0 ~ re && $0 !~ /IC_EPOCH|PropIc|MethodIc/ { print FNR": "$0 }
  ' "$BYTECODE")
  if [ -n "$BADLINES" ]; then
    echo "FAIL: interior-mutability line outside the IC allowlist (DR-1):"
    echo "$BADLINES"
    FAILS=$((FAILS+1))
  else
    echo "OK  bytecode.rs: $n type-position tokens, all IC_EPOCH/PropIc/MethodIc (pinned 3)"
  fi
fi
# --- 1w. WIDTH pin (A-TH19) + KH83-1 mutation self-test ----------------------
check_width() { # <bytecode-file> -> rc 1 on violation
  local ep n32 n64
  ep=$(awk '/^[[:space:]]*\/\//{next} /IC_EPOCH/ && /Cell</ {print; exit}' "$1")
  if ! printf '%s' "$ep" | grep -q 'Cell<u64>'; then
    echo "FAIL: IC_EPOCH declaration is not Cell<u64> (A-TH19): $ep"; return 1
  fi
  n64=$(awk '/^[[:space:]]*\/\//{next} { n += gsub(/Cell<\(u64,/, "&") } END{print n+0}' "$1")
  n32=$(awk '/^[[:space:]]*\/\//{next} { n += gsub(/Cell<\(u32|Cell<u32>/, "&") } END{print n+0}' "$1")
  if [ "$n64" -ne 2 ]; then
    echo "FAIL: Cell<(u64, tuples = $n64, pinned 2 (PropIc+MethodIc, A-TH19)"; return 1
  fi
  if [ "$n32" -ne 0 ]; then
    echo "FAIL: u32-width IC cell present ($n32 occurrences) — silent epoch narrowing (A-TH19)"; return 1
  fi
  return 0
}
if check_width "$BYTECODE"; then
  echo "OK  width pin: IC_EPOCH Cell<u64> + 2x Cell<(u64, tuples, no u32 (A-TH19)"
else
  FAILS=$((FAILS+1))
fi
# KH83-1: the synthetic u64->u32 revert MUST fail the tooth — otherwise the
# "DR-1 width residual closed" claim demotes itself to ADVISORY.
sed -e 's/Cell<u64>/Cell<u32>/g' -e 's/Cell<(u64,/Cell<(u32,/g' "$BYTECODE" > "$TMPD/mutant.rs"
if check_width "$TMPD/mutant.rs" > /dev/null 2>&1; then
  echo "SELF-TEST BROKEN (KH83-1): synthetic u64->u32 revert did NOT fail the width tooth"
  exit 2
fi
echo "OK  KH83-1 mutation: synthetic u64->u32 revert fails the width tooth (bites)"

n=$(count_tokens "$HIR")
if [ "$n" -ne 0 ]; then
  echo "FAIL: hir.rs (Program graph, A-PP13) has $n interior-mutability tokens, pinned 0:"
  grep -nE "$TOKEN_RE" "$HIR" | grep -v '^\s*//' | head -5
  FAILS=$((FAILS+1))
else
  echo "OK  hir.rs (Program graph): 0 interior-mutability tokens (A-PP13)"
fi

# --- 2. mechanism: epoch-invalidation tests exist and RUN --------------------
for t in prop_ic_equality_is_structural_only method_ic_equality_is_structural_only; do
  if ! grep -q "fn $t" "$VMMOD"; then
    echo "FAIL: mechanism test $t missing from vm/mod.rs (KH81-3)"; FAILS=$((FAILS+1))
  fi
done
if ! grep -q 'epoch bump invalidates' "$VMMOD"; then
  echo "FAIL: no 'epoch bump invalidates' assertion in vm/mod.rs"; FAILS=$((FAILS+1))
fi
TESTOUT=$(cd "$REPO" && cargo test --release -p php-runtime --lib ic_equality_is_structural_only 2>&1 | tail -3)
if echo "$TESTOUT" | grep -q "2 passed"; then
  echo "OK  mechanism: both epoch-invalidation tests PASS under cargo"
else
  echo "FAIL: epoch-invalidation tests did not both pass:"; echo "$TESTOUT"
  FAILS=$((FAILS+1))
fi

# --- 3. wiring: bump called in non-test Vm construction ----------------------
# A-PP15 class (same commit): the arming line is ANCHORED to a real module
# declaration — an unanchored /mod tests/ is blinded by a comment mentioning it.
NCALL=$(awk '
  /^[[:space:]]*(pub[[:space:]]+)?mod tests/ { in_tests = 1 }
  in_tests { next }
  /^[[:space:]]*\/\// { next }
  /bump_ic_epoch\(\);/ { n++ }
  END { print n + 0 }
' "$VMMOD")
if [ "$NCALL" -ne 1 ]; then
  echo "FAIL: bump_ic_epoch() called $NCALL times in non-test vm/mod.rs, pinned 1 (the Vm constructor)"
  FAILS=$((FAILS+1))
else
  echo "OK  wiring: bump_ic_epoch() called once in Vm construction (non-test)"
fi

if [ "$FAILS" = 0 ]; then
  echo "== DR-1 PASS: Module IC epoch-guarded + Program graph immutable, machine-checked [git $GIT_REV] =="
  exit 0
else
  echo "== DR-1 FAIL($FAILS) [git $GIT_REV] =="
  exit 1
fi
