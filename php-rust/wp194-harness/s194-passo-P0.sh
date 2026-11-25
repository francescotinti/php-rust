#!/bin/bash
# s194-passo-P0.sh — S-194 package P, step 0: baseline of the path/property fixtures on the pin (probe, instructions per iteration) and
# leaf profile of each on the pin-recipe arm (same source as pin s188). Direction only. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")" && pwd)"; ARM="$HOME/Claude/phpr-target/s192-armD"; rm -f "$W/out/passo-P0.done"
"$W/s194-primitive.sh" P0 --only noop,p_isset_str,p_isset_miss,p_isset2,p_assign2,p_incdec,p_field_isset,p_field_isset_obj,p_field_assign,p_field_incdec,p_this_isset,p_this_assign,p_this_set,p_prop_isset,p_empty_str
for f in p_isset_str p_isset_miss p_isset2 p_assign2 p_incdec p_field_isset p_field_isset_obj p_field_assign p_field_incdec p_this_isset p_this_assign p_this_set p_prop_isset p_empty_str; do
  S194_BIN="$ARM/release/phpr" S194_SUF="P0" python3 "$W/s194-profilo-fixture.py" "$f" 20000000 > "$W/out/profilo-P0-$f.log" 2>&1
done
echo "$(cat "$W/s194-primitive-P0.rc")" > "$W/out/passo-P0.done"
