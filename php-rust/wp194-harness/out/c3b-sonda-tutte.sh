#!/bin/bash
# c3b-sonda-tutte.sh — S-194 C3b: the WHOLE primitive probe (every fixture) on the pin s187, then on the C3b arm against it:
# where does the arm pay outside the call fixtures? Outcome: out/c3b-sonda-tutte.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; rm -f "$W/out/c3b-sonda-tutte.done"
bash "$W/s194-primitive.sh" C3ballbase --bin "$HOME/Claude/php-rust-output/release/phpr"
bash "$W/s194-primitive.sh" C3ball --bin "$W/out/phpr-c3b" --base s194-primitive-C3ballbase.out
echo "base rc=$(cat "$W/s194-primitive-C3ballbase.rc") arm rc=$(cat "$W/s194-primitive-C3ball.rc")" > "$W/out/c3b-sonda-tutte.done"
