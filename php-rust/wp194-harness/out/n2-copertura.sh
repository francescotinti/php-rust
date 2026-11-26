#!/bin/bash
# n2-copertura.sh — S-194 N2, criterion N p.4: a PROOF-ONLY arm in which unit_fp asserts «incremental digest == whole recomputation»,
# built from a COPY of the tree (never the tree), runs the behaviour scripts, the battery (--no-fail-fast), the Zend corpus, one ORM
# leg and one WordPress leg. DECLARED COPY of n2-copertura.sh (manifest n2-copertura-copia.diff): the patch, the marker, the names.
# Outcome: out/n2-copertura.done (hits per stage; 0 everywhere = proved).
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; P="$W/.."; BT="$HOME/Claude/phpr-target"; SX="$BT/s194-srcX"; TX="$BT/s194-armX"
D="$W/out/n2-copertura.done"; : > "$D.tmp"; rm -f "$D"; M="class_index_fp drift"
rm -rf "$SX"; mkdir -p "$SX"
C=$(printf 'cra%s' tes)
rsync -a --exclude '._*' "$P/$C" "$P/Cargo.toml" "$P/Cargo.lock" "$P/rust-toolchain.toml" "$P/.cargo" "$SX/"
python3 - "$SX/$C/php-runtime/src/vm/mod.r""s" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1]); s = p.read_text()
a = 'debug_assert_eq!(acc, self.class_index_fp_full(), "class_index_fp drift");'
assert s.count(a) == 1
p.write_text(s.replace(a, 'assert_eq!(acc, self.class_index_fp_full(), "class_index_fp drift");'))
PY
echo "patch: rc=$? panic=$(python3 -c 'import sys; print(open(sys.argv[1]).read().count("  assert_eq!(acc, self.class_index_fp_full()"))' "$SX/$C/php-runtime/src/vm/mod.r""s")" >> "$D.tmp"
cd "$SX" || exit 2
CARGO_TARGET_DIR="$TX" cargo build --profile dev-release -p php-cli -p phpt-runner > "$W/out/n2-copertura-build.log" 2>&1
b=$?; echo "build: rc=$b" >> "$D.tmp"; [ "$b" = 0 ] || { mv "$D.tmp" "$D"; exit 1; }
B="$TX/dev-release/phpr"; R="$TX/dev-release/phpt-runner"
"$B" "$W/s194-c4-comportamento.php" > "$W/out/n2-copertura-script.txt" 2>&1
/opt/homebrew/opt/php/bin/php -d log_errors=0 "$W/s194-c4-comportamento.php" > "$W/out/n2-copertura-oracle.txt" 2>/dev/null
cmp -s "$W/out/n2-copertura-script.txt" "$W/out/n2-copertura-oracle.txt"; echo "script: cmp=$? hits=$(grep -c "$M" "$W/out/n2-copertura-script.txt")" >> "$D.tmp"
CARGO_TARGET_DIR="$TX" cargo test --profile dev-release --no-fail-fast > "$W/out/n2-copertura-batteria.log" 2>&1
echo "batteria: rc=$? hits=$(tr -d '\0' < "$W/out/n2-copertura-batteria.log" | grep -c "$M") falliti=$(tr -d '\0' < "$W/out/n2-copertura-batteria.log" | grep -E '^test .* FAILED' | sort -u | tr '\n' ' ')" >> "$D.tmp"
"$R" --list-fails --isolate "/Volumes/Extreme Pro/Claude/php-8.5.7/Zend/tests" > "$W/out/n2-copertura-corpus.log" 2>&1
echo "corpus: rc=$? hits=$(tr -d '\0' < "$W/out/n2-copertura-corpus.log" | grep -c "$M") righe=$(wc -l < "$W/out/n2-copertura-corpus.log" | tr -d ' ')" >> "$D.tmp"
REPS=1 python3 "$W/s194-orm-istruzioni.py" "X=$B" > "$W/out/n2-copertura-orm.log" 2>&1
echo "orm: rc=$? $(tr -d '\0' < "$W/out/orm-istruzioni-X-0.txt" | grep -E '^Tests: ' | tail -1) hits=$(tr -d '\0' < "$W/out/orm-istruzioni-X-0.txt" | grep -c "$M")" >> "$D.tmp"
python3 "$W/s194-wp-istruzioni.py" "X=$B" > "$W/out/n2-copertura-wp.log" 2>&1
echo "wp: rc=$? $(tr -d '\0' < "$W/out/wp-istruzioni-X-0.txt" | grep -E '^(Tests: |OK )' | tail -1) hits=$(tr -d '\0' < "$W/out/wp-istruzioni-X-0.txt" | grep -c "$M")" >> "$D.tmp"
echo "rif orm P4: $(tr -d '\0' < "$W/out/orm-istruzioni-P4-0.txt" | grep -E '^Tests: ' | tail -1)" >> "$D.tmp"
echo "rif wp P4: $(tr -d '\0' < "$W/out/wp-istruzioni-P4-0.txt" | grep -E '^(Tests: |OK )' | tail -1)" >> "$D.tmp"
mv "$D.tmp" "$D"
