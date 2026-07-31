#!/bin/bash
# gate-parity-82p2.sh — S-82.0 p2 parity gate for the A-TH21 one-shot fold.
# Sequential, detached (project rule): corpus Zend BY NAME vs corpus81.fails,
# refl BY NAME vs refl81.fails, then workspace `cargo test --release`.
# Verdict = SET identity per NOME (gate-diff-fail-set-not-count), never counts.
export PATH=/usr/bin:/bin:/usr/sbin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$HERE/.." && pwd)"
OUT="$HOME/Claude/php-rust-output/release"
CORPUS="/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests"
REFL="/Volumes/Extreme Pro/Claude/php-8.5.7/ext/reflection/tests"
EV="$REPO/wp81-harness/evidence"
# OUTSIDE the repo (S-82.0 battery lesson): this gate re-runs inside the
# battery and its logs are TRACKED-file churn that dirties the tree for the
# next feature-matrix whole-tree porcelain. The committed wp82-harness/
# p2-parity/ snapshot stays as the p2-era evidence; live runs write here.
W="/Volumes/Extreme Pro/Claude/wp82-battery-out/p2-parity"
mkdir -p "$W"
FAILS=0

extract_fails() { # <runner-log> -> sorted fail paths on stdout
  awk '/^failures: /{f=1; next} f && /^  \//{sub(/^  /,""); print}' "$1" | sort
}

echo "== corpus Zend (--isolate) =="
"$OUT/phpt-runner" --isolate "$CORPUS" > "$W/corpus82p2.log" 2>&1
extract_fails "$W/corpus82p2.log" > "$W/corpus82p2.fails"
sort "$EV/corpus81.fails" > "$W/corpus81.sorted"
if diff -q "$W/corpus81.sorted" "$W/corpus82p2.fails" > /dev/null; then
  echo "OK  corpus: fail set BY NAME identical ($(wc -l < "$W/corpus82p2.fails" | tr -d ' ') files)"
else
  echo "FAIL corpus: fail set DIFFERS by name:"
  diff "$W/corpus81.sorted" "$W/corpus82p2.fails" | head -40
  FAILS=$((FAILS+1))
fi

echo "== refl (--isolate) =="
"$OUT/phpt-runner" --isolate "$REFL" > "$W/refl82p2.log" 2>&1
extract_fails "$W/refl82p2.log" > "$W/refl82p2.fails"
sort "$EV/refl81.fails" > "$W/refl81.sorted"
if diff -q "$W/refl81.sorted" "$W/refl82p2.fails" > /dev/null; then
  echo "OK  refl: fail set BY NAME identical ($(wc -l < "$W/refl82p2.fails" | tr -d ' ') files)"
else
  echo "FAIL refl: fail set DIFFERS by name:"
  diff "$W/refl81.sorted" "$W/refl82p2.fails" | head -40
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

if [ "$FAILS" = 0 ]; then echo "== GATE-PARITY-82P2 PASS =="; else echo "== GATE-PARITY-82P2 FAIL($FAILS) =="; fi
touch "$W/.done"
exit $FAILS
