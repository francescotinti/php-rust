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

# A-TH-64 (Council WP-93, Hoare Q3 — generalizes the v9 guard at the a/ml
# site): EVERY awk/grep-c capture passes through this fail-closed assert
# before any `-ne` test. `[ "" -ne N ]` errors with status 2 and the if
# treats it as FALSE: a broken program used to turn its teeth VACUOUS
# silently (the S-91.0 macOS-awk lesson: a count that is not a number is
# a broken judge). KS-TH-93-2: a non-numeric count escaping this = gate
# VOID.
num_or_void() { # <label> <value>... — exit 2 on any non-numeric capture
  local lbl="$1"; shift
  local v
  for v in "$@"; do
    case "$v" in ''|*[!0-9]*)
      echo "SELF-TEST BROKEN: non-numeric count in $lbl ('$v') — awk/grep program broken, gate VOID (A-TH-64/KS-TH-93-2)"
      exit 2;;
    esac
  done
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
num_or_void "A-TH32 GATE_MINTS" "$GATE_MINTS"
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

# --- 1d. Sigilli v4 (Council WP-87): mint-per-METHOD + anchor + literal ------
# A-MS32/KS-MS-87-1: the probe anchor is `&mut ()` — `&()` is subject to
# constant promotion (Matsakis, compiled proof: vm_gate_probe(&()) minted a
# legal 'static token without the spelling). Pin the fixed signature ==1 and
# the promotable spelling ==0.
check_pin "$VMMOD" 'vm_gate_probe[(]anchor: &mut [(][)][)]' 1 'vm_gate_probe(anchor: &mut ()) (A-MS32)'
cat > "$TMPD/decoy_ms32.rs" <<'EOF'
fn f() {
    let g = vm_gate_probe(&());
}
fn vm_gate_probe(anchor: &()) -> u8 { 0 }
EOF
n=$(count_nontest "$TMPD/decoy_ms32.rs" 'vm_gate_probe[(]&[(][)][)]|anchor: &[(][)]')
if [ "$n" -ne 2 ]; then
  echo "SELF-TEST BROKEN: A-MS32 promotable-anchor decoy count $n != 2"; exit 2
fi
echo "OK  self-test: A-MS32 promotable-anchor decoy bites (2/2)"
MS32_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" 'vm_gate_probe[(]&[(][)][)]|anchor: &[(][)]')
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$MS32_SWEEP" ]; then
  echo "FAIL: promotable probe anchor spelling (&()) found (A-MS32/KS-MS-87-1):"
  echo "$MS32_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no promotable &() probe anchor in the workspace (A-MS32)"
fi
# A-TH38/KH87-1: the leaf module closed the LITERAL mint; the mint-by-METHOD
# spellings `.production_gate(` / `.vm_gate(` are visible beyond the leaf
# (pub(super)/pub) and were BLIND to every sweep above. Named sites:
#   .production_gate( : vm/mod.rs == 1 (run_module_with_hir), 0 elsewhere
#   .vm_gate(         : worker_pool.rs == 1 (execute_with_retain), 0 elsewhere
cat > "$TMPD/decoy_th38.rs" <<'EOF'
fn f() {
    let g = retain.production_gate();
    let h = unit.vm_gate();
    let t: VmGate = unsafe { std::mem::transmute(()) };
}
impl Copy for VmGate<'_> {}
EOF
n=$(count_nontest "$TMPD/decoy_th38.rs" '[.]production_gate[(]')
m=$(count_nontest "$TMPD/decoy_th38.rs" '[.]vm_gate[(]')
if [ "$n" -ne 1 ] || [ "$m" -ne 1 ]; then
  echo "SELF-TEST BROKEN: A-TH38 method-mint decoy counts $n/$m != 1/1"; exit 2
