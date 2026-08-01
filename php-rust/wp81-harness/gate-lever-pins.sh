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
#      VmGate token — rustc is the JUDGE (a new call-site cannot
#      compile without a mint); this awk sweep is demoted to BELT and pins
#      the three mint sites BY NAME (section 1b). v2 (Council WP-85,
#      A-MS25/A-MS26/A-TH27/A-TH28): the token is LIFETIME-BOUND to its
#      mint origin (no banking beyond the acquire, KS-MS-85-1) and the
#      probe mint is cfg-gated out of campaign builds (KH85-1); section 1c
#      pins the ONE producer site of main_program:Some (A-TH31/KH85-3).
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

# --- 1b. A-MS17 v2 belt: the VmGate mints, pinned BY NAME --------------------
# rustc is the judge (private constructor + LIFETIME BOUND + cfg-gated probe,
# Council WP-85 A-MS25/A-MS26/A-TH27/A-TH28); this belt pins WHERE tokens
# are minted so a new mint is a named bump, never silent:
#   VmGate(std::marker::PhantomData) mint expressions: vm/mod.rs == 3
#     (RetainSet::production_gate mint 1 + MainUnit::vm_gate mint 2 +
#     vm_gate_probe mint 3); 0 everywhere else.
#   struct VmGate< lifetime-bound declaration: vm/mod.rs == 1 (a revert to
#     the ownable ZST `struct VmGate(())` fails THIS pin — KS-MS-85-1).
#   production_gate( : vm/mod.rs == 2 (private def + run_module_with_hir).
#   vm_gate_probe( non-test: vm/mod.rs == 1 (the cfg-gated fn def itself);
#     0 elsewhere (only call-sites = the two hand-replicated lifecycle
#     tests in worker_pool.rs `mod tests` — non-test callers are banned).
#   cfg gate `any(test, feature = "vm-gate-probe")`: vm/mod.rs == 1 (fn) and
#     lib.rs == 1 (re-export) — a probe reachable in a campaign build is
#     KH85-1 (sigillo declassato ad ADVISORY) — plus lib.rs re-export == 1.
#   A-MS30 (Council WP-86, declared rename): Council WP-85 ordered the
#     feature name `lifecycle-probe`; it was implemented as `vm-gate-probe`.
#     Substance equal, ONE name in the corpus: the old name must never
#     appear in crates/ (pinned below at 0).
#   .vm_gate( production call: worker_pool.rs == 1 (execute_with_retain).
LIBRS="$REPO/crates/php-runtime/src/lib.rs"
# A-TH32 (Council WP-86): the mint CLASS `VmGate(` — every spelling, not
# just `VmGate(std::marker::PhantomData)` (a `use std::marker::PhantomData`
# made the old pin blind) — is ==3 in vm/mod.rs AND all three sit INSIDE
# the leaf `mod gate` (where the field is private: rustc judges in-module).
check_pin "$VMMOD"  'VmGate[(]' 3 'VmGate( (mint class, every spelling)'
GATE_MINTS=$(awk '/^pub\(crate\) mod gate \{/{on=1}
  on && /^\}/{on=0}
  !on{next} /^[[:space:]]*\/\//{next}
  {n+=gsub(/VmGate[(]/,"&")} END{print n+0}' "$VMMOD")
if [ "$GATE_MINTS" -ne 3 ]; then
  echo "FAIL: VmGate( mints inside mod gate == $GATE_MINTS, pinned 3 — a mint escaped the leaf module (A-TH32)"
  FAILS=$((FAILS+1))
else
  echo "OK  vm/mod.rs: all 3 VmGate( mints live inside the leaf mod gate (A-TH32)"
fi
# A-MS29/KS-MS-86-3: no VmGate<'static> anywhere in the workspace — the
# probe mint is lifetime-bound too (anchor borrow).
STATIC_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" "VmGate<'static>")
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$STATIC_SWEEP" ]; then
  echo "FAIL: VmGate<'static> found — sigillo ADVISORY (A-MS29/KS-MS-86-3):"
  echo "$STATIC_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no VmGate<'static> in the workspace (A-MS29/KS-MS-86-3)"
fi
check_pin "$WORKER" 'VmGate[(]' 0 'VmGate('
check_pin "$CLISRV" 'VmGate[(]' 0 'VmGate('
check_pin "$VMMOD"  'struct VmGate<' 1 'struct VmGate<'
check_pin "$VMMOD"  'struct VmGate[(]' 0 'struct VmGate( (v1 ownable ZST)'
check_pin "$VMMOD"  'production_gate[(]' 2 'production_gate('
check_pin "$VMMOD"  'vm_gate_probe[(]'   1 'vm_gate_probe('
check_pin "$WORKER" 'vm_gate_probe[(]'   0 'vm_gate_probe('
check_pin "$WORKER" 'vm_gate[(]'         1 'vm_gate('
check_pin "$VMMOD"  'cfg[(]any[(]test, feature = "vm-gate-probe"[)][)]' 1 'cfg-gate on probe fn'
check_pin "$LIBRS"  'cfg[(]any[(]test, feature = "vm-gate-probe"[)][)]' 1 'cfg-gate on re-export'
check_pin "$LIBRS"  'vm_gate_probe' 1 'vm_gate_probe re-export'
# A-TH28 anti-alias belt: NO `vm_gate_probe as` alias and NO `use` of the
# probe anywhere (tests included — the two legit call-sites are DIRECT
# `php_runtime::vm_gate_probe()` calls; an alias would blind every pin
# above). grep, not count_nontest: an alias inside `mod tests` also bites.
ALIAS_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' ! -path "$LIBRS" -print0 |
  xargs -0 grep -l -E 'vm_gate_probe[[:space:]]+as[[:space:]]|use .*vm_gate_probe' 2>/dev/null)
