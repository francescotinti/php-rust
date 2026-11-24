#!/bin/bash
# c3a-coda.sh — S-194 C3a, one detached chain: (1) probe base reading on the pin s187 with the two new fixtures, (2) arm build + probe
# of the tree against that base, (3) day acceptance. Outcome: out/c3a-coda.done (one line per step). Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"; D="$W/out/c3a-coda.done"; : > "$D.tmp"; rm -f "$D"
bash "$W/s194-primitive.sh" C3base --bin "$HOME/Claude/php-rust-output/release/phpr" --only noop,ns_noop,call_user0,call_user2,call_typed,call_method,call_static,call_closure,call_this_inc,new_ctor,ns_user2,ret_typed_only,ret_typed_class,call_typed_class,call_typed_iface,mcall_typed,mcall_typed_class,call_default,mcall_default
echo "base: rc=$(cat "$W/s194-primitive-C3base.rc")" >> "$D.tmp"
bash "$W/s194-passo-C3.sh" C3a; echo "passo: rc=$? done=$(cat "$W/out/passo-C3a.done")" >> "$D.tmp"
bash "$W/../wp192-harness/s192-accettazione.sh" s194C3a; echo "accettazione: rc=$?" >> "$D.tmp"
mv "$D.tmp" "$D"
