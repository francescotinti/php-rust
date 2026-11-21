#!/bin/bash
# s194-f-rif.sh — reference outputs of the package F behaviour teeth on the pin and on the oracle, BEFORE the code. Generato con assistenza AI (Claude Fable 5.2).
W="$(cd "$(dirname "$0")" && pwd)"
"$HOME/Claude/php-rust-output/release/phpr" "$W/s194-f-comportamento.php" > "$W/out/f-comportamento.pin" 2> /dev/null; echo "pin rc=$? righe=$(wc -l < "$W/out/f-comportamento.pin" | tr -d ' ')" > "$W/out/f-rif.out"
/opt/homebrew/opt/php/bin/php "$W/s194-f-comportamento.php" > "$W/out/f-comportamento.oracle" 2> /dev/null; echo "oracle rc=$?" >> "$W/out/f-rif.out"
diff "$W/out/f-comportamento.oracle" "$W/out/f-comportamento.pin" > "$W/out/f-comportamento-oracle-pin.diff"; echo "diff oracle/pin righe=$(wc -l < "$W/out/f-comportamento-oracle-pin.diff" | tr -d ' ')" >> "$W/out/f-rif.out"
