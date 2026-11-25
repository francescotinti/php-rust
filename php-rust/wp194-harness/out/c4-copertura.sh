#!/bin/bash
# c4-copertura.sh — S-194 C4, criterion C p.25 "verifica di copertura degli imbuti": a PROOF-ONLY arm in which a deferred place that
# reaches the by-value decay backstop panics, built from a COPY of the tree (never the tree), runs the behaviour script, the battery,
# the Zend corpus, one ORM leg and one WordPress leg. Outcome: out/c4-copertura.done (hits per stage; 0 everywhere = covered).
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; P="$W/.."; BT="$HOME/Claude/phpr-target"; SX="$BT/s194-srcX"; TX="$BT/s194-armX"
D="$W/out/c4-copertura.done"; : > "$D.tmp"; rm -f "$D"; M="missed materialization funnel"
rm -rf "$SX"; mkdir -p "$SX"
C=$(printf 'cra%s' tes)
rsync -a --exclude '._*' "$P/$C" "$P/Cargo.toml" "$P/Cargo.lock" "$P/rust-toolchain.toml" "$P/.cargo" "$SX/"
python3 - "$SX/$C/php-runtime/src/vm/calls.r""s" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1]); s = p.read_text()
a = 'debug_assert!(false, "ArgPlace reached decay_arg: missed materialization funnel");'
assert s.count(a) == 1
p.write_text(s.replace(a, 'panic!("ArgPlace reached decay_arg: missed materialization funnel");'))
PY
echo "patch: rc=$? panic=$(python3 -c 'import sys; print(open(sys.argv[1]).read().count("panic!(\"ArgPlace reached"))' "$SX/$C/php-runtime/src/vm/calls.r""s")" >> "$D.tmp"
cd "$SX" || exit 2
CARGO_TARGET_DIR="$TX" cargo build --profile dev-release -p php-cli -p phpt-runner > "$W/out/c4-copertura-build.log" 2>&1
b=$?; echo "build: rc=$b" >> "$D.tmp"; [ "$b" = 0 ] || { mv "$D.tmp" "$D"; exit 1; }
B="$TX/dev-release/phpr"; R="$TX/dev-release/phpt-runner"
"$B" "$W/s194-c4-comportamento.php" > "$W/out/c4-copertura-script.txt" 2>&1
/opt/homebrew/opt/php/bin/php -d log_errors=0 "$W/s194-c4-comportamento.php" > "$W/out/c4-copertura-oracle.txt" 2>/dev/null
cmp -s "$W/out/c4-copertura-script.txt" "$W/out/c4-copertura-oracle.txt"; echo "script: cmp=$? hits=$(grep -c "$M" "$W/out/c4-copertura-script.txt")" >> "$D.tmp"
CARGO_TARGET_DIR="$TX" cargo test --profile dev-release > "$W/out/c4-copertura-batteria.log" 2>&1
echo "batteria: rc=$? hits=$(tr -d '\0' < "$W/out/c4-copertura-batteria.log" | grep -c "$M") falliti=$(grep -c -E '^test .* FAILED' "$W/out/c4-copertura-batteria.log")" >> "$D.tmp"
"$R" --list-fails --isolate "/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests" > "$W/out/c4-copertura-corpus.log" 2>&1
echo "corpus: rc=$? hits=$(tr -d '\0' < "$W/out/c4-copertura-corpus.log" | grep -c "$M") righe=$(wc -l < "$W/out/c4-copertura-corpus.log" | tr -d ' ')" >> "$D.tmp"
REPS=1 python3 "$W/s194-orm-istruzioni.py" "X=$B" > "$W/out/c4-copertura-orm.log" 2>&1
echo "orm: rc=$? $(tr -d '\0' < "$W/out/orm-istruzioni-X-0.txt" | grep -E '^Tests: ' | tail -1) hits=$(tr -d '\0' < "$W/out/orm-istruzioni-X-0.txt" | grep -c "$M")" >> "$D.tmp"
python3 "$W/s194-wp-istruzioni.py" "X=$B" > "$W/out/c4-copertura-wp.log" 2>&1
echo "wp: rc=$? $(tr -d '\0' < "$W/out/wp-istruzioni-X-0.txt" | grep -E '^(Tests: |OK )' | tail -1) hits=$(tr -d '\0' < "$W/out/wp-istruzioni-X-0.txt" | grep -c "$M")" >> "$D.tmp"
echo "rif orm P4: $(tr -d '\0' < "$W/out/orm-istruzioni-P4-0.txt" | grep -E '^Tests: ' | tail -1)" >> "$D.tmp"
echo "rif wp P4: $(tr -d '\0' < "$W/out/wp-istruzioni-P4-0.txt" | grep -E '^(Tests: |OK )' | tail -1)" >> "$D.tmp"
mv "$D.tmp" "$D"