if [ -n "$ALIAS_SWEEP" ]; then
  echo "FAIL: vm_gate_probe alias/use outside lib.rs (A-TH28/KH85-1):"
  echo "$ALIAS_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no vm_gate_probe alias/use outside lib.rs (A-TH28)"
fi
# A-MS30: the superseded feature name must not exist anywhere in crates/
# (one name in the corpus — a second spelling would blind the cfg pins).
RENAME_SWEEP=$(find "$REPO/crates" \( -name '*.rs' -o -name '*.toml' \) ! -name '._*' -print0 |
  xargs -0 grep -l 'lifecycle-probe' 2>/dev/null)
if [ -n "$RENAME_SWEEP" ]; then
  echo "FAIL: superseded feature name lifecycle-probe found in crates/ (A-MS30):"
  echo "$RENAME_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: one feature name only — no lifecycle-probe in crates/ (A-MS30)"
fi
GATE_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' \
          ! -path "$VMMOD" ! -path "$WORKER" ! -path "$CLISRV" ! -path "$LIBRS" -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" 'VmGate[(]|vm_gate_probe')
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$GATE_SWEEP" ]; then
  echo "FAIL: VmGate mint outside the allowlist (A-MS17):"
  echo "$GATE_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no VmGate mint outside the allowlist (A-MS17)"
fi

# --- 1c. A-TH31/KH85-3: the ONE producer site of main_program:Some -----------
# The partition-by-TYPE certificate (A-MS24, UnitSlot main lane) rests on
# main entries entering the cache from ONE producer: main_publish_ticket.
# Two teeth: (a) global count of `main_program: Some(` construction == 1 in
# vm/mod.rs and 0 in every other .rs; (b) that one site sits INSIDE
# fn main_publish_ticket (body-scoped count == 1). A put of a main entry
# from any other site = partition de-certified, campaign VOID (KH85-3).
cat > "$TMPD/decoy3.rs" <<'EOF'
fn other() {
    let e = CachedUnit { main_program: Some(p) };
}
fn main_publish_ticket(t: T) {
    let e = CachedUnit { main_program: Some(p) };
}
EOF
body_scoped_some() { # <file> -> count of main_program:Some inside fn main_publish_ticket
  awk '/^fn main_publish_ticket[(]/{inf=1}
       inf && /main_program: Some[(]/{c++}
       inf && /^}/{inf=0}
       END{print c+0}' "$1"
}
n=$(body_scoped_some "$TMPD/decoy3.rs")
if [ "$n" -ne 1 ]; then
  echo "SELF-TEST BROKEN: decoy body-scoped main_program:Some count $n != 1"; exit 2
fi
echo "OK  self-test: body-scoped producer counter bites (decoy=1, out-of-body excluded)"
check_pin "$VMMOD" 'main_program: Some[(]' 1 'main_program: Some('
n=$(body_scoped_some "$VMMOD")
if [ "$n" -ne 1 ]; then
  echo "FAIL: main_program:Some inside fn main_publish_ticket == $n, pinned 1 (A-TH31/KH85-3)"
  FAILS=$((FAILS+1))
else
  echo "OK  vm/mod.rs: the ONE main_program:Some producer is main_publish_ticket (A-TH31)"
fi
PROD_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' ! -path "$VMMOD" -print0 |
  xargs -0 grep -l 'main_program: Some(' 2>/dev/null)
