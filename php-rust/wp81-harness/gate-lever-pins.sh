#!/bin/bash
# gate-lever-pins.sh — S-81.0 (Council WP-82): machine pins of the A-BB6
# lever's structural contract. Four teeth, each with a positive decoy
# (WP-72: a detector never seen firing proves nothing):
#
#   1. A-MS13/KS-MS-82-1: `vm_new(` / `RetainSet::new(` / `park_main(`
#      NON-TEST call-sites are ALLOWLISTED per file — the vm_new door is the
#      only way to reuse a RetainSet, so a new caller outside the sealed
#      SAPI paths = lever rejected until re-audit (bump = same-commit, named).
#      A-MS17 (Council WP-84, KS-MS-84-1): the door now also demands the
#      VmGate ZST token — rustc is the JUDGE (a new call-site cannot
#      compile without a mint); this awk sweep is demoted to BELT and pins
#      the three mint sites BY NAME (section 1b).
#   2. A-PP16: in BOTH SAPI files the publish call (`publish_if_armed()`)
#      sits lexically AFTER the `link_fatal_check(` call — the put position
#      relative to link is PART of the design79 §6 contract (F8c).
#   3. KS-PP-82-3: in execute_with_retain the SplitDrain guard construction
#      (`let mut split_drain`) precedes the FIRST `return` of the body — a
#      new early-return above it would leave census residue undrained.
#   4. A-TH14: probe on/off is ONE parameter at the SAPI boundary — exactly
#      one `main_unit_acquire(..., true)` call-site (the worker) and exactly
#      one `run_source_probed(..., true)` call-site (the cli-server); the
#      one-shot paths reach the same code with probe=false only.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
GIT_REV="$(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo 'no-git')"
FAILS=0

WORKER="$REPO/crates/php-server/src/worker_pool.rs"
VMMOD="$REPO/crates/php-runtime/src/vm/mod.rs"
CLISRV="$REPO/crates/php-cli/src/server.rs"

# Count NON-TEST, non-comment occurrences of a fixed token in a file.
# The arming line for the test region is ANCHORED (A-PP15 class).
count_nontest() { # <file> <token-regex> -> count
  awk -v re="$2" '
    /^[[:space:]]*(pub[[:space:]]+)?mod tests/ { in_tests = 1 }
    in_tests { next }
    /^[[:space:]]*\/\// { next }
    { n += gsub(re, "&") }
    END { print n + 0 }
  ' "$1"
}

# --- self-test: the counter must bite (and respect test/comment arms) --------
TMPD=$(mktemp -d)
trap 'rm -rf "$TMPD"' EXIT
cat > "$TMPD/decoy.rs" <<'EOF'
fn f() {
    let vm = vm_new(&retain, &m, &r, None);
    // vm_new( in a comment does not count
}
mod tests {
    fn g() { let vm = vm_new(&retain, &m, &r, None); }
}
EOF
n=$(count_nontest "$TMPD/decoy.rs" 'vm_new[(]')
if [ "$n" -ne 1 ]; then
  echo "SELF-TEST BROKEN: decoy vm_new count $n != 1"; exit 2
fi
echo "OK  self-test: non-test counter bites (decoy=1; comment+test excluded)"

