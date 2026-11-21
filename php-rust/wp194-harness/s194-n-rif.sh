#!/bin/bash
# s194-n-rif.sh — reference outputs of the package N behaviour teeth on the pin and on the oracle, BEFORE the code. Generato con assistenza AI (Claude Fable 5.2).
W="$(cd "$(dirname "$0")" && pwd)"
"$HOME/Claude/php-rust-output/release/phpr" "$W/s194-n-comportamento.php" > "$W/out/n-comportamento.pin" 2>&1; echo "pin rc=$? righe=$(wc -l < "$W/out/n-comportamento.pin" | tr -d ' ')" > "$W/out/n-rif.out"
/opt/homebrew/opt/php/bin/php "$W/s194-n-comportamento.php" > "$W/out/n-comportamento.oracle" 2> /dev/null; echo "oracle rc=$?" >> "$W/out/n-rif.out"
diff "$W/out/n-comportamento.oracle" "$W/out/n-comportamento.pin" > "$W/out/n-comportamento-oracle-pin.diff"; echo "diff oracle/pin righe=$(wc -l < "$W/out/n-comportamento-oracle-pin.diff" | tr -d ' ')" >> "$W/out/n-rif.out"
