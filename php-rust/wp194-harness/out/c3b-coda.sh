#!/bin/bash
# c3b-coda.sh — S-194 C3b, one detached chain: (1) probe base reading on the pin s187 with the three new fixtures, (2) arm build +
# probe of the tree, (3) day acceptance, (4) whole-workload instruction counts (ORM, WordPress) pin against the arm.
# Outcome: out/c3b-coda.done (one line per step). Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; D="$W/out/c3b-coda.done"; : > "$D.tmp"; rm -f "$D"
PIN="$HOME/Claude/php-rust-output/release/phpr"; ARM="$W/out/phpr-c3b"
bash "$W/s194-primitive.sh" C3bbase --bin "$PIN" --only noop,ns_noop,call_user0,call_user2,call_typed,call_method,call_static,call_closure,call_this_inc,new_ctor,ns_user2,ret_typed_only,ret_typed_class,call_typed_class,call_typed_iface,mcall_typed,mcall_typed_class,call_default,mcall_default,mcall_private,this_private0,this_protected0
echo "base: rc=$(cat "$W/s194-primitive-C3bbase.rc")" >> "$D.tmp"
bash "$W/s194-passo-C3b.sh" C3b; echo "passo: rc=$? done=$(cat "$W/out/passo-C3b.done")" >> "$D.tmp"
cp "$HOME/Claude/phpr-target/s192-armD/release/phpr" "$ARM"
bash "$W/../wp192-harness/s192-accettazione.sh" s194C3b; echo "accettazione: rc=$?" >> "$D.tmp"
python3 "$W/s194-orm-istruzioni.py" "pin=$PIN" "C3b=$ARM" > "$W/out/c3b-orm.log" 2>&1; echo "orm: rc=$?" >> "$D.tmp"
python3 "$W/s194-wp-istruzioni.py" "pin=$PIN" "C3b=$ARM" > "$W/out/c3b-wp.log" 2>&1; echo "wp: rc=$?" >> "$D.tmp"
mv "$D.tmp" "$D"
