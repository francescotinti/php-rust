#!/bin/bash
# object-id probe for the package N behaviour gate: oracle, pin and arm on the same global-namespace file. Generato con assistenza AI (Claude Fable 5.2).
W="$(cd "$(dirname "$0")" && pwd)"
for e in /opt/homebrew/opt/php/bin/php "$HOME/Claude/php-rust-output/release/phpr" "$HOME/Claude/phpr-target/s192-armD/release/phpr"; do echo "== $e"; "$e" "$W/s194-n-id-sonda.php" 2>/dev/null | grep -E '^object|^[0-9 a-z|]+$' | tr '\n' ' '; echo; done > "$W/out/n-id-sonda.out"
