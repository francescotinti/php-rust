#!/bin/bash
# s194-profilo-passo.sh <fixture> <suffix> — line profile of one fixture on the pin-recipe arm (S-194 D-bis steps). Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")" && pwd)"; ARM="$HOME/Claude/phpr-target/s192-armD"; rm -f "$W/out/profilo-passo.done"
S194_BIN="$ARM/release/phpr" S194_SUF="$2" python3 "$W/s194-profilo-fixture.py" "$1" 40000000 > "$W/out/profilo-passo.log" 2>&1
echo $? > "$W/out/profilo-passo.done"
