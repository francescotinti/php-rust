#!/bin/bash
# p3-lancio.sh — S-194 P3: the path fixtures changed header (class Q added), so their pin reading is taken again (P3base) and
# joined to the whole-probe pin reading Pbase (every other fixture) as s194-primitive-Pbase3.out; then p-coda.sh P3 P2 with that base.
# Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")/.." && pwd)"
bash "$W/s194-primitive.sh" P3base --only noop,p_isset_str,p_isset_miss,p_isset2,p_assign2,p_incdec,p_field_isset,p_field_isset_obj,p_field_assign,p_field_incdec,p_this_isset,p_this_assign,p_this_set,p_prop_isset,p_empty_str,p_priv_set,p_priv_assign,p_priv_typed
{ grep -v -E '^(p_|confronto|  |note)' "$W/s194-primitive-Pbase.out"; grep -E '^p_' "$W/s194-primitive-P3base.out"; } > "$W/s194-primitive-Pbase3.out"
S194_BASE=s194-primitive-Pbase3.out bash "$W/out/p-coda.sh" P3 P2