fi
echo "OK  self-test: A-TH38 method-mint decoy bites (1/1)"
check_pin "$VMMOD"  '[.]production_gate[(]' 1 '.production_gate( method mint (A-TH38)'
check_pin "$WORKER" '[.]production_gate[(]' 0 '.production_gate( method mint (A-TH38)'
check_pin "$VMMOD"  '[.]vm_gate[(]' 0 '.vm_gate( method mint (A-TH38)'
check_pin "$WORKER" '[.]vm_gate[(]' 1 '.vm_gate( method mint (A-TH38)'
TH38_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' \
          ! -path "$VMMOD" ! -path "$WORKER" -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" '[.]production_gate[(]|[.]vm_gate[(]')
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$TH38_SWEEP" ]; then
  echo "FAIL: mint-by-method outside the named sites (A-TH38/KH87-1):"
  echo "$TH38_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no .production_gate(/.vm_gate( outside the named sites (A-TH38)"
fi
# A-TH38 second half: no transmute involving VmGate and no impl of
# Copy/Clone/Send/Sync for VmGate — anywhere, TESTS INCLUDED (a test-side
# token duplicator forges positive controls; KS-MS-87-2: such an impl is a
# Council deliberation, never a session edit). grep, not count_nontest.
if ! grep -q -E 'transmute' "$TMPD/decoy_th38.rs" || \
   ! grep -q -E 'impl[^{]*(Copy|Clone|Send|Sync)[^{]*for[[:space:]]*VmGate' "$TMPD/decoy_th38.rs"; then
  echo "SELF-TEST BROKEN: A-TH38 transmute/impl decoy not detected"; exit 2
fi
echo "OK  self-test: A-TH38 transmute/impl decoy bites"
TH38B_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  xargs -0 grep -l -E 'transmute[^;]*VmGate|VmGate[^;]*transmute|impl[^{]*(Copy|Clone|Send|Sync)[^{]*for[[:space:]]*VmGate' 2>/dev/null)
if [ -n "$TH38B_SWEEP" ]; then
  echo "FAIL: transmute/forbidden-impl on VmGate (A-TH38/KS-MS-87-2):"
  echo "$TH38B_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no transmute/Copy/Clone/Send/Sync impl on VmGate (A-TH38)"
fi
# A-TH39: (a) functional update `..` inside a CachedUnit literal copies
# main_program:Some without ANY pinned spelling — 0 in the workspace,
# tests included (struct decl excluded by the `struct` guard);
# (b) TH33RE extended: the carrier spelling `main_program: <var>`, the
# no-space `main_program:Some(` and the `main_program :` variants elude
# both the producer pin and TH33RE — 0 non-test.
cat > "$TMPD/decoy_th39.rs" <<'EOF'
struct CachedUnit { main_program: Option<u8> }
fn f() {
    let a = CachedUnit { fp, ..other };
    let b = CachedUnit {
        module,
        ..base
    };
    let c = CachedUnit { main_program: mp };
    let d = CachedUnit { main_program:Some(p) };
    let e = CachedUnit { main_program : Some(p) };
}
EOF
cu_functional_update() { # <file> -> count of `..` inside CachedUnit literals
  awk '/struct CachedUnit/ { next }
       /CachedUnit[[:space:]]*\{/ { depth = 1; n += gsub(/[.][.]/, "&"); next }
       depth > 0 {
         n += gsub(/[.][.]/, "&")
         open = gsub(/\{/, "&"); close_ = gsub(/\}/, "&")
         depth += open - close_
         if (depth <= 0) depth = 0
       }
       END { print n + 0 }' "$1"
}
n=$(cu_functional_update "$TMPD/decoy_th39.rs")
if [ "$n" -ne 2 ]; then
  echo "SELF-TEST BROKEN: A-TH39 functional-update decoy count $n != 2"; exit 2
fi
TH39RE='main_program:[^[:space:]]|main_program[[:space:]]+:|main_program:[[:space:]]+[a-z_]'
n=$(count_nontest "$TMPD/decoy_th39.rs" "$TH39RE")
if [ "$n" -ne 3 ]; then
  echo "SELF-TEST BROKEN: A-TH39 eluded-spelling decoy count $n != 3"; exit 2
fi
echo "OK  self-test: A-TH39 decoys bite (functional-update 2/2, eluded spellings 3/3)"
TH39A_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  while IFS= read -r -d '' f; do
    n=$(cu_functional_update "$f")
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$TH39A_SWEEP" ]; then
  echo "FAIL: functional update (..) inside a CachedUnit literal (A-TH39):"
  echo "$TH39A_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no functional update inside CachedUnit literals (A-TH39)"
fi
TH39B_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" "$TH39RE")
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$TH39B_SWEEP" ]; then
  echo "FAIL: eluded main_program spelling — carrier/no-space/pre-colon (A-TH39):"
  echo "$TH39B_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no carrier/no-space/pre-colon main_program spellings (A-TH39)"
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
num_or_void "A-TH23 PROD_N/SELF_N" "$PROD_N" "$SELF_N"
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
num_or_void "A-PP23 RS_FLUSH" "$RS_FLUSH"
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
# A-PP35 (Council WP-87, KS-PP-87-1): the old sweep was `grep -q 'w=1'` —
# satisfied by `w=10` or a `# TODO w=1` comment: token-presence is not a
# rejection. Two-tier form: (via A, closure) a script that SOURCES the
# shared tracked parser wp86-harness/reqns-guard.pl (own bite-test:
# synthetic w=2 => VOID) is compliant BY NAME; (immediate) otherwise the
# script must carry the DIGIT-GUARDED rejection `w=1[^0-9]` in its body.
# A-PP40 (Council WP-88, KS-PP-88-1): tier A is EXECUTIVE — a `# TODO
# reqns-guard.pl` comment satisfied the old token-presence check (the
# KS-PP-87-1 class one level up). The invocation must be a NON-comment
# line that runs perl with the guard; scan extended to .pl helpers
# (reqns-guard.pl itself excluded BY NAME: it IS the shared parser).
PP31_SWEEP=$(grep -ln 'reqns: ' "$REPO"/wp8[4-9]-harness/*.sh "$REPO"/wp9[0-9]-harness/*.sh \
    "$REPO"/wp8[4-9]-harness/*.pl "$REPO"/wp9[0-9]-harness/*.pl 2>/dev/null |
  grep -v '/\._' | grep -v '/reqns-guard[.]pl$' |
  while IFS= read -r f; do
    if grep -qE '^[^#]*perl[^#]*reqns-guard[.]pl' "$f"; then :
    elif grep -qE 'w=1([^0-9]|$)' "$f"; then :
    else echo "${f#"$REPO"/}"; fi
  done)
if [ -n "$PP31_SWEEP" ]; then
  echo "FAIL: script parses 'reqns: ' without digit-guarded w=1 rejection nor reqns-guard.pl (A-PP31/A-PP35/KS-PP-87-1):"
  echo "$PP31_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: every wp84+ reqns-parsing script rejects w!=1 (digit-guard or shared parser, A-PP35)"
fi
# decoys: (a) parser without any rejection; (b) parser whose only token is
# w=10; (c) A-PP40: parser whose only reqns-guard occurrence is a COMMENT
# — ALL must be caught (the WP-87 vacuity finding + its WP-88 recidiva).
printf 'while (<$fh>) { push @s, $1 if /^reqns: (\\d+)/ }\n' > "$TMPD/pp31decoy.sh"
printf 'while (<$fh>) { push @s, $1 if /^reqns: (\\d+)/ && / w=10 / }\n' > "$TMPD/pp35decoy.sh"
printf '# TODO reqns-guard.pl\nwhile (<$fh>) { push @s, $1 if /^reqns: (\\d+)/ }\n' > "$TMPD/pp40decoy.sh"
printf 'perl "$HERE/reqns-guard.pl" "$raw" || exit 1\nwhile (<$fh>) { push @s, $1 if /^reqns: (\\d+)/ }\n' > "$TMPD/pp40positive.sh"
pp35_ok() { # <file> -> 0 if the two-tier sweep would accept it
  grep -qE '^[^#]*perl[^#]*reqns-guard[.]pl' "$1" || grep -qE 'w=1([^0-9]|$)' "$1"
}
if ! pp35_ok "$TMPD/pp31decoy.sh" && ! pp35_ok "$TMPD/pp35decoy.sh" \
   && ! pp35_ok "$TMPD/pp40decoy.sh" && pp35_ok "$TMPD/pp40positive.sh"; then
  echo "OK  self-test: A-PP35/A-PP40 decoys discriminate (no-rejection, w=10-only, comment-only ALL caught; real invocation accepted)"
else
  echo "SELF-TEST BROKEN: A-PP35/A-PP40 decoy accepted or positive rejected (vacuity regressed)"; exit 2
fi
# The shared parser must exist, be tracked, and its own bite-test must bite.
RGUARD="$REPO/wp86-harness/reqns-guard.pl"
if [ -f "$RGUARD" ] && git -C "$REPO" ls-files --error-unmatch "wp86-harness/reqns-guard.pl" >/dev/null 2>&1 \
   && perl "$RGUARD" --selftest >/dev/null 2>&1; then
  echo "OK  shared reqns-guard.pl exists, tracked, selftest bites (A-PP35)"
else
  echo "FAIL: reqns-guard.pl missing/untracked/selftest-failing (A-PP35/KS-PP-87-1)"
  FAILS=$((FAILS+1))
fi
# A-DS38 (Council WP-88): the shared eviction-pair checker must exist, be
# tracked, and its own bite-test must bite (KS-DS-88-1: an armed log with
# main_evicted rows that has not passed it is order-ADVISORY).
PGUARD="$REPO/wp87-harness/putord-pair-guard.pl"
if [ -f "$PGUARD" ] && git -C "$REPO" ls-files --error-unmatch "wp87-harness/putord-pair-guard.pl" >/dev/null 2>&1 \
   && perl "$PGUARD" --selftest >/dev/null 2>&1; then
  echo "OK  shared putord-pair-guard.pl exists, tracked, selftest bites (A-DS38)"
else
  echo "FAIL: putord-pair-guard.pl missing/untracked/selftest-failing (A-DS38/KS-DS-88-1)"
  FAILS=$((FAILS+1))
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

# --- 6. Sigilli v5 (Council WP-88, Hoare A-TH41/A-TH42) ---------------------
# A-TH41/KH88-1: the A-TH37 marker payload and the noprobe gate's grep are
# coupled by CONVENTION only — a rename of either side compiles green,
# selftest green, and the tooth dies in silence. Pin the coupling:
#   (a) the payload literal appears EXACTLY once in vm/mod.rs;
#   (b) gate-binary-noprobe.sh carries the SAME literal on an EXECUTABLE
#       (non-comment) line — single-source, machine-checked.
TH41_PAYLOAD='phpr_vm_gate_probe_tainted_a_th37'
NOPROBE="$REPO/wp85-harness/gate-binary-noprobe.sh"
n=$(grep -c "$TH41_PAYLOAD" "$VMMOD" || true)
if [ "$n" -ne 1 ]; then
  echo "FAIL: A-TH37 marker payload count in vm/mod.rs == $n, expected 1 (A-TH41/KH88-1)"
  FAILS=$((FAILS+1))
else
  echo "OK  A-TH37 marker payload ==1 in vm/mod.rs (A-TH41)"
fi
if grep -qE "^[^#]*${TH41_PAYLOAD}" "$NOPROBE"; then
  echo "OK  gate-binary-noprobe.sh greps the FULL marker payload on an executable line (A-TH41 single-source)"
else
  echo "FAIL: gate-binary-noprobe.sh has no executable grep of the marker payload — coupling by convention only (A-TH41/KH88-1)"
  FAILS=$((FAILS+1))
fi

# A-TH42: grafie residue del WP-88 audit, PER NOME.
# (1) UFCS mint: `::production_gate(` / `::vm_gate(` — the method pins
#     demand the dot; UFCS is the same call without it. ==0 workspace-wide
#     (definitions spell `fn production_gate`, never `::production_gate(`).
# (2) spacing-tolerant method mint: `. vm_gate (` etc. ==0 outside the
#     named sites (the exact-adjacency pins above stay as the site pins).
# (3) alias mints: `= CachedUnit;` / `= VmGate` (type alias) and
#     `transmute as` (renamed transmute) ==0 non-test.
# (4) multiline: `CachedUnit` at end-of-line with `{` opening on the NEXT
#     line (the awk line-based functional-update guard cannot arm depth
#     there) ==0; transmute/impl-Copy joined across lines ==0.
cat > "$TMPD/decoy_th42.rs" <<'EOF'
fn f() {
    let a = RetainSet::production_gate(&rs);
    let b = MainUnit::vm_gate(&u);
    let c = retain . production_gate ();
    type CU = CachedUnit;
    type G<'a> = VmGate<'a>;
    use std::mem::transmute as tm;
    let d = CachedUnit
    { fp, ..o };
    let e: VmGate = unsafe { std::mem::
        transmute(()) };
}
impl Copy
for VmGate<'_> {}
EOF
n=$(count_nontest "$TMPD/decoy_th42.rs" '::production_gate[(]|::vm_gate[(]')
m=$(count_nontest "$TMPD/decoy_th42.rs" '[.][[:space:]]+production_gate|[.][[:space:]]+vm_gate')
a=$(count_nontest "$TMPD/decoy_th42.rs" '=[[:space:]]*CachedUnit|=[[:space:]]*VmGate|transmute[[:space:]]+as[[:space:]]')
ml=$(awk '/CachedUnit[[:space:]]*$/ { pend=1; next } pend && /^[[:space:]]*\{/ { n++ } { pend=0 } END { print n+0 }' "$TMPD/decoy_th42.rs")
tj=$(tr '\n' ' ' < "$TMPD/decoy_th42.rs" | grep -cE 'transmute[^;]{0,200}[(][)][^;]{0,40};|impl[[:space:]]+Copy[[:space:]]+for[[:space:]]+VmGate' || true)
num_or_void "A-TH42 decoys" "$n" "$m" "$a" "$ml" "$tj"
if [ "$n" -ne 2 ] || [ "$m" -ne 1 ] || [ "$a" -lt 3 ] || [ "$ml" -ne 1 ] || [ "$tj" -lt 1 ]; then
  echo "SELF-TEST BROKEN: A-TH42 decoy counts ufcs=$n spaced=$m alias=$a multiline=$ml joined=$tj (expected 2/1/>=3/1/>=1)"; exit 2
fi
echo "OK  self-test: A-TH42 decoys bite (UFCS 2, spaced 1, alias >=3, multiline 1, joined >=1)"
TH42_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" '::production_gate[(]|::vm_gate[(]|[.][[:space:]]+production_gate|[.][[:space:]]+vm_gate|=[[:space:]]*CachedUnit;|=[[:space:]]*VmGate|transmute[[:space:]]+as[[:space:]]')
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$TH42_SWEEP" ]; then
  echo "FAIL: A-TH42 eluded spelling (UFCS/spaced/alias) found:"
  echo "$TH42_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no UFCS/spaced/alias mint spellings in the workspace (A-TH42)"
fi
TH42_ML=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  while IFS= read -r -d '' f; do
    ml=$(awk '/struct CachedUnit/ { next }
              /CachedUnit[[:space:]]*$/ { pend=1; next }
              pend && /^[[:space:]]*\{/ { n++ } { pend=0 }
              END { print n+0 }' "$f")
    tj=$(tr '\n' ' ' < "$f" | grep -cE 'transmute[^;]{0,200}VmGate|VmGate[^;]{0,200}transmute|impl[[:space:]]+Copy[[:space:]]+for[[:space:]]+(VmGate|G<)' || true)
    t=$((ml + tj))
    [ "$t" -gt 0 ] && echo "${f#"$REPO"/}: ml=$ml joined=$tj"
  done)
if [ -n "$TH42_ML" ]; then
  echo "FAIL: A-TH42 multiline/joined spelling found:"
  echo "$TH42_ML"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no multiline CachedUnit-literal opener nor joined transmute/impl on VmGate (A-TH42)"
fi

# --- 7. Sigilli v6 (Council WP-89: A-TH44/A-TH45/A-MS41) --------------------
# A-TH44: grafie che bucavano ENTRAMBI gli sweep v5, PER NOME (Hoare Q3):
# (1) `.vm_gate (` — spazio SOLO pre-parentesi (il pin di sede esige `(`
#     adiacente, lo sweep spaced esigeva spazio DOPO il punto);
# (2) alias PATH-QUALIFIED `type CU = vm::CachedUnit;` (la regex v5 esige
#     CachedUnit adiacente all'uguale);
# (3) `use …::CachedUnit as CU;` / `use …::VmGate as G;` (il belt A-TH28
#     copre solo vm_gate_probe, mai i TIPI).
cat > "$TMPD/decoy_th44.rs" <<'EOF'
fn g() {
    let a = retain.production_gate ();
    let b = u.vm_gate  (x);
    type CU = vm::CachedUnit;
    type CU2 = crate::vm::CachedUnit;
    use crate::vm::CachedUnit as AliasedCU;
    use vm::gate::VmGate as AliasedG;
}
EOF
sp=$(count_nontest "$TMPD/decoy_th44.rs" '[.][[:space:]]*(production_gate|vm_gate)[[:space:]]+[(]')
pq=$(count_nontest "$TMPD/decoy_th44.rs" '=[[:space:]]*([A-Za-z_][A-Za-z0-9_]*::)+(CachedUnit|VmGate)')
ua=$(count_nontest "$TMPD/decoy_th44.rs" 'use[[:space:]].*::(CachedUnit|VmGate)[[:space:]]+as[[:space:]]')
if [ "$sp" -ne 2 ] || [ "$pq" -ne 2 ] || [ "$ua" -ne 2 ]; then
  echo "SELF-TEST BROKEN: A-TH44 decoy counts spaced-paren=$sp path-alias=$pq use-as=$ua (expected 2/2/2)"; exit 2
fi
echo "OK  self-test: A-TH44 decoys bite (spaced-paren 2, path-qualified alias 2, use-as 2)"
TH44_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" '[.][[:space:]]*(production_gate|vm_gate)[[:space:]]+[(]|=[[:space:]]*([A-Za-z_][A-Za-z0-9_]*::)+(CachedUnit|VmGate)|use[[:space:]].*::(CachedUnit|VmGate)[[:space:]]+as[[:space:]]')
    [ "$n" -gt 0 ] && echo "${f#"$REPO"/}: $n"
  done)
if [ -n "$TH44_SWEEP" ]; then
  echo "FAIL: A-TH44 eluded spelling (spaced-paren / path-qualified alias / use-as) found:"
  echo "$TH44_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no spaced-paren mint, path-qualified type alias, nor use-as alias of CachedUnit/VmGate (A-TH44)"
fi

# A-TH45 (Hoare Q2): the v5 pin verified the literal on a non-comment line
# of noprobe — an `echo "<payload>"` satisfied it while the DETECTION grep
# used another string. Pin the literal as the ARGUMENT of a grep/strings
# invocation. THIRD COPY DECLARED: this script's TH41_PAYLOAD variable is
# copy #3 of the literal (vm/mod.rs #1, noprobe #2) — a three-way rename
# without th41-positive re-run in the same commit is KH89-2 (campaigns
# VOID); the coupling comment in vm/mod.rs names all three.
if grep -qE "^[^#]*(grep|strings)[^#]*${TH41_PAYLOAD}" "$NOPROBE"; then
  echo "OK  noprobe payload literal is an ARGUMENT of the detection grep/strings line (A-TH45)"
else
  echo "FAIL: noprobe payload literal not on a grep/strings invocation line — detection decoupled (A-TH45/KH89-2)"
  FAILS=$((FAILS+1))
fi

# A-MS41 (Matsakis): the probe flag has EXACTLY one arm/disarm pair, both
# inside the ProbeWindow RAII (A-MS40) — a second probe site or a manual
# arm dies here, not in a review.
WPOOL="$REPO/crates/php-server/src/worker_pool.rs"
nset=$(grep -c 'CENSUS_PROBE_ACTIVE\.with(|f| f\.set(' "$WPOOL" || true)
ntrue=$(grep -c 'CENSUS_PROBE_ACTIVE\.with(|f| f\.set(true))' "$WPOOL" || true)
nfalse=$(grep -c 'CENSUS_PROBE_ACTIVE\.with(|f| f\.set(false))' "$WPOOL" || true)
armctx=$(awk '/fn arm\(\) -> ProbeWindow/{w=3; next} w>0 { if (/CENSUS_PROBE_ACTIVE\.with\(\|f\| f\.set\(true\)\)/) found=1; w-- } END{print found+0}' "$WPOOL")
dropctx=$(awk '/impl Drop for ProbeWindow/{w=4; next} w>0 { if (/CENSUS_PROBE_ACTIVE\.with\(\|f\| f\.set\(false\)\)/) found=1; w-- } END{print found+0}' "$WPOOL")
if [ "$nset" = 2 ] && [ "$ntrue" = 1 ] && [ "$nfalse" = 1 ] && [ "$armctx" = 1 ] && [ "$dropctx" = 1 ]; then
  echo "OK  probe flag sites ==2 (one pair), arm in ProbeWindow::arm, disarm in its Drop (A-MS40/A-MS41)"
else
  echo "FAIL: probe flag sites set=$nset true=$ntrue false=$nfalse arm-in-ctor=$armctx clear-in-drop=$dropctx (expected 2/1/1/1/1) — arm outside the unique constructor (A-MS41/KS-MS-89-1)"
  FAILS=$((FAILS+1))
fi

# --- 8. Sigilli v7 (Council WP-90: A-TH48/A-TH49 + A-MS43/44/45 + A-PP48) ---
# A-TH48(a): the noprobe behavioral positive (tainted2.bin) lived in a
# --selftest NOBODY ran in the ledgered chain — run it HERE (KH90-1's
# machine half: a payload rename without this PASS in the same commit
# voids campaigns).
if bash "$REPO/wp85-harness/gate-binary-noprobe.sh" --selftest >/dev/null 2>&1; then
  echo "OK  noprobe --selftest PASS in the ledgered chain (A-TH48/KH90-1)"
else
  echo "FAIL: gate-binary-noprobe --selftest failed — behavioral positive dead (A-TH48/KH90-1)"
  FAILS=$((FAILS+1))
fi
# A-TH48(b): the A-TH45 literal-as-grep-argument pin, SCOPED to probe_in()
# — a decoy `grep -q 'payload' /dev/null` elsewhere in noprobe no longer
# satisfies it.
PROBE_IN_BODY=$(awk '/^probe_in\(\)/{w=1} w{print} w && /^}/{exit}' "$NOPROBE")
if echo "$PROBE_IN_BODY" | grep -qE "^[^#]*(grep|strings)[^#]*${TH41_PAYLOAD}"; then
  echo "OK  payload literal is a grep/strings ARGUMENT inside probe_in() (A-TH48 scoped)"
else
  echo "FAIL: payload literal not on a grep/strings line INSIDE probe_in() (A-TH48/KH89-2)"
  FAILS=$((FAILS+1))
fi

# A-TH49 (Hoare Q1 WP-90): the five graphies that STILL evaded A-TH42/44,
# with decoys that must bite in this same commit:
# (1) raw identifier `r#production_gate`; (2) call split across lines;
# (3) interposed comment `./*x*/production_gate(`; (4) spaced/global `::`
# alias; (5) use-GROUP alias.
cat > "$TMPD/decoy_th49.rs" <<'EOF'
fn h() {
    let a = retain.r#production_gate();
    let b = u.vm_gate
        (x);
    let c = retain./*x*/production_gate();
    type CU5 = vm :: CachedUnit;
    type CU6 = ::vm::CachedUnit;
    use crate::vm::{CachedUnit as GroupCU};
}
EOF
r1=$(count_nontest "$TMPD/decoy_th49.rs" 'r#(production_gate|vm_gate)')
r3=$(count_nontest "$TMPD/decoy_th49.rs" '[.]/[*].*[*]/[[:space:]]*(production_gate|vm_gate)')
r4=$(count_nontest "$TMPD/decoy_th49.rs" '=[[:space:]]*([A-Za-z_][A-Za-z0-9_]*)?[[:space:]]*::[[:space:]]+(CachedUnit|VmGate)|=[[:space:]]*::([A-Za-z_][A-Za-z0-9_]*::)*(CachedUnit|VmGate)')
r5=$(count_nontest "$TMPD/decoy_th49.rs" 'use[[:space:]][^;]*[{][^}]*(CachedUnit|VmGate)[[:space:]]+as[[:space:]]')
r2=$(awk '/[.][[:space:]]*(production_gate|vm_gate)[[:space:]]*$/ { pend=1; next }
          pend && /^[[:space:]]*[(]/ { n++ } { pend=0 } END { print n+0 }' "$TMPD/decoy_th49.rs")
num_or_void "A-TH49 decoys" "$r1" "$r2" "$r3" "$r4" "$r5"
if [ "$r1" -ne 1 ] || [ "$r2" -ne 1 ] || [ "$r3" -ne 1 ] || [ "$r4" -ne 2 ] || [ "$r5" -ne 1 ]; then
  echo "SELF-TEST BROKEN: A-TH49 decoy counts raw=$r1 multiline=$r2 comment=$r3 colons=$r4 usegroup=$r5 (expected 1/1/1/2/1)"; exit 2
fi
echo "OK  self-test: A-TH49 decoys bite (raw-ident 1, split-call 1, interposed-comment 1, spaced/global :: 2, use-group 1)"
TH49_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" 'r#(production_gate|vm_gate)|[.]/[*].*[*]/[[:space:]]*(production_gate|vm_gate)|=[[:space:]]*([A-Za-z_][A-Za-z0-9_]*)?[[:space:]]*::[[:space:]]+(CachedUnit|VmGate)|=[[:space:]]*::([A-Za-z_][A-Za-z0-9_]*::)*(CachedUnit|VmGate)|use[[:space:]][^;]*[{][^}]*(CachedUnit|VmGate)[[:space:]]+as[[:space:]]')
    ml=$(awk '/[.][[:space:]]*(production_gate|vm_gate)[[:space:]]*$/ { pend=1; next }
              pend && /^[[:space:]]*[(]/ { n++ } { pend=0 } END { print n+0 }' "$f")
    t=$((n + ml))
    [ "$t" -gt 0 ] && echo "${f#"$REPO"/}: n=$n ml=$ml"
  done)
if [ -n "$TH49_SWEEP" ]; then
  echo "FAIL: A-TH49 eluded spelling (raw/split/comment/spaced-::/use-group) found:"
  echo "$TH49_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: none of the five A-TH49 graphies in the workspace"
fi

# A-MS43 (KS-MS-90-1): the probe flag is PRIVATE — visibility is the seal,
# this check is the belt's confirmation.
npub=$(grep -c 'pub static CENSUS_PROBE_ACTIVE' "$WPOOL" || true)
npriv=$(grep -c '^[[:space:]]*static CENSUS_PROBE_ACTIVE' "$WPOOL" || true)
CPA_OUT=$(find "$REPO/crates" -name '*.rs' ! -name '._*' ! -path '*worker_pool.rs' -print0 |
  while IFS= read -r -d '' f; do
    grep -l 'CENSUS_PROBE_ACTIVE' "$f" 2>/dev/null || true
  done)
if [ "$npub" = 0 ] && [ "$npriv" = 1 ] && [ -z "$CPA_OUT" ]; then
  echo "OK  probe flag PRIVATE to worker_pool (pub=0, decl=1, external refs=0 — A-MS43/KS-MS-90-1)"
else
  echo "FAIL: probe flag visibility broken: pub=$npub priv-decl=$npriv external=[$CPA_OUT] (A-MS43/KS-MS-90-1)"
  FAILS=$((FAILS+1))
fi
# A-MS44: #[must_use] pinned on ProbeWindow (empty-window guard).
nmu=$(grep -c 'must_use = "bind the window' "$WPOOL" || true)
if [ "$nmu" = 1 ]; then
  echo "OK  ProbeWindow carries #[must_use] (A-MS44/KS-MS-90-3)"
else
  echo "FAIL: ProbeWindow #[must_use] count=$nmu != 1 (A-MS44)"
  FAILS=$((FAILS+1))
fi
# A-MS45: belt extended to today's AND tomorrow's write graphies on the
# flag (LocalKey::set / replace / update, direct or in-closure).
nbad=$(grep -cE 'CENSUS_PROBE_ACTIVE[[:space:]]*\.[[:space:]]*(set|replace|update)[[:space:]]*\(' "$WPOOL" || true)
nrepl=$(grep -cE '\.with\(\|[a-z]+\|[[:space:]]*[a-z]+\.(replace|update)\(' "$WPOOL" || true)
if [ "$nbad" = 0 ] && [ "$nrepl" = 0 ]; then
  echo "OK  no LocalKey::set/replace/update graphy on the probe flag (A-MS45)"
else
  echo "FAIL: probe-flag write graphy found: direct=$nbad closure-replace=$nrepl (A-MS45)"
  FAILS=$((FAILS+1))
fi

# A-PP48(b): the a_pp38 teeth are counted in the SOURCE — a gutting of the
# A-PP45/A-PP47 asserts passes the name-grep of gate-axum-tests (class
# A-DS30); these pins die instead. Bump same-commit when teeth evolve.
npp45=$(grep -c 'A-PP45' "$WPOOL" || true)
npp47=$(grep -c 'A-PP47' "$WPOOL" || true)
if [ "$npp45" = 5 ] && [ "$npp47" = 13 ]; then
  echo "OK  a_pp38 teeth counted in source: A-PP45=5 A-PP47=13 (A-PP48)"
else
  echo "FAIL: a_pp38 teeth counts A-PP45=$npp45 (want 5) A-PP47=$npp47 (want 13) — gutted or unpinned bump (A-PP48)"
  FAILS=$((FAILS+1))
fi

# --- 9. Sigilli v8 (Council WP-91: A-TH52/A-TH56 + A-MS47/48) ---------------
# A-TH52≡A-MS48 (team-sigilli convergence 1): SINGLE-SOURCE — the
# self-test exercises the SAME composed regex/awk variables production
# uses (model TH33RE). The v7 self-test exercised per-graphy regexes
# while the sweep used a RE-TYPED alternation: a branch lost in the sweep
# left the self-test green (class A-PP48).
# Extended v8 branches (Hoare WP-91 Q1 graphies 1-6): dot-at-EOL with
# name on the NEXT line; interposed comment/blank lines inside a split
# call; comment not adjacent to the dot / between name and paren
# (single-line); spaced multi-segment `::` and global spaced `::`;
# use-group MULTILINE and `use …\n as`; r# on TYPES.
# DECLARED residuals (out of lexical reach, A-TH53/A-MS27 lane):
# multi-LINE block comments inside a path, and ident-pasting macros.
# v9 (Council WP-92, Hoare Q1): A-TH-58 adds the SINGLE-LINE `use … as`
# branch (the most natural spelling was the only uncovered one: the use
# branch demanded `{`, the awk `next`ed on one-line use, the alias net
# demanded `=`); A-TH-59 extends the awk with comment/blank tolerance on
# the DOT branch, the THREE-line split (dot / name / paren) and the
# trailing-`//` name (the pend rule demanded name at EOL).
# v10 (Council WP-93, Hoare — A-TH-65): ProbeWindow alias/type-alias
# branches added (the net covered ONLY CachedUnit|VmGate: `use …
# ProbeWindow as PW` and `type P = ProbeWindow` eluded it and left the
# A-TH-57 2==2 pin green with an aliased third site). Invariant DECLARED
# (awk-slash audit): TH49RE_V8 contains NO backslash — -v would eat it.
TH49RE_V8='r[#](production_gate|vm_gate|CachedUnit|VmGate)|[.][[:space:]]*[/][*].*[*][/][[:space:]]*(production_gate|vm_gate)|[.](production_gate|vm_gate)[[:space:]]*[/][*].*[*][/][[:space:]]*[(]|=[[:space:]]*([A-Za-z_][A-Za-z0-9_]*)?[[:space:]]*(::[[:space:]]*[A-Za-z_][A-Za-z0-9_]*[[:space:]]*)*::[[:space:]]+(r[#])?(CachedUnit|VmGate)|=[[:space:]]*::[[:space:]]*([A-Za-z_][A-Za-z0-9_]*[[:space:]]*::[[:space:]]*)*(r[#])?(CachedUnit|VmGate)|::[[:space:]]*r[#](CachedUnit|VmGate)|use[[:space:]][^;]*[{][^}]*(CachedUnit|VmGate)[[:space:]]+as[[:space:]]|use[[:space:]][^;{]*(CachedUnit|VmGate)[[:space:]]+as[[:space:]]|use[[:space:]][^;]*ProbeWindow[[:space:]]+as[[:space:]]|type[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*=[^;]*ProbeWindow'
TH49ML_PROG='
  /[.][[:space:]]*(production_gate|vm_gate)[[:space:]]*(\/\/.*)?$/ { pend=1; dot=0; nmp=0; next }
  /[.][[:space:]]*(\/\/.*)?$/ { dot=1; pend=0; nmp=0; next }
  /^[[:space:]]*use[[:space:]]/ && /;[[:space:]]*$/ { pend=0; dot=0; nmp=0; next }
  /^[[:space:]]*use[[:space:]]/ { useopen=1; seen=0 }
  useopen && /(CachedUnit|VmGate)/ { seen=1 }
  useopen && seen && /[[:space:]]as[[:space:]]/ { n++; seen=0 }
  useopen && seen && /[[:space:]]as[[:space:]]*$/ { n++; seen=0 }
  useopen && /;/ { useopen=0; seen=0 }
  pend && (/^[[:space:]]*\/\// || /^[[:space:]]*$/ || /^[[:space:]]*\/[*].*[*]\/[[:space:]]*$/) { next }
  pend && /^[[:space:]]*[(]/ { n++; pend=0; next }
  { pend=0 }
  dot && (/^[[:space:]]*\/\// || /^[[:space:]]*$/ || /^[[:space:]]*\/[*].*[*]\/[[:space:]]*$/) { next }
  dot && /^[[:space:]]*(production_gate|vm_gate)[[:space:]]*[(]/ { n++ }
  dot && /^[[:space:]]*(production_gate|vm_gate)[[:space:]]*(\/\/.*)?$/ { nmp=1; dot=0; next }
  { dot=0 }
  nmp && (/^[[:space:]]*\/\// || /^[[:space:]]*$/ || /^[[:space:]]*\/[*].*[*]\/[[:space:]]*$/) { next }
  nmp && /^[[:space:]]*[(]/ { n++ }
  { nmp=0 }
  END { print n+0 }'
cat > "$TMPD/decoy_th52.rs" <<'EOF'
fn h() {
    let a = retain.r#production_gate();
    let b = u.vm_gate
        (x);
    let c = retain./*x*/production_gate();
    type CU5 = vm :: CachedUnit;
    type CU6 = ::vm::CachedUnit;
    use crate::vm::{CachedUnit as GroupCU};
    let d = u.
        vm_gate(x);
    let e = u.vm_gate
        // interposed comment keeps the split alive
        (x);
    let f = retain. /*c*/ production_gate();
    let g = retain.production_gate/*c*/(x);
    type CU7 = crate :: vm :: CachedUnit;
    type CU8 = vm::r#CachedUnit;
    use crate::vm::{Other,
        CachedUnit as MLGroupCU};
    use crate::vm::CachedUnit
        as MLCU;
    use crate::vm::CachedUnit as CU9;
    let h1 = u.
        // comment on the dot path (A-TH-59)
        vm_gate(x);
    let h2 = u.
        vm_gate
        (x);
    let h3 = u.production_gate // trailing comment (A-TH-59)
        (x);
    let i1 = u. // trailing comment on the NAKED dot (A-TH-62)
        vm_gate(x);
    let i2 = u.vm_gate
        /* whole-line block comment interposed (A-TH-62) */
        (x);
    use crate::vm::{CachedUnit as
        CU93};
    use crate::probe::ProbeWindow as PW93;
    type PW94 = crate::probe::ProbeWindow;
}
EOF
v8n=$(count_nontest "$TMPD/decoy_th52.rs" "$TH49RE_V8")
v8ml=$(awk "$TH49ML_PROG" "$TMPD/decoy_th52.rs" 2>/dev/null)
# fail-closed on non-numeric: a BROKEN awk program (e.g. an unescaped /
# inside a bracket class — macOS awk terminates the pattern there) used
# to print errors and leave the count EMPTY; `[ "" -ne 8 ]` errors with
# status 2, the || treats it as false, and every ml tooth downstream
# went VACUOUS silently. A count that is not a number is a broken judge.
case "${v8n}x${v8ml}" in *[!0-9x]*|x*|*x) echo "SELF-TEST BROKEN: non-numeric decoy counts v8n='$v8n' v8ml='$v8ml' — awk/regex program is broken, gate VOID"; exit 2;; esac
if [ "$v8n" -ne 12 ] || [ "$v8ml" -ne 11 ]; then
  echo "SELF-TEST BROKEN: A-TH52/58/59/62/65/67 decoys against the PRODUCTION regex/awk: single-line=$v8n (want 12) multiline=$v8ml (want 11)"; exit 2
fi
echo "OK  self-test: A-TH52 v10 decoys bite through the SINGLE-SOURCE production regex (12 single-line + 11 multiline — A-TH-62 naked-dot-comment/block-line, A-TH-65 ProbeWindow alias/type-alias, A-TH-67 use-as at EOL)"
TH52_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' -print0 |
  while IFS= read -r -d '' f; do
    n=$(count_nontest "$f" "$TH49RE_V8")
    ml=$(awk "$TH49ML_PROG" "$f")
    t=$((n + ml))
    [ "$t" -gt 0 ] && echo "${f#"$REPO"/}: n=$n ml=$ml"
  done)
if [ -n "$TH52_SWEEP" ]; then
  echo "FAIL: A-TH52 eluded spelling found (v8 single-source sweep):"
  echo "$TH52_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  sweep: no A-TH52 v8 graphy in the workspace (single-source with the self-test)"
fi

# A-MS48: belt on the probe flag closes the four NAMED holes (generic
# closure binder, take/swap, UFCS LocalKey, multiline split) — decoys
# bite through the SAME composed production regexes, same-commit.
MS48_ANYBIND='CENSUS_PROBE_ACTIVE[[:space:]]*\.[[:space:]]*with[[:space:]]*[(][|][A-Za-z_][A-Za-z0-9_]*[|][[:space:]]*[A-Za-z_][A-Za-z0-9_]*[[:space:]]*\.[[:space:]]*(set|replace|update|take|swap)[[:space:]]*[(]'
MS48_DIRECT='CENSUS_PROBE_ACTIVE[[:space:]]*\.[[:space:]]*(set|replace|update|take|swap)[[:space:]]*[(]'
MS48_UFCS='LocalKey[[:space:]]*::[[:space:]]*(set|replace|update|take|swap)[^;]*CENSUS_PROBE_ACTIVE|CENSUS_PROBE_ACTIVE[^;]*LocalKey[[:space:]]*::[[:space:]]*(set|replace|update|take|swap)'
MS48_SPLIT_PROG='
  /CENSUS_PROBE_ACTIVE[[:space:],.&)]*$/ { pend=1; next }
  pend && /(\.|::)[[:space:]]*(with|set|replace|update|take|swap)[[:space:]]*[(]/ { n++ }
  { pend=0 }
  END { print n+0 }'
cat > "$TMPD/decoy_ms48.rs" <<'EOF'
fn h() {
    CENSUS_PROBE_ACTIVE.with(|g| g.set(true));
    CENSUS_PROBE_ACTIVE.take();
    LocalKey::set(&CENSUS_PROBE_ACTIVE, true);
    CENSUS_PROBE_ACTIVE
        .with(|q| q.set(false));
}
EOF
d1=$(grep -cE "$MS48_ANYBIND" "$TMPD/decoy_ms48.rs" || true)
d2=$(grep -cE "$MS48_DIRECT" "$TMPD/decoy_ms48.rs" || true)
d3=$(grep -cE "$MS48_UFCS" "$TMPD/decoy_ms48.rs" || true)
d4=$(awk "$MS48_SPLIT_PROG" "$TMPD/decoy_ms48.rs")
num_or_void "A-MS48 decoys" "$d1" "$d2" "$d3" "$d4"
if [ "$d1" -ne 1 ] || [ "$d2" -ne 1 ] || [ "$d3" -ne 1 ] || [ "$d4" -ne 1 ]; then
  echo "SELF-TEST BROKEN: A-MS48 decoys anybind=$d1 direct=$d2 ufcs=$d3 split=$d4 (want 1/1/1/1)"; exit 2
fi
echo "OK  self-test: A-MS48 decoys bite (generic binder, take, UFCS, multiline split)"
nany=$(grep -cE "$MS48_ANYBIND" "$WPOOL" || true)
ndir=$(grep -cE "$MS48_DIRECT" "$WPOOL" || true)
nufcs=$(grep -cE "$MS48_UFCS" "$WPOOL" || true)
nsplit=$(awk "$MS48_SPLIT_PROG" "$WPOOL")
if [ "$nany" = 2 ] && [ "$ndir" = 0 ] && [ "$nufcs" = 0 ] && [ "$nsplit" = 0 ]; then
  echo "OK  probe-flag write graphies: with-closure==2 (the legal pair, binder-checked by A-MS41), direct/UFCS/split==0 (A-MS48/KS-MS-91-1)"
else
  echo "FAIL: probe-flag graphy census any-binder=$nany (want 2) direct=$ndir ufcs=$nufcs split=$nsplit (want 0/0/0) (A-MS48/KS-MS-91-1)"
  FAILS=$((FAILS+1))
fi

# A-MS47: the #[must_use] seal is machine-distinguishable only at the USE
# site — `let _ = arm()` SILENCES the lint AND drops immediately (the
# wildcard does not bind). Pin the named binding, ban the silencers.
# comment/attribute lines carry the SPELLING as documentation (the
# must_use message itself says `let _w = …`) — only CODE lines count.
# ==2 NAMED sites (bump S-90.0, same-commit discipline A-PP48): the
# teardown probe (A-DL24 block) and the A-DL49 v2 per-worker self-census
# probe — both `let probe_window = ProbeWindow::arm()`.
narm=$(grep -vE '^[[:space:]]*(//|#\[)' "$WPOOL" | grep -cE 'let (_[a-z0-9][a-z0-9_]*|[a-z][a-z0-9_]*) = ProbeWindow::arm\(\)' || true)
nsil=$(grep -vE '^[[:space:]]*(//|#\[)' "$WPOOL" | grep -cE 'let _ = ProbeWindow::arm|^[[:space:]]*ProbeWindow::arm\(\);' || true)
if [ "$narm" = 2 ] && [ "$nsil" = 0 ]; then
  echo "OK  ProbeWindow::arm bound to NAMED bindings (==2: teardown probe + A-DL49 self-census) and never silenced (A-MS47/KS-MS-90-3)"
else
  echo "FAIL: ProbeWindow::arm sites named=$narm (want 2) silenced/nude=$nsil (want 0) (A-MS47)"
  FAILS=$((FAILS+1))
fi
# A-MS47 clippy half: -D clippy::let_underscore_must_use in the chain —
# a future `let _ = ProbeWindow::arm()` dies at lint, not at review.
# Scoped --no-deps: lints only this workspace's crates (cached after the
# first run; the clippy profile never touches the parity binaries).
if ( cd "$REPO" && cargo clippy -q -p php-server --features axum-server --release --no-deps -- -D clippy::let_underscore_must_use -A warnings ) >/dev/null 2>&1; then
  echo "OK  clippy -D let_underscore_must_use clean on php-server (A-MS47)"
else
  echo "FAIL: cargo clippy -D clippy::let_underscore_must_use refused php-server (A-MS47)"
  FAILS=$((FAILS+1))
fi

# A-TH56 (KH91-3): noprobe structural pins — the double-definition and
# open-window evasions die BEFORE the verdict.
npdef=$(grep -c '^probe_in()' "$NOPROBE" || true)
if [ "$npdef" = 1 ]; then
  echo "OK  noprobe carries EXACTLY one column-0 probe_in() definition (A-TH56/KH91-3)"
else
  echo "FAIL: noprobe probe_in() definitions ==$npdef, want 1 — decoy-front/gutting-behind double definition (A-TH56/KH91-3)"
  FAILS=$((FAILS+1))
fi
PROBE_IN_V8=$(awk '/^probe_in\(\)/{w=1} w{print} w && /^}/{exit}' "$NOPROBE")
if [ "$(echo "$PROBE_IN_V8" | tail -1)" = "}" ]; then
  echo "OK  probe_in() window CLOSED by a column-0 brace (A-TH56/KH91-3)"
else
  echo "FAIL: probe_in() awk window never closed — the extraction swallowed the file (A-TH56/KH91-3)"
  FAILS=$((FAILS+1))
fi
nform=$(echo "$PROBE_IN_V8" | grep -cE "^[^#]*strings[[:space:]]+--[[:space:]]+\"\\\$1\"[^#]*grep -q '${TH41_PAYLOAD}'" || true)
if [ "$nform" = 1 ]; then
  echo "OK  detection line FORM-pinned inside probe_in(): strings -- \"\$1\" | grep -q '<full payload>' ==1 (A-TH56)"
else
  echo "FAIL: full-payload detection line form-pin count=$nform, want 1 (A-TH56 — an echo/decoy inside probe_in no longer satisfies A-TH45)"
  FAILS=$((FAILS+1))
fi

# --- 10. Sigilli v9+v10 (WP-92: A-TH-57..61 + quinta rete A-MS48 + A-TH53;
# WP-93: A-TH-62..67 — KS-TH-93-1: nessuna cifra m91 con probe
# VERDICT-grade finché A-TH-63/65 non mordono)
# A-TH-57 (KS-TH-92-1) — census TOTALE dei siti arm, in forma PREFISSO:
# narm counts only `let <nome> = ProbeWindow::arm()` — a third site
# spelled `let mut w =`, `let w: ProbeWindow =`, `let (w,_) =`,
# `drop(ProbeWindow::arm())` or in expression position was invisible to
# narm AND nsil (the pin bit on converting the 2 known sites, not on
# adding one). ntot counts EVERY code-line occurrence of the PREFIX
# `ProbeWindow::arm` (no parens — SEQUENCE CONSTRAINT, team-sigilli: a
# future `arm(&'static str)` signature (A-MS-52, design) would break an
# empty-paren pin; the prefix census survives it; narm's own regex must
# be updated in the SAME commit as any signature change).
# A-TH-65 (Council WP-93, Hoare Q2): the fixed-string census was TOTAL on
# ONE spelling in ONE file — `ProbeWindow :: arm` spaced, aliases and
# out-of-file sites all eluded it while 2==2 stayed green. The census is
# now an ERE (spaces tolerated) + a WORKSPACE sweep; alias/type-alias
# spellings are routed to TH49RE_V8 (branches added, decoys above). The
# wording is emended accordingly: total ON THE ERE NET, per file + sweep.
TH65_RX='ProbeWindow[[:space:]]*::[[:space:]]*arm'
d65=$(printf 'let w = ProbeWindow :: arm();\n' | grep -cE "$TH65_RX" || true)
num_or_void "A-TH-65 decoy" "$d65"
if [ "$d65" -ne 1 ]; then
  echo "SELF-TEST BROKEN: A-TH-65 spaced-ERE decoy=$d65 (want 1)"; exit 2
fi
ntot=$(grep -vE '^[[:space:]]*(//|#\[)' "$WPOOL" | grep -cE "$TH65_RX" || true)
num_or_void "A-TH-57/65 ntot" "$ntot" "$narm"
if [ "$ntot" = 2 ] && [ "$ntot" = "$narm" ]; then
  echo "OK  ProbeWindow::arm census on the ERE net (prefix form, spaces tolerated) ==2 == narm (A-TH-57/A-TH-65/KS-TH-92-1)"
else
  echo "FAIL: ProbeWindow::arm ERE census ntot=$ntot (want 2, == narm=$narm) — an arm site invisible to the named-binding pin exists (A-TH-57/A-TH-65/KS-TH-92-1: campaign figures with probe VOID)"
  FAILS=$((FAILS+1))
fi
TH65_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' ! -path '*php-server/src/worker_pool.rs' -print0 |
  while IFS= read -r -d '' f; do
    c=$(grep -vE '^[[:space:]]*(//|#\[)' "$f" | grep -cE "$TH65_RX" || true)
    case "$c" in (''|*[!0-9]*) echo "${f#"$REPO"/}: NON-NUMERIC"; continue;; esac
    [ "$c" -gt 0 ] && echo "${f#"$REPO"/}: $c"
  done)
if [ -n "$TH65_SWEEP" ]; then
  echo "FAIL: ProbeWindow::arm site(s) OUTSIDE worker_pool.rs (A-TH-65 workspace sweep):"
  echo "$TH65_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  workspace sweep: ProbeWindow::arm lives only in worker_pool.rs (A-TH-65)"
fi

# A-TH-63 (Council WP-93, Hoare Q1.3 — the widest v9 hole): single-line
# name-space-paren (`retain.production_gate (x)`) and turbofish
# (`.vm_gate::<>(x)`) eluded EVERY v9 net — each spelling demanded an
# adjacent `(` or an inline comment; a third call site in those grafie
# left the ==2 pin GREEN. New net in PREFIX form (A-TH-57 model, no call
# anchor): TOTAL census of `.name` on code lines, per file, plus the
# workspace sweep. Multi-line dot splits stay with the ML program.
cat > "$TMPD/decoy_th63.rs" <<'EOF'
fn h() {
    let a = retain.production_gate (x);
    let b = u.vm_gate::<>(x);
}
EOF
TH63_RX='[.][[:space:]]*(production_gate|vm_gate)([^A-Za-z0-9_]|$)'
d63=$(grep -vE '^[[:space:]]*(//|#\[)' "$TMPD/decoy_th63.rs" | grep -cE "$TH63_RX" || true)
num_or_void "A-TH-63 decoys" "$d63"
if [ "$d63" -ne 2 ]; then
  echo "SELF-TEST BROKEN: A-TH-63 prefix decoys name-space-paren/turbofish=$d63 (want 2)"; exit 2
fi
echo "OK  self-test: A-TH-63 prefix net bites (name-space-paren, turbofish)"
n63v=$(grep -vE '^[[:space:]]*(//|#\[)' "$VMMOD" | grep -cE "$TH63_RX" || true)
n63w=$(grep -vE '^[[:space:]]*(//|#\[)' "$WPOOL" | grep -cE "$TH63_RX" || true)
num_or_void "A-TH-63 census" "$n63v" "$n63w"
if [ "$n63v" = 1 ] && [ "$n63w" = 1 ]; then
  echo "OK  .production_gate/.vm_gate PREFIX census: vm/mod.rs==1 worker_pool.rs==1 (A-TH-63)"
else
  echo "FAIL: dot-name PREFIX census vm/mod=$n63v (want 1) worker_pool=$n63w (want 1) — a call site in an eluded spelling exists (A-TH-63)"
  FAILS=$((FAILS+1))
fi
TH63_SWEEP=$(find "$REPO/crates" -name '*.rs' ! -name '._*' ! -path '*php-runtime/src/vm/mod.rs' ! -path '*php-server/src/worker_pool.rs' -print0 |
  while IFS= read -r -d '' f; do
    c=$(grep -vE '^[[:space:]]*(//|#\[)' "$f" | grep -cE "$TH63_RX" || true)
    case "$c" in (''|*[!0-9]*) echo "${f#"$REPO"/}: NON-NUMERIC"; continue;; esac
    [ "$c" -gt 0 ] && echo "${f#"$REPO"/}: $c"
  done)
if [ -n "$TH63_SWEEP" ]; then
  echo "FAIL: dot-name PREFIX found outside the two pinned files (A-TH-63):"
  echo "$TH63_SWEEP"
  FAILS=$((FAILS+1))
else
  echo "OK  workspace sweep: dot-name PREFIX only in the two pinned files (A-TH-63)"
fi

# Quinta rete A-MS48 (Hoare Q2.2, WP-92): point-free `with(Cell::set)`,
# closure calling `Cell::set(f, true)`, and BRACED closure body
# `|f| { f.set(true) }` evaded the four v8 nets (anybind demands
# `|b| b.set`; UFCS demands `LocalKey::`). Same-commit decoys.
MS48_NET5='CENSUS_PROBE_ACTIVE[^;]*Cell[[:space:]]*::[[:space:]]*(set|replace|update|take|swap)|CENSUS_PROBE_ACTIVE[[:space:]]*\.[[:space:]]*with[[:space:]]*[(][|][A-Za-z_][A-Za-z0-9_]*[|][[:space:]]*[{][[:space:]]*[A-Za-z_][A-Za-z0-9_]*[[:space:]]*\.[[:space:]]*(set|replace|update|take|swap)'
cat > "$TMPD/decoy_ms48v9.rs" <<'EOF'
fn h() {
    CENSUS_PROBE_ACTIVE.with(Cell::set);
    CENSUS_PROBE_ACTIVE.with(|f| Cell::set(f, true));
    CENSUS_PROBE_ACTIVE.with(|f| { f.set(true) });
}
EOF
d5=$(grep -cE "$MS48_NET5" "$TMPD/decoy_ms48v9.rs" || true)
num_or_void "A-MS48 net5 decoys" "$d5"
if [ "$d5" -ne 3 ]; then
  echo "SELF-TEST BROKEN: A-MS48 fifth-net decoys point-free/Cell-call/braced=$d5 (want 3)"; exit 2
fi
echo "OK  self-test: A-MS48 fifth net bites (point-free, Cell-call, braced body)"
n5=$(grep -cE "$MS48_NET5" "$WPOOL" || true)
if [ "$n5" = 0 ]; then
  echo "OK  probe-flag fifth-net graphies ==0 in production (A-MS48 v9)"
else
  echo "FAIL: probe-flag fifth-net graphy found in worker_pool ($n5) (A-MS48 v9)"
  FAILS=$((FAILS+1))
fi

# A-TH-60 (KS-TH-92-2) — probe_in DEFINITION census in ANY spelling:
# npdef counted only column-0 `probe_in()`; an INDENTED redefinition or a
# `function probe_in {` form after the decoy passed npdef==1 while bash
# used the LAST (gutted) definition. Call sites start with `if` and do
# not match the anchored definition census (verified on the real file).
npdef2=$(grep -cE '^[[:space:]]*(function[[:space:]]+)?probe_in' "$NOPROBE" || true)
if [ "$npdef2" = 1 ]; then
  echo "OK  noprobe probe_in DEFINITION census ==1 in any spelling (A-TH-60/KS-TH-92-2)"
else
  echo "FAIL: noprobe probe_in definition census ==$npdef2, want 1 — indented/function-form redefinition (A-TH-60/KS-TH-92-2: VERDICTs leaning on noprobe VOID)"
  FAILS=$((FAILS+1))
fi

# A-TH53 (design90, attuazione v9) — ident-pasting macros are beyond ANY
# lexical seal (grafia 7): pin ==0 occurrences of paste/concat_idents in
# the Cargo.toml of the two probe-bearing crates until A-MS27 (rustc as
# judge) closes the lane for real. Lexical limit DECLARED, not closed.
npaste=$(grep -cE 'paste|concat_idents' "$REPO/crates/php-runtime/Cargo.toml" "$REPO/crates/php-server/Cargo.toml" 2>/dev/null | awk -F: '{s+=$2} END{print s+0}')
if [ "$npaste" = 0 ]; then
  echo "OK  no paste/concat_idents in php-runtime/php-server Cargo.toml (A-TH53 pin; closure = A-MS27)"
else
  echo "FAIL: paste/concat_idents occurrence(s) in probe-crate Cargo.toml ($npaste) (A-TH53)"
  FAILS=$((FAILS+1))
fi

# A-TH-61 — the TLS doc carries the THIRD state (never-initialized at
# teardown) and the lost-write channel. A-TH-66 (Council WP-93, Hoare Q4):
# the -ge 1 presence pin was satisfiable by an EMPTIED doc keeping the
# marker; the pin is now ==1 on the marker PLUS the distinctive phrases of
# the emendation (UC_STATS second victim, fallback leaked box, own-dtor
# subsumption) — content-pinned, not marker-pinned.
nth61=$(grep -c 'A-TH-61' "$REPO/crates/php-runtime/src/vm/mod.rs" || true)
nth66a=$(grep -c 'same channel, second victim' "$REPO/crates/php-runtime/src/vm/mod.rs" || true)
nth66b=$(grep -c 'write lost AND memory retained' "$REPO/crates/php-runtime/src/vm/mod.rs" || true)
nth66c=$(grep -c 'SUBSUMED under already-destroyed' "$REPO/crates/php-runtime/src/vm/mod.rs" || true)
num_or_void "A-TH-61/66 doc pins" "$nth61" "$nth66a" "$nth66b" "$nth66c"
if [ "$nth61" = 1 ] && [ "$nth66a" = 1 ] && [ "$nth66b" = 1 ] && [ "$nth66c" = 1 ]; then
  echo "OK  vm/mod.rs TLS doc carries A-TH-61 (==1) + A-TH-66 phrases (UC_STATS lost-write, fallback leaked box, own-dtor subsumption)"
else
  echo "FAIL: TLS doc pins A-TH-61=$nth61 (want 1) UC_STATS-victim=$nth66a leak=$nth66b own-dtor=$nth66c (want 1/1/1) — emendation absent or doc emptied around the marker (A-TH-66)"
  FAILS=$((FAILS+1))
fi

# A-PP52 belt note: the external npfail==2 counter lives in
# gate-axum-tests.sh (the armed-UCL consumer); here we only pin that the
# tooth exists there — a gutting of that gate's counter dies here.
npf=$(grep -c 'main_probe_fail' "$REPO/wp88-harness/gate-axum-tests.sh" || true)
if [ "$npf" -ge 1 ]; then
  echo "OK  gate-axum-tests carries the A-PP52 npfail counter (KS-PP-91-2 lift armed)"
else
  echo "FAIL: gate-axum-tests has NO main_probe_fail counter — A-PP52 not armed (KS-PP-91-2)"
  FAILS=$((FAILS+1))
fi

if [ "$FAILS" = 0 ]; then
  echo "== GATE-LEVER-PINS PASS (A-MS13 + A-PP16 + KS-PP-82-3 + A-TH14 + v5 A-TH41/42 + v6 A-TH44/45 A-MS41 + v7 A-TH48/49 A-MS43/44/45 A-PP48 + v8 A-TH52/56 A-MS47/48 + v9 A-TH-57/58/59/60/61 A-MS48-net5 A-TH53 + v10 A-TH-62/63/64/65/66/67) [git $GIT_REV] =="
  exit 0
else
  echo "== GATE-LEVER-PINS FAIL($FAILS) [git $GIT_REV] =="
  exit 1
fi