if [ -n "$PROD_SWEEP" ]; then
  echo "FAIL: main_program:Some producer outside vm/mod.rs (KH85-3):"
  echo "$PROD_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no main_program:Some producer outside vm/mod.rs (KH85-3)"
fi
# A-TH33 (Council WP-86, Hoare Q2): the pin above sees ONE spelling. The
# eluded producer spellings — post-hoc assignment `.main_program = Some(`
# and field-shorthand `main_program,` in construction — are 0 in the whole
# workspace. True closure remains A-MS27 (typed CachedMain/CachedInclude);
# until then "partition by construction" reads "discipline + tripwire"
# (KH86-2).
cat > "$TMPD/decoy5.rs" <<'EOF'
fn f() {
    cu.main_program = Some(p);
    let main_program = Some(p);
    let e = CachedUnit { main_program, fp };
}
EOF
# NB: awk -v processes backslash escapes, so the dot is written [.]
TH33RE='[.]main_program[[:space:]]*=[[:space:]]*Some[(]|[^.[:alnum:]_]main_program,'
n=$(count_nontest "$TMPD/decoy5.rs" "$TH33RE")
if [ "$n" -ne 2 ]; then
  echo "SELF-TEST BROKEN: A-TH33 decoy count $n != 2 (assignment + shorthand)"; exit 2
fi
echo "OK  self-test: A-TH33 eluded-spelling decoy bites (2/2)"
TH33_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" "$TH33RE")
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$TH33_SWEEP" ]; then
  echo "FAIL: eluded main_program producer spelling in workspace (A-TH33/KH85-3):"
  echo "$TH33_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no eluded main_program producer spellings (A-TH33)"
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
# A-TH23 (Council WP-84, KH84-1 — replaces the ==9 CONFLATED pin): the old
# single count admitted compensating swaps (a new prod lower + a removed
# selftest = same 9). SPLIT counts, anchored on the selftest fn marker
# (`pub fn retained_walk_selftest` — a renamed/moved anchor breaks LOUDLY):
#   prod region (file head .. anchor) == 2, NAMED:
#     1. main_unit_acquire        (the ONE main lower path)
#     2. include-lower fallback   (Vm include path, main_hir.is_none() arm)
#   selftest region (anchor .. mod tests) == 7, NAMED (mem-census PUB
#     selftest, WP-81 lesson): 2 module-dedup + 1 program-dedup (A-DL16)
#     + 4 bracket-control lowers (warm, small, small2, big — A-DL15).
ANCH=$(grep -c '^pub fn retained_walk_selftest' "$VMMOD")
if [ "$ANCH" -ne 1 ]; then
  echo "FAIL: A-TH23 anchor 'pub fn retained_walk_selftest' count $ANCH != 1 — split pin is blind"
  FAILS=$((FAILS+1))
