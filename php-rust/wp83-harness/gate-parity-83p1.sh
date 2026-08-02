#!/bin/bash
# gate-parity-83p1.sh — S-83.0 p1 parity gate for the A-MS17/A-MS22 seal
# commit (VmGate token + lower_net→ticket; signature-only, zero semantics).
# Sequential, detached (project rule): corpus Zend BY NAME vs corpus81.fails
# (set verified IDENTICAL through S-82.0 battery-5), refl BY NAME vs
# refl81.fails, then workspace `cargo test --release`.
# Verdict = SET identity per NOME (gate-diff-fail-set-not-count), never counts.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$HOME/Claude/php-rust-output/release"
CORPUS="/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
REFL="/Volumes/Extreme Pro/Claude/php-8.5.7/ext/reflection/tests"
EV="$REPO/wp81-harness/evidence"
# OUTSIDE the repo (S-82.0 battery lesson): live gate output is tracked-file
# churn that would dirty the whole-tree porcelain of the next feature matrix.
W="/Volumes/Extreme Pro/Claude/wp83-battery-out/p1-parity"
mkdir -p "$W"
FAILS=0

extract_fails() { # <runner-log> -> sorted fail paths on stdout
  awk '/^failures: /{f=1; next} f && /^  \//{sub(/^  /,""); print}' "$1" | sort
}

echo "== corpus Zend (--isolate) =="
"$OUT/phpt-runner" --isolate "$CORPUS" > "$W/corpus83p1.log" 2>&1
extract_fails "$W/corpus83p1.log" > "$W/corpus83p1.fails"
sort "$EV/corpus81.fails" > "$W/corpus81.sorted"
if diff -q "$W/corpus81.sorted" "$W/corpus83p1.fails" > /dev/null; then
  echo "OK  corpus: fail set BY NAME identical ($(wc -l < "$W/corpus83p1.fails" | tr -d ' ') files)"
else
  echo "FAIL corpus: fail set DIFFERS by name:"
  diff "$W/corpus81.sorted" "$W/corpus83p1.fails" | head -40
  FAILS=$((FAILS+1))
fi

echo "== refl (--isolate) =="
"$OUT/phpt-runner" --isolate "$REFL" > "$W/refl83p1.log" 2>&1
extract_fails "$W/refl83p1.log" > "$W/refl83p1.fails"
sort "$EV/refl81.fails" > "$W/refl81.sorted"
if diff -q "$W/refl81.sorted" "$W/refl83p1.fails" > /dev/null; then
  echo "OK  refl: fail set BY NAME identical ($(wc -l < "$W/refl83p1.fails" | tr -d ' ') files)"
else
  echo "FAIL refl: fail set DIFFERS by name:"
  diff "$W/refl81.sorted" "$W/refl83p1.fails" | head -40
  FAILS=$((FAILS+1))
fi

echo "== workspace cargo test --release =="
( cd "$REPO" && cargo test --release ) > "$W/cargo-test.log" 2>&1
rc=$?
if [ $rc -ne 0 ]; then
  echo "FAIL workspace tests rc=$rc:"
  grep -E "^(test .* FAILED|error|failures:)" "$W/cargo-test.log" | head -20
  FAILS=$((FAILS+1))
else
  echo "OK  workspace tests (rc=0)"
fi

# A-MS37 (Council WP-88, Matsakis): the VmGate !Send/!Sync doctests are
# COUNTED, never presumed — if the pub re-export fell or VmGate went
# doc(hidden), rustdoc would collect ZERO doctests and rc=0 would bless a
# battery with the seal silently dead (KS-MS-88-2: parity-full PASS with
# count != 2 => A-MS33 not proven, seal ADVISORY).
NDOC=$(grep -cE 'vm::gate::VmGate.*compile fail.*ok' "$W/cargo-test.log" || true)
if [ "$NDOC" = 2 ]; then
  echo "OK  VmGate compile-fail doctests counted ==2 (A-MS37/KS-MS-88-2)"
else
  echo "FAIL: VmGate compile-fail doctests counted == $NDOC, expected 2 (A-MS37/KS-MS-88-2)"
  FAILS=$((FAILS+1))
fi

if [ "$FAILS" = 0 ]; then echo "== GATE-PARITY-83P1 PASS =="; else echo "== GATE-PARITY-83P1 FAIL($FAILS) =="; fi
touch "$W/.done"
exit $FAILS
