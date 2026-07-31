#!/bin/bash
# gate-lever-pins.sh — S-81.0 (Council WP-82): machine pins of the A-BB6
# lever's structural contract. Four teeth, each with a positive decoy
# (WP-72: a detector never seen firing proves nothing):
#
#   1. A-MS13/KS-MS-82-1: `vm_new(` / `RetainSet::new(` / `park_main(`
#      NON-TEST call-sites are ALLOWLISTED per file — the vm_new door is the
#      only way to reuse a RetainSet, so a new caller outside the sealed
#      SAPI paths = lever rejected until re-audit (bump = same-commit, named).
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
# vm_new( non-test call-sites:
#   vm/mod.rs      = 2 (run_module_with_hir + eval-image sub-VM if any; see
#                       `grep -n 'vm_new(' vm/mod.rs` for the named list)
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

# --- 2. A-PP16: publish AFTER link_fatal_check, in both SAPI files -----------
check_order() { # <file> <first-re> <second-re> <label> -> rc 1 on violation
  awk -v a="$2" -v b="$3" -v lbl="$4" -v f="${1##*/}" '
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
n=$(count_nontest "$WORKER" 'publish_if_armed[(][)]')
m=$(count_nontest "$VMMOD" 'publish_if_armed[(][)]')
if [ "$n" -ne 1 ] || [ "$m" -lt 1 ]; then
  echo "FAIL: publish_if_armed() call-sites worker=$n (pin 1) vm/mod.rs=$m (>=1: impl+call)"
  FAILS=$((FAILS+1))
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
awk '
  /fn execute_with_retain\(/ { inside = 1 }
  inside && /let mut split_drain/ && !sd { sd = NR }
  inside && /^[[:space:]]*return / && !fr { fr = NR }
  END {
    if (!sd)      { print "FAIL: split_drain construction not found (KS-PP-82-3)"; exit 1 }
    if (fr && fr < sd) { printf "FAIL: return at %d precedes SplitDrain at %d (KS-PP-82-3)\n", fr, sd; exit 1 }
    printf "OK  SplitDrain (line %d) precedes the first return (line %d) in execute_with_retain\n", sd, fr
  }
' "$WORKER" || FAILS=$((FAILS+1))

# --- 4. A-TH14: probe = ONE parameter, pinned call-sites ----------------------
n=$(count_nontest "$WORKER" 'main_unit_acquire[(]name, source, reg, true[)]')
if [ "$n" -ne 1 ]; then
  echo "FAIL: worker main_unit_acquire(..., true) sites = $n, pinned 1 (A-TH14)"
  FAILS=$((FAILS+1))
else
  echo "OK  worker: exactly 1 probed acquire call-site"
fi
n=$(count_nontest "$CLISRV" 'run_source_probed[(]&name, &source, registry, &[[][]], true[)]')
if [ "$n" -ne 1 ]; then
  echo "FAIL: cli-server run_source_probed(..., true) sites = $n, pinned 1 (A-TH14)"
  FAILS=$((FAILS+1))
else
  echo "OK  cli-server: exactly 1 probed run call-site"
fi
# the one-shot wrapper must pass false (the corpus never probes):
if ! grep -q 'run_source_probed(name, source, registry, ini_overrides, false)' "$VMMOD"; then
  echo "FAIL: run_source_with_ini does not delegate with probe=false (A-TH14/F-oneshot)"
  FAILS=$((FAILS+1))
else
  echo "OK  one-shot wrapper delegates with probe=false"
fi

if [ "$FAILS" = 0 ]; then
  echo "== GATE-LEVER-PINS PASS (A-MS13 + A-PP16 + KS-PP-82-3 + A-TH14) [git $GIT_REV] =="
  exit 0
else
  echo "== GATE-LEVER-PINS FAIL($FAILS) [git $GIT_REV] =="
  exit 1
fi
