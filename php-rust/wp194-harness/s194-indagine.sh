#!/bin/bash
# s194-indagine.sh — S-194 call-cost investigation (phase I of the D-bis plan), no production code touched:
# new probe fixtures on the pin-recipe arm, bytecode of the loops, caller attribution from the recorded profile,
# twin profiles of abs and count. Outcome: s194-indagine.out, out/indagine.done. Generato con assistenza AI (Claude Fable 5.2).
set -u
export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin:$HOME/.cargo/bin
W="$(cd "$(dirname "$0")" && pwd)"; ARM="$HOME/Claude/phpr-target/s192-armD"; V="$W/s194-indagine.out"; : > "$V"; rm -f "$W/out/indagine.done"
echo "braccio sha=$(shasum -a 256 "$ARM/release/phpr" | cut -c1-16)" >> "$V"
S194_BIN="$ARM/release/phpr" python3 "$W/s194-chiamanti.py" builtin_strlen drop_glue clone libdyld to_bool to_zstr > "$W/out/indagine-chiamanti.log" 2>&1
echo "chiamanti strlen: rc=$?" >> "$V"
bash "$W/s194-primitive.sh" I1 --bin "$ARM/release/phpr" --only noop,assign_const,add_assign,float_add,builtin_strlen,builtin_abs,builtin_count,builtin_pi,builtin_abs_lit,builtin_intdiv,builtin_ord,builtin_strcont,builtin_strlen2
echo "sonda I1: rc=$(cat "$W/s194-primitive-I1.rc")" >> "$V"
for f in builtin_strlen builtin_abs builtin_count builtin_pi builtin_abs_lit builtin_intdiv builtin_ord builtin_strcont; do
  PHPR_DUMP_OPS=1 "$ARM/release/phpr" "$W/primitive/$f.php" 3 2> "$W/primitive/$f.ops" > /dev/null
  awk '/^-- fn run /{p=1} p' "$W/primitive/$f.ops" > "$W/out/indagine-ops-$f.txt"
done
echo "bytecode: fatto" >> "$V"
for f in builtin_abs builtin_count; do
  S194_BIN="$ARM/release/phpr" S194_SUF=-D python3 "$W/s194-profilo-fixture.py" $f 40000000 > "$W/out/indagine-profilo-$f.log" 2>&1
  echo "profilo $f: rc=$?" >> "$V"
  S194_BIN="$ARM/release/phpr" python3 "$W/s194-chiamanti.py" $f drop_glue clone libdyld > /dev/null 2>&1
done
echo "fine $(date '+%F %T')" >> "$V"; echo 0 > "$W/out/indagine.done"