fi
PROD_N=$(awk '/^pub fn retained_walk_selftest/{exit}
  /^[[:space:]]*\/\//{next} {n+=gsub(/crate::lower_source[(]/,"&")} END{print n+0}' "$VMMOD")
SELF_N=$(awk '/^pub fn retained_walk_selftest/{on=1}
  on && /^[[:space:]]*(pub[[:space:]]+)?mod tests/{exit}
  !on{next} /^[[:space:]]*\/\//{next} {n+=gsub(/crate::lower_source[(]/,"&")} END{print n+0}' "$VMMOD")
if [ "$PROD_N" -ne 2 ] || [ "$SELF_N" -ne 7 ]; then
  echo "FAIL: vm/mod.rs crate::lower_source( split prod=$PROD_N/selftest=$SELF_N, pinned 2/7 SEPARATE (A-TH23)"
  FAILS=$((FAILS+1))
else
  echo "OK  vm/mod.rs: crate::lower_source( prod==2 / selftest==7 SEPARATE (A-TH23, anchor-split)"
fi

# A-TH23(b): workspace CLASS sweep on lower_source(/lower_source_seeded(/
# compile_program( — per-file NAMED allowlist (the Hoare 10th site is
# NAMED here, not folded: the phpt-runner capability scan discards the
# Program and feeds no cache). A new file/count = FAIL until named.
CLASSRE='(lower_source_seeded|lower_source|compile_program)[(]'
LOWERDEFS="$REPO/crates/php-runtime/src/lower/mod.rs"
COMPDEF="$REPO/crates/php-runtime/src/compile/mod.rs"
PHPTLIB="$REPO/crates/phpt-runner/src/lib.rs"
check_class() { # <file> <want> <label>
  local n
  n=$(count_nontest "$1" "$CLASSRE")
  if [ "$n" -ne "$2" ]; then
    echo "FAIL: ${1##*/} has $n non-test lower/compile CLASS sites, pinned $2 NAMED (A-TH23)"
    FAILS=$((FAILS+1))
  else
    echo "OK  ${1##*/}: $n lower/compile CLASS sites (pinned: $3)"
  fi
}
# A-TH29 (Council WP-85) + A-TH34 (Council WP-86, Hoare Q3): the 5/9
# region aggregates killed prod↔selftest compensation, but INSIDE a region
# a seeded↔compile swap kept the aggregate invariant. Split per-SPELLING —
# three counters per region, never one number:
#   prod     (head..anchor):    seeded==2  lower_source==2  compile==1
#   selftest (anchor..mod tests): seeded==0  lower_source==7  compile==2
region_split() { # <file> <prod|self> -> "seeded plain compile"
  awk -v want="$2" '
    /^pub fn retained_walk_selftest/{infl=1}
    infl && /^[[:space:]]*(pub[[:space:]]+)?mod tests/{exit}
    /^[[:space:]]*\/\//{next}
    {
      reg = infl ? "self" : "prod"
      if (reg != want) next
      s += gsub(/lower_source_seeded[(]/, "S")
      p += gsub(/lower_source[(]/, "P")
      c += gsub(/compile_program[(]/, "C")
    }
    END { printf "%d %d %d\n", s+0, p+0, c+0 }' "$1"
}
read -r PS PP PC <<< "$(region_split "$VMMOD" prod)"
read -r SS SP SC <<< "$(region_split "$VMMOD" self)"
if [ "$PS" -ne 2 ] || [ "$PP" -ne 2 ] || [ "$PC" -ne 1 ]; then
  echo "FAIL: vm/mod.rs prod region per-spelling seeded=$PS/ls=$PP/compile=$PC, pinned 2/2/1 (A-TH34)"
  FAILS=$((FAILS+1))
else
  echo "OK  vm/mod.rs: prod region seeded==2 / lower_source==2 / compile==1 (A-TH34)"
fi
if [ "$SS" -ne 0 ] || [ "$SP" -ne 7 ] || [ "$SC" -ne 2 ]; then
  echo "FAIL: vm/mod.rs selftest region per-spelling seeded=$SS/ls=$SP/compile=$SC, pinned 0/7/2 (A-TH34)"
  FAILS=$((FAILS+1))
else
  echo "OK  vm/mod.rs: selftest region seeded==0 / lower_source==7 / compile==2 (A-TH34)"
fi
# decoy: an intra-region seeded->compile swap keeps the aggregate (3) but
# must fail the per-spelling pins.
cat > "$TMPD/swapdecoy.rs" <<'EOF'
fn prod() {
    let a = crate::lower_source(x, y);
    let b = crate::lower_source_seeded(x, y);
    let c = compile_program(&p, r);
}
EOF
read -r DS DP DC <<< "$(region_split "$TMPD/swapdecoy.rs" prod)"
if [ "$DS" -ne 1 ] || [ "$DP" -ne 1 ] || [ "$DC" -ne 1 ]; then
  echo "SELF-TEST BROKEN: A-TH34 decoy split $DS/$DP/$DC != 1/1/1"; exit 2
fi
echo "OK  self-test: A-TH34 per-spelling split discriminates (decoy 1/1/1)"
check_class "$LOWERDEFS" 2 "the two fn defs"
check_class "$COMPDEF"   1 "the fn def"
check_class "$PHPTLIB"   1 "capability scan (discards Program, feeds no cache — Hoare 10th site)"
# crates/*/tests/*.rs are cargo INTEGRATION tests — test-by-construction
# (the whole file is a test target, count_nontest's `mod tests` arm never
# fires there); excluded DECLARED, same rationale as the in-file test arm.
CLASS_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' ! -path '*/tests/*' \
    ! -path "$VMMOD" ! -path "$LOWERDEFS" ! -path "$COMPDEF" ! -path "$PHPTLIB" -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" "$CLASSRE")
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$CLASS_SWEEP" ]; then
  echo "FAIL: lower/compile CLASS call-site outside the named allowlist (A-TH23):"
  echo "$CLASS_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: lower/compile CLASS confined to the 4 named files (A-TH23)"
fi
# decoy: the class regex must bite on every spelling
printf 'fn f() { let p = my::lower_source_seeded(a, b); let m = xx::compile_program(&p, r); }\n' > "$TMPD/cls.rs"
n=$(count_nontest "$TMPD/cls.rs" "$CLASSRE")
if [ "$n" -ne 2 ]; then
  echo "SELF-TEST BROKEN: class-sweep decoy count $n != 2 (A-TH23)"; exit 2
fi
echo "OK  self-test: A-TH23 class decoy bites (seeded+compile == 2)"

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

# --- 5b. A-DS23 (Council WP-84): map INVENTORY by name + extended verbs ------
# Stogov: the suffix regex above sees neither `for … in &mappa` nor
# drain/retain/into_iter, nor maps outside the suffix convention (prop_info,
# mtab, labels…). Two teeth:
#   (i) INVENTORY: every Hash*/FxHash*/IndexMap declaration in compile/ is
#       NAMED here — a new container = FAIL until named;
#   (ii) ITERATION allowlist over the inventory names with the FULL verb set:
#       class.rs == 6 named ORDER-INSENSITIVE sites (audited S-83.0 p1b:
#       1 keyed join `for (name, hooks) in &prop_hooks` + 1 per-entry stamp
#       `prop_info.values_mut()` + 4 boolean folds `prop_info.values()`
#       any/all — none feeds an emission in iteration order); 0 elsewhere.
MAPS_GOT=$(grep -rhE '(FxHashMap|FxHashSet|HashMap|HashSet|IndexMap)<' "$COMPILE_DIR" --include='*.rs' \
  | sed -E 's/^[[:space:]]+//; s/^(pub )?(let mut |let |static )?//; s/:.*$//' | LC_ALL=C sort -u | tr '\n' ' ')
MAPS_WANT="STUBS backed_seen class_index cond_retained conditional_fns labels mtab prop_attributes prop_hooks prop_info "
if [ "$MAPS_GOT" != "$MAPS_WANT" ]; then
  echo "FAIL: compile/ map inventory drifted (A-DS23 — name the new/renamed container):"
  echo "  got:  $MAPS_GOT"
  echo "  want: $MAPS_WANT"
  FAILS=$((FAILS+1))
else
  echo "OK  compile/: map inventory == 10 named containers (A-DS23)"
fi
INVNAMES='STUBS|backed_seen|class_index|cond_retained|conditional_fns|labels|mtab|prop_attributes|prop_hooks|prop_info'
ITER_RE="($INVNAMES)\.(iter|keys|values|values_mut|into_iter|drain|retain)\(|for [^;{]* in &($INVNAMES)"
CLASSRS="$COMPILE_DIR/class.rs"
n=$(grep -cE "$ITER_RE" "$CLASSRS")
if [ "$n" -ne 6 ]; then
  echo "FAIL: class.rs has $n inventory-map iterations, pinned 6 NAMED order-insensitive (A-DS23)"
  FAILS=$((FAILS+1))
else
  echo "OK  class.rs: 6 named order-insensitive inventory iterations (A-DS23)"
fi
DS23_SWEEP=$(find "$COMPILE_DIR" -name '*.rs' ! -name '._*' ! -path "$CLASSRS" -print0 |
  while IFS= read -r -d '' f; do
    m=$(grep -cE "$ITER_RE" "$f")
    [ "$m" -gt 0 ] && echo "${f#"$REPO"/}: $m"
  done)
if [ -n "$DS23_SWEEP" ]; then
  echo "FAIL: inventory-map iteration outside class.rs allowlist (A-DS23):"
  echo "$DS23_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  compile/: no inventory-map iteration outside the class.rs allowlist (A-DS23)"
fi
# decoys: for-in-& and drain must bite
printf 'fn f() { for k in &class_index { emit(k); } prop_hooks.drain(); }\n' > "$TMPD/ds23.rs"
n=$(grep -cE "$ITER_RE" "$TMPD/ds23.rs")
if [ "$n" -lt 1 ]; then
  echo "SELF-TEST BROKEN: A-DS23 decoy not matched"; exit 2
fi
echo "OK  self-test: A-DS23 decoy bites (for-in-& / drain)"

# --- 6. A-TH24 (Council WP-84, KH84-3): epoch narrowing-cast tooth -----------
# Reintroducing `as u32` on the OBSERVED epoch at a fill/get site keeps
# Cell<u64> and passes the 1w declaration tooth — wrap in the value domain,
# invisible ("nessun as u32" in WP-83 was a hand check, never a gate).
# Pins: zero epoch-narrowing casts in bytecode.rs + vm/mod.rs; the IC cell
# tuples stay u64-FIRST (PropIc + MethodIc == 2) and IC_EPOCH stays
# Cell<u64> (==1).
BYTEC="$REPO/crates/php-runtime/src/bytecode.rs"
EPOCH_CAST='(ic_epoch[(][)][[:space:]]*|[a-z_0-9]*epoch[a-z_0-9]*[[:space:]]+)as[[:space:]]+u(32|16|8)'
for f in "$BYTEC" "$VMMOD"; do
  HITS=$(grep -inE "$EPOCH_CAST" "$f" || true)
  if [ -n "$HITS" ]; then
    echo "FAIL: epoch narrowing cast (A-TH24/KH84-3 — u64 tooth bypassed in the value domain):"
    echo "$HITS" | head -5
    FAILS=$((FAILS+1))
  else
    echo "OK  ${f##*/}: zero epoch narrowing casts (A-TH24)"
  fi
done
CELLN=$(grep -cE 'Cell<[(]u64, u32, u32, u32[)]>' "$BYTEC")
EPN=$(grep -cE 'Cell<u64>' "$BYTEC")
if [ "$CELLN" -ne 2 ] || [ "$EPN" -ne 1 ]; then
  echo "FAIL: IC cell shape drifted (A-TH24): Cell<(u64,u32,u32,u32)>=$CELLN (want 2: PropIc+MethodIc), Cell<u64>=$EPN (want 1: IC_EPOCH)"
  FAILS=$((FAILS+1))
else
  echo "OK  bytecode.rs: IC cells u64-first (PropIc+MethodIc==2) + IC_EPOCH Cell<u64>==1 (A-TH24)"
fi
# decoys: both narrowing spellings must bite
printf 'fn f() { let a = ic_epoch() as u32; let b = my_epoch as u16; }\n' > "$TMPD/ep.rs"
n=$(grep -cE "$EPOCH_CAST" "$TMPD/ep.rs")
if [ "$n" -ne 1 ]; then
  # one LINE with two matches counts once under grep -c; require the line found
  echo "SELF-TEST BROKEN: A-TH24 decoy not matched (n=$n)"; exit 2
fi
echo "OK  self-test: A-TH24 epoch-cast decoy bites"

# --- 7. A-PP23 (Council WP-84): flush-before-response, pinned LEXICALLY ------
# The twin-pair MARK cut relied on "end-of-run flush precedes send" as
# DISCIPLINE (comments only — Pedersen Q2). Machine form here:
#   (a) request_shutdown() carries the end-of-run uc_log_flush ==1 in its
#       body (the worker captures the response AFTER request_shutdown, so
#       flush precedes the response's very existence);
#   (b) worker loop: response_tx.send( sits lexically AFTER the
#       execute_request( call (same style as the A-PP16 put-after-link pin).
# The head-segment positive control is the CAMPAIGN half (verdict83).
RS_FLUSH=$(awk '/^    pub fn request_shutdown\(/{on=1; next}
  on && /^    pub fn /{exit}
  on && !/^[[:space:]]*\/\//{n+=gsub(/uc_log_flush\(\)/,"&")} END{print n+0}' "$VMMOD")
if [ "$RS_FLUSH" -ne 1 ]; then
  echo "FAIL: request_shutdown() carries $RS_FLUSH uc_log_flush() (want ==1) — flush-before-response unpinned (A-PP23)"
  FAILS=$((FAILS+1))
else
  echo "OK  vm/mod.rs: request_shutdown() end-of-run uc_log_flush ==1 (A-PP23)"
fi
EXEC_LN=$(grep -n 'execute_request(&reg, &task.meta)' "$WORKER" | head -1 | cut -d: -f1)
SEND_LN=$(grep -n 'response_tx\.send(' "$WORKER" | head -1 | cut -d: -f1)
if [ -z "$EXEC_LN" ] || [ -z "$SEND_LN" ] || [ "$EXEC_LN" -ge "$SEND_LN" ]; then
  echo "FAIL: worker loop order execute_request (line ${EXEC_LN:-absent}) !< response_tx.send (line ${SEND_LN:-absent}) (A-PP23)"
  FAILS=$((FAILS+1))
else
  echo "OK  worker_pool.rs: A-PP23 send-after-execute (line $SEND_LN after $EXEC_LN)"
fi
# decoy: the region counter must bite (flush OUTSIDE the fn body != inside)
cat > "$TMPD/pp23.rs" <<'EOF'
    pub fn request_shutdown(&mut self) {
        uc_log_flush();
    }
    pub fn other() { uc_log_flush(); }
EOF
n=$(awk '/^    pub fn request_shutdown\(/{on=1; next}
  on && /^    pub fn /{exit}
  on{n+=gsub(/uc_log_flush\(\)/,"&")} END{print n+0}' "$TMPD/pp23.rs")
if [ "$n" -ne 1 ]; then
  echo "SELF-TEST BROKEN: A-PP23 region decoy count $n != 1"; exit 2
fi
echo "OK  self-test: A-PP23 region decoy bites (flush outside body excluded)"

# --- 5. A-PP31/KS-PP-86-1 + KG-86-1 (Council WP-86): mechanical INHERITANCE
# of the verdict guards — a refusal living only in one campaign's verdict
# does not inherit by itself (three blind zeros already paid for this).
# (a) A-PP31: every wp8[4-9]/wp9x harness script that PARSES `reqns: ` lines
#     must contain the fail-closed w!=1 rejection in its own body.
#     verdict83.sh is EXCLUDED BY NAME: pre-A-PP28 archive (the w= stamp
#     did not exist), its VC figure is binario-bound to 7a610457 (A-BB43).
PP31_SWEEP=$(grep -ln 'reqns: ' "$REPO"/wp8[4-9]-harness/*.sh "$REPO"/wp9[0-9]-harness/*.sh 2>/dev/null |
  grep -v '/\._' |
  while IFS= read -r f; do
    grep -q 'w=1' "$f" || echo "${f#"$REPO"/}"
  done)
if [ -n "$PP31_SWEEP" ]; then
  echo "FAIL: script parses 'reqns: ' without the w!=1 fail-closed rejection (A-PP31/KS-PP-86-1 — verdict illegittimo):"
  echo "$PP31_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: every wp84+ reqns-parsing script carries the w!=1 rejection (A-PP31)"
fi
# decoy: a parser without the rejection must be caught
printf 'while (<$fh>) { push @s, $1 if /^reqns: (\\d+)/ }\n' > "$TMPD/pp31decoy.sh"
if grep -q 'reqns: ' "$TMPD/pp31decoy.sh" && ! grep -q 'w=1' "$TMPD/pp31decoy.sh"; then
  echo "OK  self-test: A-PP31 decoy discriminates (parser without rejection detected)"
else
  echo "SELF-TEST BROKEN: A-PP31 decoy not detected"; exit 2
fi
# (b) KG-86-1: the slope-verdict checker must EXIST and be tracked — the
#     next verdict slope is born fail-closed or is illegitimate (any slope
#     claim without gate-slope-verdict PASS is VERDICT NULL d'ufficio).
SLOPEGATE="$REPO/wp85-harness/gate-slope-verdict.sh"
if [ -f "$SLOPEGATE" ] && git -C "$REPO" ls-files --error-unmatch "wp85-harness/gate-slope-verdict.sh" >/dev/null 2>&1; then
  echo "OK  KG-86-1: gate-slope-verdict.sh exists and is tracked (slope verdicts born fail-closed)"
else
  echo "FAIL: gate-slope-verdict.sh missing/untracked (KG-86-1 — future slope verdicts have no machine)"
  FAILS=$((FAILS+1))
fi

if [ "$FAILS" = 0 ]; then
  echo "== GATE-LEVER-PINS PASS (A-MS13 + A-PP16 + KS-PP-82-3 + A-TH14) [git $GIT_REV] =="
  exit 0
else
  echo "== GATE-LEVER-PINS FAIL($FAILS) [git $GIT_REV] =="
  exit 1
fi
