#!/bin/bash
# s194-orm-F.sh <commit> — ORM instruction count after package F: pin, pre-N and N binaries kept from the earlier run, F built from the commit archive.
# Outcome: s194-orm-istruzioni-F.out. Generato con assistenza AI (Claude Fable 5.2).
W="$(cd "$(dirname "$0")" && pwd)"; rm -f "$W/out/orm-F.done"
cp "$W/s194-orm-istruzioni.out" "$W/s194-orm-istruzioni-1.out"; : > "$W/s194-orm-istruzioni.out"
python3 "$W/s194-orm-istruzioni.py" pin="$HOME/Claude/php-rust-output/release/phpr" preN="$W/out/phpr-orm-istr-preN" N="$W/out/phpr-orm-istr-N" F="$1" > "$W/out/orm-istruzioni-F.log" 2>&1
cp "$W/s194-orm-istruzioni.out" "$W/s194-orm-istruzioni-F.out"; cp "$W/s194-orm-istruzioni-1.out" "$W/s194-orm-istruzioni.out"; rm -f "$W/s194-orm-istruzioni-1.out"
echo "$(cat "$W/out/orm-istruzioni.rc" 2>/dev/null || echo 99)" > "$W/out/orm-F.done"
