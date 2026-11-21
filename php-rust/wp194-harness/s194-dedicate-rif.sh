#!/bin/bash
# s194-dedicate-rif.sh — records the reference output of the dedicated-op behaviour teeth on the pin and on the oracle, BEFORE the code.
# Generato con assistenza AI (Claude Fable 5.2).
W="$(cd "$(dirname "$0")" && pwd)"
"$HOME/Claude/php-rust-output/release/phpr" "$W/s194-dedicate-comportamento.php" > "$W/out/dedicate-comportamento.pin" 2>&1; echo "pin rc=$? sha=$(shasum -a 256 "$HOME/Claude/php-rust-output/release/phpr" | cut -c1-16) righe=$(wc -l < "$W/out/dedicate-comportamento.pin" | tr -d ' ')" > "$W/out/dedicate-rif.out"
/opt/homebrew/opt/php/bin/php "$W/s194-dedicate-comportamento.php" > "$W/out/dedicate-comportamento.oracle" 2> /dev/null; echo "oracle rc=$?" >> "$W/out/dedicate-rif.out"
diff "$W/out/dedicate-comportamento.oracle" "$W/out/dedicate-comportamento.pin" > "$W/out/dedicate-comportamento-oracle.diff"; echo "diff oracle/pin righe=$(wc -l < "$W/out/dedicate-comportamento-oracle.diff" | tr -d ' ')" >> "$W/out/dedicate-rif.out"
