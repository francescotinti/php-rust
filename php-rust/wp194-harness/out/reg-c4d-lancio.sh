#!/bin/bash
# reg-c4d-lancio.sh — S-194: fresh instruction ledger per op of the ORM leg on commit 9b9b5f11 (after P1-P4 and C4), once the C4d
# chain has ended. Outcome: s194-registro-op-C4d.out, s194-registro-op-C4d-orm.txt. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"
until [ -f "$W/out/p-coda-C4d.done" ]; do sleep 20; done
cp "$W/s194-registro-op.out" "$W/out/registro-op-prima-C4d.out"; : > "$W/s194-registro-op.out"
python3 "$W/s194-registro-op.py" 9b9b5f11 orm > "$W/out/reg-c4d.log" 2>&1; echo "rc=$?" > "$W/out/reg-c4d.done.tmp"
cp "$W/s194-registro-op.out" "$W/s194-registro-op-C4d.out"; cp "$W/s194-registro-op-orm.txt" "$W/s194-registro-op-C4d-orm.txt"
cat "$W/out/registro-op-prima-C4d.out" "$W/s194-registro-op-C4d.out" > "$W/s194-registro-op.out"
mv "$W/out/reg-c4d.done.tmp" "$W/out/reg-c4d.done"