# --- 1. A-MS13 allowlist (pinned counts, bump-in-commit BY NAME) -------------
# vm_new( non-test call-sites (A-MS19: this comment used to say "vm/mod.rs=2
# ... eval-image sub-VM" while the pin below is 1 — doc drift corrected
# same-commit; false doc is worse than none, WP-78):
#   vm/mod.rs      = 1 (run_module_with_hir — the ONLY production vm_new)
#   worker_pool.rs = 1 (execute_with_retain)
#   ALL other .rs  = 0
check_pin() { # <file> <token> <want> <label>
  local n
  n=$(count_nontest "$1" "$2")
  if [ "$n" -ne "$3" ]; then
    echo "FAIL: ${1##*/} has $n non-test '$4' sites, pinned $3 (A-MS13/KS-MS-82-1)"
    FAILS=$((FAILS+1))
  else
    echo "OK  ${1##*/}: $n '$4' non-test sites (pinned)"
  fi
}
check_pin "$VMMOD"  'vm_new[(]'        1               'vm_new('   # run_module_with_hir (named: vm/mod.rs:815)
check_pin "$WORKER" 'vm_new[(]'        1               'vm_new('
check_pin "$CLISRV" 'vm_new[(]'        0               'vm_new('
check_pin "$VMMOD"  'RetainSet::new[(]' 1              'RetainSet::new('  # run_module_with_hir
check_pin "$WORKER" 'RetainSet::new[(]' 1              'RetainSet::new('
check_pin "$VMMOD"  'park_main[(]'      2              'park_main('  # impl RetainSet (def) + run_module_with_hir
check_pin "$WORKER" 'park_main[(]'      1              'park_main('
# Sweep: no OTHER .rs file in the workspace may call vm_new(/park_main(
# outside tests (new caller = lever rejected until named here).
SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' \
          ! -path "$VMMOD" ! -path "$WORKER" ! -path "$CLISRV" -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" 'vm_new[(]|park_main[(]')
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$SWEEP" ]; then
  echo "FAIL: vm_new(/park_main( non-test call-site outside the allowlist (A-MS13):"
  echo "$SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no vm_new(/park_main( non-test sites outside the allowlist"
fi

# --- 1b. A-MS17 belt: the VmGate mints, pinned BY NAME -----------------------
# rustc is the judge (private constructor); this belt pins WHERE tokens are
# minted so a new mint is a named bump, never silent:
#   VmGate(()) tokens: vm/mod.rs == 4 (the `struct VmGate(());` DECLARATION
#     itself + run_module_with_hir mint 1 + MainUnit::vm_gate mint 2 +
#     vm_gate_probe mint 3); 0 everywhere else.
#   vm_gate_probe( non-test: vm/mod.rs == 1 (the fn def itself); 0 elsewhere
#     (its only call-sites are the two hand-replicated lifecycle tests in
#     worker_pool.rs `mod tests` — non-test callers are banned).
#   .vm_gate( production call: worker_pool.rs == 1 (execute_with_retain).
check_pin "$VMMOD"  'VmGate[(][(][)][)]' 4 'VmGate(())'
check_pin "$WORKER" 'VmGate[(][(][)][)]' 0 'VmGate(())'
check_pin "$CLISRV" 'VmGate[(][(][)][)]' 0 'VmGate(())'
check_pin "$VMMOD"  'vm_gate_probe[(]'   1 'vm_gate_probe('
check_pin "$WORKER" 'vm_gate_probe[(]'   0 'vm_gate_probe('
check_pin "$WORKER" 'vm_gate[(]'         1 'vm_gate('
GATE_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' \
          ! -path "$VMMOD" ! -path "$WORKER" ! -path "$CLISRV" -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" 'VmGate[(][(][)][)]|vm_gate_probe[(]')
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$GATE_SWEEP" ]; then
  echo "FAIL: VmGate mint outside the allowlist (A-MS17):"
  echo "$GATE_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no VmGate mint outside the allowlist (A-MS17)"
fi

# --- 2. A-PP16: publish AFTER link_fatal_check, in both SAPI files -----------
# NOTE (A-TH20): this lexical order pin is POSITION only — it is guard-blind
# (an unconditional publish lexically after the check would pass). The
# BEHAVIORAL falsifier of the guard is F8c-contatori in
# gate-lever-fixtures.sh (double link-fatal => put==0/hit==0; fix => put==1):
# the two teeth judge together, neither alone.
check_order() { # <file> <first-re> <second-re> <label> -> rc 1 on violation
  awk -v a="$2" -v b="$3" -v lbl="$4" -v f="${1##*/}" '
    /^[[:space:]]*\/\// { next }   # A-TH20: comments must not satisfy order pins
    $0 ~ a && !la { la = NR }
    $0 ~ b && !lb { lb = NR }
    END {
      if (!la || !lb) { printf "FAIL: %s: pattern missing for %s (a@%d b@%d)\n", f, lbl, la, lb; exit 1 }
      if (lb < la)    { printf "FAIL: %s: %s out of order (%d before %d)\n", f, lbl, lb, la; exit 1 }
      printf "OK  %s: %s (line %d after %d)\n", f, lbl, lb, la
    }
  ' "$1"
}
check_order "$WORKER" 'link_fatal_check[(]' 'publish_if_armed[(][)]' 'A-PP16 put-after-link (worker)' || FAILS=$((FAILS+1))
check_order "$VMMOD"  'let link_fatal = vm[.]link_fatal_check[(]' 'publish_if_armed[(][)]' 'A-PP16 put-after-link (run_module_with_hir)' || FAILS=$((FAILS+1))
# A-TH20 (Council WP-83): EXACT pins, no more `>=`. Call-sites (empty-paren
# spelling) == 1 per SAPI file; total `publish_if_armed(` tokens in vm/mod.rs
# == 2 EXACT (the impl def + the one guarded call) — a second call-site
# after run_module_with_hir:828 can no longer pass.
n=$(count_nontest "$WORKER" 'publish_if_armed[(][)]')
m=$(count_nontest "$VMMOD" 'publish_if_armed[(][)]')
t=$(count_nontest "$VMMOD" 'publish_if_armed[(]')
if [ "$n" -ne 1 ] || [ "$m" -ne 1 ] || [ "$t" -ne 2 ]; then
  echo "FAIL: publish_if_armed pins — worker calls=$n (pin 1), vm/mod.rs calls=$m (pin 1), vm/mod.rs tokens=$t (pin 2 = def+call) (A-TH20)"
  FAILS=$((FAILS+1))
else
  echo "OK  publish_if_armed: worker=1 call, vm/mod.rs=1 call + def (==2 tokens exact, A-TH20)"
fi
# decoy: reversed order must be flagged
cat > "$TMPD/rev.rs" <<'EOF'
fn f() {
    lever.publish_if_armed();
    let link_fatal = vm.link_fatal_check(&m);
}
EOF
if check_order "$TMPD/rev.rs" 'link_fatal_check[(]' 'publish_if_armed[(][)]' decoy > /dev/null 2>&1; then
  echo "SELF-TEST BROKEN: reversed publish/link order not flagged (A-PP16)"; exit 2
fi
echo "OK  self-test: reversed publish/link order flagged (decoy)"

# --- 3. KS-PP-82-3: SplitDrain before the first return -----------------------
check_splitdrain() { # <file> -> rc 1 on violation
  awk '
    /fn execute_with_retain\(/ { inside = 1 }
    inside && /let mut split_drain/ && !sd { sd = NR }
    inside && /^[[:space:]]*return / && !fr { fr = NR }
    END {
      if (!sd)      { print "FAIL: split_drain construction not found (KS-PP-82-3)"; exit 1 }
      if (fr && fr < sd) { printf "FAIL: return at %d precedes SplitDrain at %d (KS-PP-82-3)\n", fr, sd; exit 1 }
      printf "OK  SplitDrain (line %d) precedes the first return (line %d) in execute_with_retain\n", sd, fr
    }
  ' "$1"
}
check_splitdrain "$WORKER" || FAILS=$((FAILS+1))
# decoy (A-SK24, Council WP-83): tooth 3 was the ONLY one without a negative —
# a return ABOVE the SplitDrain construction must be flagged.
cat > "$TMPD/sd.rs" <<'EOF'
fn execute_with_retain() {
    return early;
    let mut split_drain = SplitDrain::new();
}
EOF
if check_splitdrain "$TMPD/sd.rs" > /dev/null 2>&1; then
  echo "SELF-TEST BROKEN: return-before-SplitDrain not flagged (KS-PP-82-3 decoy)"; exit 2
fi
echo "OK  self-test: return-before-SplitDrain flagged (decoy)"

# --- 4. A-TH14: probe = ONE parameter, pinned call-sites ----------------------
n=$(count_nontest "$WORKER" 'main_unit_acquire[(]name, source, reg, true[)]')
if [ "$n" -ne 1 ]; then
  echo "FAIL: worker main_unit_acquire(..., true) sites = $n, pinned 1 (A-TH14)"
  FAILS=$((FAILS+1))
else
  echo "OK  worker: exactly 1 probed acquire call-site"
fi
n=$(count_nontest "$CLISRV" 'run_source_probed[(]&name, &source, registry, &[[][]], None, true[)]')
if [ "$n" -ne 1 ]; then
  echo "FAIL: cli-server run_source_probed(..., None, true) sites = $n, pinned 1 (A-TH14)"
  FAILS=$((FAILS+1))
else
  echo "OK  cli-server: exactly 1 probed run call-site"
fi
# A-TH21 (Council WP-83): ALL one-shot wrappers delegate to the acquire path
# with probe=false — none may own a lexical lower+compile of its own.
for pat in \
  'run_source_probed(name, source, registry, ini_overrides, None, false)' \
  'run_source_probed(name, source, registry, &\[\], None, false)' \
  'run_source_probed(name, source, registry, ini_overrides, Some(argv), false)'; do
  if ! grep -q "$pat" "$VMMOD"; then
    echo "FAIL: one-shot wrapper delegation missing: '$pat' (A-TH21)"
    FAILS=$((FAILS+1))
  fi
done
echo "OK  one-shot wrappers (with/with_ini/with_argv) all delegate with probe=false (A-TH21)"
# The old duplicate bodies are BANNED: pin the non-test 'crate::lower_source('
# count in vm/mod.rs so a new wrapper cannot quietly re-grow its own main
# compile. NAMED sites (bump = same-commit, named — KS-AH-81-3 class):
#   1. main_unit_acquire        (the ONE main lower path)
#   2. include-lower fallback   (Vm include path, main_hir.is_none() arm)
#   3..9. retained_walk_selftest (mem-census PUB selftest — outside mod tests
#        by design: lib-tests of an allocator-linking instrument live as
#        pub-selftests called from the bin, WP-81 lesson): 2 module-dedup
#        lowers + 1 program-dedup lower (A-DL16) + 4 bracket-control lowers
#        (warm, small, small2, big — A-DL15). Bumped 4->9 S-82.0 p5/p6.
n=$(count_nontest "$VMMOD" 'crate::lower_source[(]')
if [ "$n" -ne 9 ]; then
  echo "FAIL: vm/mod.rs has $n non-test 'crate::lower_source(' sites, pinned 9 NAMED (A-TH21)"
  FAILS=$((FAILS+1))
else
  echo "OK  vm/mod.rs: crate::lower_source( == 9 named sites (acquire, include-fallback, selftest x7)"
fi

# --- 4a2. A-AH26/KS-AH-83-3: MAIN_CHAIN_FP single-binding pins ----------------
# `main_chain_fp_from` is module-private with a doc contract ("MUST NOT call
# with anything but PRELUDE_SRC") — prose. The machine pins: exactly 2
# non-test tokens in lower/mod.rs (the def + the ONE production call), and
# the production call is textually `main_chain_fp_from(PRELUDE_SRC`. A
# call-site outside the pin or with a masked copy of the binding =>
# lever de-certified, MAIN_CHAIN_FP back to ADVISORY (KS-AH-83-3).
LOWERMOD="$REPO/crates/php-runtime/src/lower/mod.rs"
# The falsifier lives in `#[cfg(test)] mod main_chain_fp_tests` — NOT named
# `mod tests`, so count_nontest would count it (the Matsakis Q3 prefix-match
# class, live). This tooth arms on the #[cfg(test)] attribute instead —
# declared: valid because in lower/mod.rs the cfg(test) region is terminal.
count_noncfgtest() { # <file> <token-regex> -> count
  awk -v re="$2" '
    /^[[:space:]]*#\[cfg\(test\)\]/ { in_t = 1 }
    in_t { next }
    /^[[:space:]]*\/\// { next }
    { n += gsub(re, "&") }
    END { print n + 0 }
  ' "$1"
}
n=$(count_noncfgtest "$LOWERMOD" 'main_chain_fp_from[(]')
if [ "$n" -ne 2 ]; then
  echo "FAIL: lower/mod.rs main_chain_fp_from( tokens = $n, pinned 2 (def+call, A-AH26)"
  FAILS=$((FAILS+1))
elif ! grep -q 'main_chain_fp_from(PRELUDE_SRC' "$LOWERMOD"; then
  echo "FAIL: production call is not textually main_chain_fp_from(PRELUDE_SRC (A-AH26/KS-AH-83-3)"
  FAILS=$((FAILS+1))
else
  echo "OK  main_chain_fp_from: ==2 tokens (def+call), call arg is PRELUDE_SRC literal (A-AH26)"
fi
# sweep: no caller anywhere else in the workspace
SWEEPFP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' ! -path "$LOWERMOD" -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" 'main_chain_fp_from[(]')
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$SWEEPFP" ]; then
  echo "FAIL: main_chain_fp_from( call-site outside lower/mod.rs (KS-AH-83-3):"
  echo "$SWEEPFP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: main_chain_fp_from( confined to lower/mod.rs"
fi

# --- 4b. KS-SK-83-4/A-SK24: CLASS sweep, any spelling ------------------------
# Exactly ONE probed-acquire call-site and ONE probed-run call-site in the
# WHOLE workspace, matched as a CLASS (`…true)`), not as a pinned spelling —
# a second call-site with different arg names must trip this even though the
# tooth-4 literal pins would stay green.
sweep_class() { # <token-class-regex> <want> <label>
  local total=0 n f
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" "$1")
    total=$((total+n))
  done < <(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0)
  if [ "$total" -ne "$2" ]; then
    echo "FAIL: class sweep '$3' = $total workspace-wide, pinned $2 (KS-SK-83-4)"
    FAILS=$((FAILS+1))
  else
    echo "OK  class sweep: $3 == $2 workspace-wide (any spelling)"
  fi
}
sweep_class 'main_unit_acquire[(][^)]*true[)]' 1 'main_unit_acquire(...true)'
sweep_class 'run_source_probed[(][^)]*true[)]' 1 'run_source_probed(...true)'
# decoy: a differently-spelled probed call must be counted by the class regex
cat > "$TMPD/cls.rs" <<'EOF'
fn f() { let u = main_unit_acquire(nm, src, r, true); }
EOF
n=$(count_nontest "$TMPD/cls.rs" 'main_unit_acquire[(][^)]*true[)]')
if [ "$n" -ne 1 ]; then
  echo "SELF-TEST BROKEN: class regex missed alt spelling (KS-SK-83-4 decoy)"; exit 2
fi
echo "OK  self-test: class sweep catches alternative spellings (decoy)"

# --- 5. A-DS17: compile purity grep-gate (DR-1 style) ------------------------
# compile_program must stay a pure function of (Program, Registry, reg_mode):
# no iteration over a Hash* container may feed an emission (hash order is
# nondeterministic across processes — a baked order would break the cached
# main's determinism-from-virgin-fp). Pin: ZERO direct iteration on
# hash-suffixed containers in compile/ (behavioral tooth = the in-cargo
# double-compile test, KS-DS-83-2 — the two judge together).
COMPILE_DIR="$REPO/crates/php-runtime/src/compile"
HASHITER=$(grep -rnE '(_index|_map|_set|conditional_fns|conditional_classes|stubs)\.(iter|keys|values)\(' \
  "$COMPILE_DIR" --include='*.rs' 2>/dev/null | grep -v '^\s*//' || true)
if [ -n "$HASHITER" ]; then
  echo "FAIL: hash-container iteration in compile/ (A-DS17 — re-audit before any emission feeds on it):"
  echo "$HASHITER" | head -5
  FAILS=$((FAILS+1))
else
  echo "OK  compile/: zero hash-container iterations (A-DS17 pin ==0)"
fi
# decoy: the pattern must bite
printf 'fn f() { for k in class_index.keys() { emit(k); } }\n' > "$TMPD/hi.rs"
if ! grep -qE '(_index|_map|_set)\.(iter|keys|values)\(' "$TMPD/hi.rs"; then
  echo "SELF-TEST BROKEN: hash-iteration decoy not matched (A-DS17)"; exit 2
fi
echo "OK  self-test: hash-iteration decoy matched (A-DS17)"

if [ "$FAILS" = 0 ]; then
  echo "== GATE-LEVER-PINS PASS (A-MS13 + A-PP16 + KS-PP-82-3 + A-TH14) [git $GIT_REV] =="
  exit 0
else
  echo "== GATE-LEVER-PINS FAIL($FAILS) [git $GIT_REV] =="
  exit 1
fi
